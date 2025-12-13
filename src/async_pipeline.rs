//! Async ingest -> normalize -> enrich -> store pipeline with Ollama embeddings and HIBP
//! enrichment.
//!
//! The AsyncPipeline is designed for production use with PostgreSQL storage, async embedding
//! generation via Ollama, and breach enrichment via HIBP. It maintains the same logical
//! structure as the synchronous Pipeline but with support for these advanced features.

use crate::{
	adapters::FormatAdapter, enrichment::EnrichmentPlugin, hash_utils, hibp::HibpClient,
	normalization, ollama::OllamaClient, storage::StorageAdapter,
};

/// Configuration for the async pipeline.
pub struct AsyncPipelineConfig {
	/// Enable Ollama embedding generation for new addresses
	pub enable_embeddings: bool,
	/// Enable HIBP breach lookups for new addresses
	pub enable_hibp: bool,
	/// Threshold for vector similarity when detecting near-duplicates (0.0-1.0)
	pub vector_similarity_threshold: f32,
}

impl Default for AsyncPipelineConfig {
	fn default() -> Self {
		AsyncPipelineConfig {
			enable_embeddings: true,
			enable_hibp: true,
			vector_similarity_threshold: 0.85,
		}
	}
}

/// Async pipeline wires together adapter, enricher, storage, and optional services.
pub struct AsyncPipeline<A: FormatAdapter, E: EnrichmentPlugin, S: StorageAdapter> {
	adapter: A,
	enricher: E,
	storage: S,
	config: AsyncPipelineConfig,
	ollama_client: Option<OllamaClient>,
	hibp_client: Option<HibpClient>,
}

impl<A: FormatAdapter, E: EnrichmentPlugin, S: StorageAdapter> AsyncPipeline<A, E, S> {
	/// Create a new async pipeline with default configuration.
	pub fn new(adapter: A, enricher: E, storage: S) -> Self {
		AsyncPipeline {
			adapter,
			enricher,
			storage,
			config: AsyncPipelineConfig::default(),
			ollama_client: None,
			hibp_client: None,
		}
	}

	/// Create a new async pipeline with custom configuration.
	pub fn with_config(adapter: A, enricher: E, storage: S, config: AsyncPipelineConfig) -> Self {
		AsyncPipeline {
			adapter,
			enricher,
			storage,
			config,
			ollama_client: None,
			hibp_client: None,
		}
	}

	/// Set the Ollama client for embedding generation.
	pub fn with_ollama(mut self, client: OllamaClient) -> Self {
		self.ollama_client = Some(client);
		self
	}

	/// Set the HIBP client for breach enrichment.
	pub fn with_hibp(mut self, client: HibpClient) -> Self {
		self.hibp_client = Some(client);
		self
	}

	/// Ingest input asynchronously, normalize, enrich, and persist rows.
	/// Process includes embedding generation and breach lookups if configured.
	pub async fn ingest(self, input: &str) -> Result<S, std::io::Error> {
		let AsyncPipeline {
			adapter,
			enricher,
			mut storage,
			config,
			ollama_client,
			hibp_client,
		} = self;

		let rows = adapter.parse(input);

		// Compute file-level hashes
		let bytes = input.as_bytes();
		let file_md5 = hash_utils::md5_hex_bytes(bytes);
		let file_sha256 = hash_utils::sha256_hex_bytes(bytes);
		let file_id = file_sha256.clone();

		let meta = vec![
			"__file_hash__".to_string(),
			file_md5.clone(),
			file_sha256.clone(),
		];
		let mut meta_row = meta.clone();
		meta_row.push(format!("file_id:{}", file_id));
		let _ = storage.store_row(&meta_row)?;

		// Detect header row
		let mut header: Option<Vec<String>> = None;
		let mut expected_columns: Option<usize> = None;
		if !rows.is_empty() {
			let first = &rows[0];
			if first.iter().any(|c| c.chars().any(|ch| ch.is_alphabetic())) {
				header = Some(first.clone());
				expected_columns = Some(first.len());
			}
		}

		// Helper closure to store row with file_id
		let store_with_file = |storage: &mut S, row: &[String], file_id: &str| {
			let mut r = row.to_vec();
			r.push(format!("file_id:{}", file_id));
			storage.store_row(&r)
		};

		// Process rows
		for (idx, row) in rows.iter().enumerate() {
			// Skip header row if we treated it as such
			if idx == 0 && header.is_some() {
				continue;
			}

			let normalized = normalization::normalize_row(row);

			// Validate column count
			if let Some(expected) = expected_columns {
				if normalized.len() != expected {
					let raw = row.join(",");
					let m = vec!["__malformed_row__".to_string(), idx.to_string(), raw];
					let _ = store_with_file(&mut storage, &m, &file_id)?;
					continue;
				}
			}

			// Detect address and credential columns
			let (addr_hashes, cred_hashes, has_hashed_credentials) =
				Self::extract_address_credentials(&normalized, &header)?;

			// Skip rows with only hashed credentials (no plaintext address)
			if has_hashed_credentials && !cred_hashes.is_empty() && addr_hashes.is_empty() {
				let ev = vec![
					"__hashed_credentials_only__".to_string(),
					"row_skipped".to_string(),
					format!("cred_count:{}", cred_hashes.len()),
				];
				let _ = store_with_file(&mut storage, &ev, &file_id)?;
				continue;
			}

			// Process address/credential associations
			for addr in addr_hashes.iter() {
				let addr_seen = storage.address_exists(addr)?;

				if !addr_seen {
					// New address: log event and record hash
					let ev = vec!["__new_address__".to_string(), addr.clone()];
					store_with_file(&mut storage, &ev, &file_id)?;

					let r = vec!["__address_hash__".to_string(), addr.clone()];
					store_with_file(&mut storage, &r, &file_id)?;

					// If embeddings enabled and we have plaintext, generate embedding
					if config.enable_embeddings {
						Self::enrich_with_embedding_async(
							&mut storage,
							&normalized,
							addr,
							ollama_client.as_ref(),
							config.vector_similarity_threshold,
						)
						.await?;
					}

					// If HIBP enabled, lookup breaches
					if config.enable_hibp {
						Self::enrich_with_hibp_async(
							&mut storage,
							&normalized,
							addr,
							hibp_client.as_ref(),
						)
						.await?;
					}
				}

				// Process credentials for this address
				for cred in cred_hashes.iter() {
					if !storage.contains_hash(cred)? {
						let r = vec!["__credential_hash__".to_string(), cred.clone()];
						store_with_file(&mut storage, &r, &file_id)?;
					}

					let assoc = storage.address_has_credential(addr, cred)?;
					if !assoc {
						if addr_seen {
							let ev = vec![
								"__known_address_new_credential__".to_string(),
								addr.clone(),
								cred.clone(),
							];
							store_with_file(&mut storage, &ev, &file_id)?;
						}

						let mapping = vec!["__addr_cred__".to_string(), addr.clone(), cred.clone()];
						store_with_file(&mut storage, &mapping, &file_id)?;
					}
				}
			}

			// Enrich row and store
			let mut enriched = enricher.enrich(&normalized);
			let row_join = normalized.join("|");
			let row_hash = hash_utils::sha256_hex(&row_join);

			for h in addr_hashes.iter() {
				enriched.push(format!("addr_sha256:{}", h));
			}
			for h in cred_hashes.iter() {
				enriched.push(format!("cred_sha256:{}", h));
			}

			// Deduplication
			if storage.contains_hash(&row_hash)? {
				let dup = vec!["__duplicate_row__".to_string(), row_hash.clone()];
				let _ = store_with_file(&mut storage, &dup, &file_id)?;
			} else {
				enriched.push(format!("row_hash:{}", row_hash));
				store_with_file(&mut storage, &enriched, &file_id)?;
			}
		}

		Ok(storage)
	}

	/// Extract address and credential hashes from a normalized row.
	fn extract_address_credentials(
		normalized: &[String],
		header: &Option<Vec<String>>,
	) -> Result<(Vec<String>, Vec<String>, bool), std::io::Error> {
		let mut addr_hashes = Vec::new();
		let mut cred_hashes = Vec::new();
		let mut has_hashed_credentials = false;

		if let Some(h) = header {
			for (i, col_name) in h.iter().enumerate() {
				let lname = col_name.to_lowercase();
				if i < normalized.len() {
					let val = &normalized[i];

					// Address detection
					if lname.contains("mail")
						|| lname.contains("email")
						|| lname.contains("addr")
						|| lname.contains("address")
					{
						let sha = hash_utils::sha256_hex(val);
						addr_hashes.push(sha);
					}

					// Credential detection
					if lname.contains("pass")
						|| lname.contains("pwd")
						|| lname.contains("password")
						|| lname.contains("credential")
						|| lname.contains("secret")
					{
						if hash_utils::is_credential_hash(val) {
							has_hashed_credentials = true;
						}
						let sha = hash_utils::sha256_hex(val);
						cred_hashes.push(sha);
					}
				}
			}
		} else {
			// Fallback heuristic
			for val in normalized.iter() {
				if val.contains('@') {
					addr_hashes.push(hash_utils::sha256_hex(val));
				}
				if val.contains(':') || val.to_lowercase().contains("pass") {
					if hash_utils::is_credential_hash(val) {
						has_hashed_credentials = true;
					}
					cred_hashes.push(hash_utils::sha256_hex(val));
				}
			}
		}

		Ok((addr_hashes, cred_hashes, has_hashed_credentials))
	}

	/// Generate embeddings for a new address using Ollama.
	async fn enrich_with_embedding_async(
		storage: &mut S,
		normalized: &[String],
		addr_hash: &str,
		ollama_client: Option<&OllamaClient>,
		similarity_threshold: f32,
	) -> Result<(), std::io::Error> {
		if let Some(ollama) = ollama_client {
			// Extract plaintext address from normalized row (look for email-like field)
			let address_text = normalized
				.iter()
				.find(|v| v.contains('@'))
				.map(|s| s.as_str());

			if let Some(addr_text) = address_text {
				match ollama.embed(addr_text).await {
					Ok(embedding) => {
						// Store embedding in canonical address record
						let _ = storage.update_address_embedding(addr_hash, &embedding);

						// Check for similar addresses
						if !embedding.is_empty() {
							let _ =
								storage.find_similar_addresses(&embedding, 5, similarity_threshold);
						}
					}
					Err(e) => {
						eprintln!("Failed to generate embedding for {}: {}", addr_text, e);
					}
				}
			}
		}

		Ok(())
	}

	/// Lookup breaches for a new address using HIBP API.
	async fn enrich_with_hibp_async(
		storage: &mut S,
		normalized: &[String],
		addr_hash: &str,
		hibp_client: Option<&HibpClient>,
	) -> Result<(), std::io::Error> {
		if let Some(hibp) = hibp_client {
			// Extract plaintext address from normalized row
			if let Some(addr_text) = normalized.iter().find(|v| v.contains('@')) {
				match hibp.get_breaches_for_address(addr_text).await {
					Ok(breaches) => {
						for breach in breaches {
							let _ = storage.insert_address_breach(
								addr_hash,
								&breach.name,
								Some(&breach.title),
								Some(&breach.domain),
								Some(&breach.breach_date),
								Some(breach.pwn_count as i32),
								Some(&breach.description),
								breach.is_verified,
								breach.is_fabricated,
								breach.is_sensitive,
								breach.is_retired,
							);
						}
					}
					Err(e) => {
						eprintln!("Failed to lookup HIBP for {}: {}", addr_text, e);
					}
				}
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeSet;

	use super::*;
	use crate::{adapters::CsvAdapter, enrichment::ChecksumEnricher};

	struct TestStorage {
		rows: Vec<Vec<String>>,
		index: BTreeSet<String>,
		assoc: BTreeSet<(String, String)>,
	}

	impl TestStorage {
		fn new() -> Self {
			TestStorage {
				rows: Vec::new(),
				index: BTreeSet::new(),
				assoc: BTreeSet::new(),
			}
		}
	}

	impl StorageAdapter for TestStorage {
		fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
			for v in row.iter() {
				self.index.insert(v.clone());
			}
			self.rows.push(row.to_vec());
			Ok(())
		}

		fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool> {
			Ok(self.index.contains(hash))
		}

		fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool> {
			Ok(self.index.contains(addr_hash))
		}

		fn address_has_credential(
			&mut self,
			addr_hash: &str,
			cred_hash: &str,
		) -> std::io::Result<bool> {
			Ok(self
				.assoc
				.contains(&(addr_hash.to_string(), cred_hash.to_string())))
		}

		fn add_address_credential(
			&mut self,
			addr_hash: &str,
			cred_hash: &str,
		) -> std::io::Result<()> {
			self.assoc
				.insert((addr_hash.to_string(), cred_hash.to_string()));
			Ok(())
		}
	}

	#[tokio::test]
	async fn async_pipeline_basic_ingest() {
		let csv = "email,password\nalice@example.com,pass123\nbob@example.com,secret456\n";
		let adapter = CsvAdapter::new();
		let enr = ChecksumEnricher::new();
		let store = TestStorage::new();

		let config = AsyncPipelineConfig {
			enable_embeddings: false,
			enable_hibp: false,
			vector_similarity_threshold: 0.85,
		};

		let pipeline = AsyncPipeline::with_config(adapter, enr, store, config);
		let storage = pipeline.ingest(csv).await.expect("ingest");

		// Expect metadata + data rows
		assert!(storage.rows.len() >= 3);
		assert!(
			storage
				.rows
				.iter()
				.any(|r| r.get(0).map(|s| s.as_str()) == Some("__file_hash__"))
		);
	}

	#[tokio::test]
	async fn async_pipeline_detects_new_addresses() {
		let csv = "email,password\nalice@example.com,pass123\n";
		let adapter = CsvAdapter::new();
		let enr = ChecksumEnricher::new();
		let store = TestStorage::new();

		let config = AsyncPipelineConfig {
			enable_embeddings: false,
			enable_hibp: false,
			vector_similarity_threshold: 0.85,
		};

		let pipeline = AsyncPipeline::with_config(adapter, enr, store, config);
		let storage = pipeline.ingest(csv).await.expect("ingest");

		// Should have a __new_address__ event
		assert!(
			storage
				.rows
				.iter()
				.any(|r| r.get(0).map(|s| s.as_str()) == Some("__new_address__"))
		);
	}
}

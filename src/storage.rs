//! Storage adapter examples (filesystem).

use std::{
	fs::{File, OpenOptions},
	io::{self, BufRead, BufReader, Read, Write},
	path::PathBuf,
};

use postgres::{Client, NoTls};

/// Trait for storage backends used in examples.
pub trait StorageAdapter {
	fn store_row(&mut self, row: &[String]) -> std::io::Result<()>;
	fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool>;
	/// Check whether an address (by its hash) has been seen before.
	fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool>;
	/// Check whether a specific credential hash is associated with an address.
	fn address_has_credential(&mut self, addr_hash: &str, cred_hash: &str)
	-> std::io::Result<bool>;
	/// Record an address -> credential mapping in the history store.
	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()>;

	// Canonical address tracking (production schema only; default no-op for filesystem):

	/// Store a canonical email address (SHA256 hash of normalized form).
	fn insert_canonical_address(
		&mut self,
		canonical_hash: &str,
		address_text: &str,
		normalized_form: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, address_text, normalized_form);
		Ok(false)
	}

	/// Record a Unicode alternate representation mapping to a canonical address.
	fn insert_address_alternate(
		&mut self,
		canonical_hash: &str,
		alternate_hash: &str,
		alternate_form: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, alternate_hash, alternate_form);
		Ok(false)
	}

	/// Look up a canonical address by an alternate Unicode hash.
	fn lookup_canonical_by_alternate(
		&mut self,
		alternate_hash: &str,
	) -> std::io::Result<Option<String>> {
		let _ = alternate_hash;
		Ok(None)
	}

	/// Record a credential associated with a canonical address.
	fn insert_address_credential_canonical(
		&mut self,
		canonical_hash: &str,
		credential_hash: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, credential_hash);
		Ok(false)
	}

	/// Get all credentials associated with a canonical address.
	fn get_credentials_for_address(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<String>> {
		let _ = canonical_hash;
		Ok(vec![])
	}

	/// Record a co-occurrence edge between two canonical addresses.
	/// Addresses should be in canonical order (hash_1 < hash_2) to avoid duplicates.
	/// Returns Ok(true) if newly inserted, Ok(false) if already existed (count incremented).
	fn record_address_cooccurrence(
		&mut self,
		canonical_hash_1: &str,
		canonical_hash_2: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash_1, canonical_hash_2);
		Ok(false)
	}

	/// Get all neighbors of a canonical address in the co-occurrence graph.
	/// Returns a vector of (neighbor_canonical_hash, cooccurrence_count) tuples.
	fn get_address_neighbors(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, i32)>> {
		let _ = canonical_hash;
		Ok(vec![])
	}

	/// Update the vector embedding for a canonical address (from Nomic/Ollama).
	/// Embeddings are used for similarity-based near-duplicate detection.
	fn update_address_embedding(
		&mut self,
		canonical_hash: &str,
		embedding: &[f32],
	) -> std::io::Result<()> {
		let _ = (canonical_hash, embedding);
		Ok(())
	}

	/// Find similar canonical addresses by vector similarity (cosine distance).
	/// Returns up to `limit` results as (canonical_hash, similarity_score) tuples,
	/// sorted by descending similarity.
	fn find_similar_addresses(
		&mut self,
		embedding: &[f32],
		limit: usize,
		similarity_threshold: f32,
	) -> std::io::Result<Vec<(String, f32)>> {
		let _ = (embedding, limit, similarity_threshold);
		Ok(vec![])
	}

	/// Check if a canonical address is likely a duplicate of an existing one
	/// based on hash match OR vector similarity above threshold.
	/// Returns Some(canonical_hash) if duplicate found, None otherwise.
	fn find_duplicate_address(
		&mut self,
		canonical_hash: &str,
		embedding: Option<&[f32]>,
		similarity_threshold: f32,
	) -> std::io::Result<Option<String>> {
		let _ = (canonical_hash, embedding, similarity_threshold);
		Ok(None)
	}

	/// Store breach data for a canonical address from HIBP API.
	/// Returns Ok(true) if newly inserted, Ok(false) if already existed.
	fn insert_address_breach(
		&mut self,
		canonical_hash: &str,
		breach_name: &str,
		breach_title: Option<&str>,
		breach_domain: Option<&str>,
		breach_date: Option<&str>,
		pwn_count: Option<i32>,
		description: Option<&str>,
		is_verified: bool,
		is_fabricated: bool,
		is_sensitive: bool,
		is_retired: bool,
	) -> std::io::Result<bool> {
		let _ = (
			canonical_hash,
			breach_name,
			breach_title,
			breach_domain,
			breach_date,
			pwn_count,
			description,
			is_verified,
			is_fabricated,
			is_sensitive,
			is_retired,
		);
		Ok(false)
	}

	/// Get all breaches for a canonical address.
	/// Returns a vector of (breach_name, pwn_count) tuples.
	fn get_breaches_for_address(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, i32)>> {
		let _ = canonical_hash;
		Ok(vec![])
	}

	/// Get count of breaches affecting a canonical address.
	fn get_breach_count(&mut self, canonical_hash: &str) -> std::io::Result<i32> {
		let _ = canonical_hash;
		Ok(0)
	}

	/// Get total exposed count (sum of pwn_count across all breaches).
	fn get_total_pwn_count(&mut self, canonical_hash: &str) -> std::io::Result<i32> {
		let _ = canonical_hash;
		Ok(0)
	}
}

/// Filesystem-based storage that appends CSV lines to a file.
pub struct FsStorage {
	path: PathBuf,
	file: File,
}

/// Postgres-backed storage adapter. Requires a `normalized_rows` table
/// similar to the one created in `docker/init-db.sql`.
pub struct PostgresStorage {
	client: Client,
	dataset: Option<String>,
}

impl PostgresStorage {
	pub fn new(conn_str: &str, dataset: Option<String>) -> std::io::Result<Self> {
		let client = Client::connect(conn_str, NoTls)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(PostgresStorage { client, dataset })
	}

	/// Create from environment or default to the docker-compose connection string.
	pub fn new_from_env() -> std::io::Result<Self> {
		let conn = std::env::var("DUMPTRUCK_PG_CONN").unwrap_or_else(|_| {
			// default connection string for docker-compose service
			"postgresql://dumptruck:dumpturck@dumptruck-db/dumptruck".to_string()
		});
		let dataset = std::env::var("DUMPTRUCK_DATASET").ok();
		PostgresStorage::new(&conn, dataset)
	}
}

impl FsStorage {
	pub fn new(path: PathBuf) -> std::io::Result<Self> {
		let file = OpenOptions::new().create(true).append(true).open(&path)?;
		Ok(FsStorage { path, file })
	}

	pub fn read_all(path: &PathBuf) -> std::io::Result<String> {
		let mut s = String::new();
		let mut f = File::open(path)?;
		f.read_to_string(&mut s)?;
		Ok(s)
	}
}

impl StorageAdapter for FsStorage {
	fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
		// naive CSV escaping: wrap fields with quotes if they contain comma or quote
		let mut parts: Vec<String> = Vec::with_capacity(row.len());
		for f in row {
			if f.contains(',') || f.contains('"') || f.contains('\n') {
				let esc = f.replace('"', "\"\"");
				parts.push(format!("\"{}\"", esc));
			} else {
				parts.push(f.clone());
			}
		}
		let line = parts.join(",") + "\n";
		self.file.write_all(line.as_bytes())?;
		Ok(())
	}

	fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool> {
		// look for an __address_hash__ line containing the addr_hash
		let f = File::open(&self.path)?;
		let mut reader = BufReader::new(f);
		let mut line = String::new();
		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.contains("__address_hash__") && line.contains(addr_hash) {
				return Ok(true);
			}
		}
		Ok(false)
	}

	fn address_has_credential(
		&mut self,
		addr_hash: &str,
		cred_hash: &str,
	) -> std::io::Result<bool> {
		// look for a mapping row we write as __addr_cred__,addr_hash,cred_hash
		let f = File::open(&self.path)?;
		let mut reader = BufReader::new(f);
		let mut line = String::new();
		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.contains("__addr_cred__")
				&& line.contains(addr_hash)
				&& line.contains(cred_hash)
			{
				return Ok(true);
			}
		}
		Ok(false)
	}

	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()> {
		let row = vec![
			"__addr_cred__".to_string(),
			addr_hash.to_string(),
			cred_hash.to_string(),
		];
		self.store_row(&row)
	}

	fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool> {
		// Stream the file line-by-line so we never load the entire file into memory.
		// This is important when the storage file is larger than available RAM.
		let f = File::open(&self.path)?;
		let mut reader = BufReader::new(f);
		let mut line = String::new();
		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.contains(hash) {
				return Ok(true);
			}
		}
		Ok(false)
	}
}

impl StorageAdapter for PostgresStorage {
	fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
		// Production storage: map well-known row shapes into dedicated columns
		// so dev and prod behave similarly. Convention:
		// - Event rows start with a leading token like `__new_address__`.
		// - Mapping rows: `__addr_cred__`, addr_hash, cred_hash
		// - All rows may include a trailing `file_id:<sha256>` field which we'll extract into the
		//   `file_id` column.
		let dataset = self.dataset.as_deref();

		let mut event_type: Option<String> = None;
		let mut address_hash: Option<String> = None;
		let mut credential_hash: Option<String> = None;
		let mut row_hash: Option<String> = None;
		let mut file_id: Option<String> = None;
		let mut source_file: Option<String> = None;

		// copy fields but remove reserved keys like file_id and derive typed cols
		let mut remaining: Vec<serde_json::Value> = Vec::new();
		for (i, f) in row.iter().enumerate() {
			if f.starts_with("file_id:") {
				file_id = Some(f[8..].to_string());
				continue;
			}
			if f.starts_with("source_file:") {
				source_file = Some(f[12..].to_string());
				continue;
			}
			if f.starts_with("row_hash:") {
				row_hash = Some(f[9..].to_string());
				continue;
			}
			if i == 0 && f.starts_with("__") {
				event_type = Some(f.clone());
				// derive common typed columns
				if f == "__address_hash__" && row.len() > 1 {
					address_hash = Some(row[1].clone());
				} else if f == "__credential_hash__" && row.len() > 1 {
					credential_hash = Some(row[1].clone());
				} else if f == "__addr_cred__" && row.len() > 2 {
					address_hash = Some(row[1].clone());
					credential_hash = Some(row[2].clone());
				} else if f == "__known_address_new_credential__" && row.len() > 2 {
					address_hash = Some(row[1].clone());
					credential_hash = Some(row[2].clone());
				} else if f == "__new_address__" && row.len() > 1 {
					address_hash = Some(row[1].clone());
				}
				// keep the event token in the fields as well
				remaining.push(serde_json::Value::String(f.clone()));
				continue;
			}
			remaining.push(serde_json::Value::String(f.clone()));
		}

		let fields = serde_json::Value::Array(remaining);
		let fields_text =
			serde_json::to_string(&fields).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

		let stmt = self
			.client
			.prepare(
				"INSERT INTO normalized_rows (dataset, event_type, address_hash, credential_hash, \
				 row_hash, file_id, source_file, fields) VALUES ($1, $2, $3, $4, $5, $6, $7, \
				 $8::jsonb)",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		self.client
			.execute(
				&stmt,
				&[
					&dataset,
					&event_type,
					&address_hash,
					&credential_hash,
					&row_hash,
					&file_id,
					&source_file,
					&fields_text,
				],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(())
	}

	fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool> {
		// Check dedicated columns first for speed, then fall back to fields text
		let stmt = self
			.client
			.prepare(
				"SELECT EXISTS (SELECT 1 FROM normalized_rows WHERE credential_hash = $1 OR \
				 address_hash = $1)",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let row = self
			.client
			.query_one(&stmt, &[&hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: bool = row.get(0);
		if exists {
			return Ok(true);
		}
		// fallback: search JSON text
		let pattern = format!("%{}%", hash);
		let stmt2 = self
			.client
			.prepare("SELECT EXISTS (SELECT 1 FROM normalized_rows WHERE fields::text LIKE $1)")
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let row2 = self
			.client
			.query_one(&stmt2, &[&pattern])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists2: bool = row2.get(0);
		Ok(exists2)
	}

	fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool> {
		let stmt = self
			.client
			.prepare("SELECT EXISTS (SELECT 1 FROM normalized_rows WHERE address_hash = $1)")
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let row = self
			.client
			.query_one(&stmt, &[&addr_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: bool = row.get(0);
		Ok(exists)
	}

	fn address_has_credential(
		&mut self,
		addr_hash: &str,
		cred_hash: &str,
	) -> std::io::Result<bool> {
		let stmt = self
			.client
			.prepare(
				"SELECT EXISTS (SELECT 1 FROM normalized_rows WHERE address_hash = $1 AND \
				 credential_hash = $2)",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let row = self
			.client
			.query_one(&stmt, &[&addr_hash, &cred_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: bool = row.get(0);
		Ok(exists)
	}

	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()> {
		let dataset = self.dataset.as_deref();
		let fields = serde_json::Value::Array(vec![
			serde_json::Value::String("__addr_cred__".to_string()),
			serde_json::Value::String(addr_hash.to_string()),
			serde_json::Value::String(cred_hash.to_string()),
		]);
		let fields_text =
			serde_json::to_string(&fields).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let stmt = self
			.client
			.prepare(
				"INSERT INTO normalized_rows (dataset, event_type, address_hash, credential_hash, \
				 file_id, fields) VALUES ($1, $2, $3, $4, $5, $6::jsonb)",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		// No explicit file_id available here; leave it NULL
		self.client
			.execute(
				&stmt,
				&[
					&dataset,
					&"__addr_cred__",
					&addr_hash,
					&cred_hash,
					&Option::<String>::None,
					&fields_text,
				],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(())
	}

	fn insert_canonical_address(
		&mut self,
		canonical_hash: &str,
		address_text: &str,
		normalized_form: &str,
	) -> std::io::Result<bool> {
		let stmt = self
			.client
			.prepare(
				"INSERT INTO canonical_addresses (canonical_hash, address_text, normalized_form) \
				 VALUES ($1, $2, $3) ON CONFLICT (canonical_hash) DO NOTHING",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.execute(&stmt, &[&canonical_hash, &address_text, &normalized_form])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows > 0)
	}

	fn insert_address_alternate(
		&mut self,
		canonical_hash: &str,
		alternate_hash: &str,
		alternate_form: &str,
	) -> std::io::Result<bool> {
		let stmt = self
			.client
			.prepare(
				"INSERT INTO address_alternates (canonical_hash, alternate_hash, alternate_form) \
				 VALUES ($1, $2, $3) ON CONFLICT (canonical_hash, alternate_hash) DO NOTHING",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.execute(&stmt, &[&canonical_hash, &alternate_hash, &alternate_form])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows > 0)
	}

	fn lookup_canonical_by_alternate(
		&mut self,
		alternate_hash: &str,
	) -> std::io::Result<Option<String>> {
		let stmt = self
			.client
			.prepare(
				"SELECT canonical_hash FROM address_alternates WHERE alternate_hash = $1 LIMIT 1",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		match self
			.client
			.query_opt(&stmt, &[&alternate_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
		{
			Some(row) => Ok(Some(row.get(0))),
			None => Ok(None),
		}
	}

	fn insert_address_credential_canonical(
		&mut self,
		canonical_hash: &str,
		credential_hash: &str,
	) -> std::io::Result<bool> {
		let stmt = self
			.client
			.prepare(
				"INSERT INTO address_credentials (canonical_hash, credential_hash) VALUES ($1, \
				 $2) ON CONFLICT (canonical_hash, credential_hash) DO UPDATE SET occurrence_count \
				 = occurrence_count + 1, last_seen_at = now()",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.execute(&stmt, &[&canonical_hash, &credential_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows > 0)
	}

	fn get_credentials_for_address(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<String>> {
		let stmt = self
			.client
			.prepare(
				"SELECT credential_hash FROM address_credentials WHERE canonical_hash = $1 ORDER \
				 BY first_seen_at DESC",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.query(&stmt, &[&canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
	}

	fn record_address_cooccurrence(
		&mut self,
		canonical_hash_1: &str,
		canonical_hash_2: &str,
	) -> std::io::Result<bool> {
		// Ensure canonical order: smaller hash first
		let (h1, h2) = if canonical_hash_1 < canonical_hash_2 {
			(canonical_hash_1, canonical_hash_2)
		} else {
			(canonical_hash_2, canonical_hash_1)
		};

		let stmt = self
			.client
			.prepare(
				"INSERT INTO address_cooccurrence (canonical_hash_1, canonical_hash_2) VALUES \
				 ($1, $2) ON CONFLICT (canonical_hash_1, canonical_hash_2) DO UPDATE SET \
				 cooccurrence_count = cooccurrence_count + 1, last_seen_at = now()",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.execute(&stmt, &[&h1, &h2])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows > 0)
	}

	fn get_address_neighbors(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, i32)>> {
		// Get all neighbors from both directions (since the graph is undirected)
		let stmt = self
			.client
			.prepare(
				"SELECT CASE WHEN canonical_hash_1 = $1 THEN canonical_hash_2 ELSE \
				 canonical_hash_1 END as neighbor, cooccurrence_count FROM address_cooccurrence \
				 WHERE canonical_hash_1 = $1 OR canonical_hash_2 = $1 ORDER BY cooccurrence_count \
				 DESC",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.query(&stmt, &[&canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows
			.iter()
			.map(|r| (r.get::<_, String>(0), r.get::<_, i32>(1)))
			.collect())
	}

	fn update_address_embedding(
		&mut self,
		canonical_hash: &str,
		embedding: &[f32],
	) -> std::io::Result<()> {
		// Convert embedding slice to PostgreSQL vector format: [val1,val2,...]
		let vec_str = format!(
			"[{}]",
			embedding
				.iter()
				.map(|v| v.to_string())
				.collect::<Vec<_>>()
				.join(",")
		);
		let stmt = self
			.client
			.prepare(
				"UPDATE canonical_addresses SET embedding = $1::vector WHERE canonical_hash = $2",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		self.client
			.execute(&stmt, &[&vec_str, &canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(())
	}

	fn find_similar_addresses(
		&mut self,
		embedding: &[f32],
		limit: usize,
		similarity_threshold: f32,
	) -> std::io::Result<Vec<(String, f32)>> {
		// Use pgvector cosine similarity operator (<->)
		// The operator returns distance; similarity = 1 - distance for cosine
		let vec_str = format!(
			"[{}]",
			embedding
				.iter()
				.map(|v| v.to_string())
				.collect::<Vec<_>>()
				.join(",")
		);

		let stmt = self
			.client
			.prepare(
				"SELECT canonical_hash, (1.0 - (embedding <-> $1::vector)) as similarity FROM \
				 canonical_addresses WHERE embedding IS NOT NULL AND (1.0 - (embedding <-> \
				 $1::vector)) >= $2 ORDER BY similarity DESC LIMIT $3",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

		let rows = self
			.client
			.query(&stmt, &[&vec_str, &similarity_threshold, &(limit as i64)])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows
			.iter()
			.map(|r| (r.get::<_, String>(0), r.get::<_, f32>(1)))
			.collect())
	}

	fn find_duplicate_address(
		&mut self,
		canonical_hash: &str,
		embedding: Option<&[f32]>,
		similarity_threshold: f32,
	) -> std::io::Result<Option<String>> {
		// First, check for exact canonical hash match (dedup by canonical form)
		let stmt = self
			.client
			.prepare("SELECT canonical_hash FROM canonical_addresses WHERE canonical_hash = $1")
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

		if let Some(row) = self
			.client
			.query_opt(&stmt, &[&canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
		{
			return Ok(Some(row.get(0)));
		}

		// If embedding provided, check for vector similarity duplicates
		if let Some(emb) = embedding {
			let vec_str = format!(
				"[{}]",
				emb.iter()
					.map(|v| v.to_string())
					.collect::<Vec<_>>()
					.join(",")
			);

			let stmt = self
				.client
				.prepare(
					"SELECT canonical_hash FROM canonical_addresses WHERE embedding IS NOT NULL \
					 AND (1.0 - (embedding <-> $1::vector)) >= $2 ORDER BY (1.0 - (embedding <-> \
					 $1::vector)) DESC LIMIT 1",
				)
				.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

			if let Some(row) = self
				.client
				.query_opt(&stmt, &[&vec_str, &similarity_threshold])
				.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
			{
				return Ok(Some(row.get(0)));
			}
		}

		Ok(None)
	}

	fn insert_address_breach(
		&mut self,
		canonical_hash: &str,
		breach_name: &str,
		breach_title: Option<&str>,
		breach_domain: Option<&str>,
		breach_date: Option<&str>,
		pwn_count: Option<i32>,
		description: Option<&str>,
		is_verified: bool,
		is_fabricated: bool,
		is_sensitive: bool,
		is_retired: bool,
	) -> std::io::Result<bool> {
		let stmt = self
			.client
			.prepare(
				"INSERT INTO address_breaches (canonical_hash, breach_name, breach_title, \
				 breach_domain, breach_date, pwn_count, description, is_verified, is_fabricated, \
				 is_sensitive, is_retired) VALUES ($1, $2, $3, $4, $5::date, $6, $7, $8, $9, $10, \
				 $11) ON CONFLICT (canonical_hash, breach_name) DO NOTHING",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.execute(
				&stmt,
				&[
					&canonical_hash,
					&breach_name,
					&breach_title,
					&breach_domain,
					&breach_date,
					&pwn_count,
					&description,
					&is_verified,
					&is_fabricated,
					&is_sensitive,
					&is_retired,
				],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows > 0)
	}

	fn get_breaches_for_address(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, i32)>> {
		let stmt = self
			.client
			.prepare(
				"SELECT breach_name, pwn_count FROM address_breaches WHERE canonical_hash = $1 \
				 ORDER BY pwn_count DESC",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let rows = self
			.client
			.query(&stmt, &[&canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(rows
			.iter()
			.map(|r| (r.get::<_, String>(0), r.get::<_, i32>(1)))
			.collect())
	}

	fn get_breach_count(&mut self, canonical_hash: &str) -> std::io::Result<i32> {
		let stmt = self
			.client
			.prepare("SELECT COUNT(*) as count FROM address_breaches WHERE canonical_hash = $1")
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let row = self
			.client
			.query_one(&stmt, &[&canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(row.get::<_, i64>(0) as i32)
	}

	fn get_total_pwn_count(&mut self, canonical_hash: &str) -> std::io::Result<i32> {
		let stmt = self
			.client
			.prepare(
				"SELECT COALESCE(SUM(pwn_count), 0) as total FROM address_breaches WHERE \
				 canonical_hash = $1",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let row = self
			.client
			.query_one(&stmt, &[&canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(row.get::<_, i64>(0) as i32)
	}
}

#[cfg(test)]
mod tests {
	use std::{env, fs};

	use super::*;

	#[test]
	fn fs_storage_writes_and_reads() {
		let mut path = env::temp_dir();
		let ts = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_nanos();
		path.push(format!("dumptruck_storage_test_{}.csv", ts));

		// ensure file removed at end
		let _ = fs::remove_file(&path);

		let mut store = FsStorage::new(path.clone()).expect("create storage");
		let row = vec!["a".to_string(), "b,c".to_string(), "d".to_string()];
		store.store_row(&row).expect("store row");

		let content = FsStorage::read_all(&path).expect("read back");
		assert!(content.contains("a,\"b,c\",d"));

		// test contains_hash
		let sha = "deadbeef";
		let _ = fs::write(&path, format!("{}\n", sha));
		let mut s = FsStorage::new(path.clone()).expect("open");
		assert!(s.contains_hash(sha).expect("contains"));

		// cleanup
		let _ = fs::remove_file(&path);
	}
}

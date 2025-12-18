//! Database statistics and analytics.

use postgres::Client;
use serde::{Deserialize, Serialize};

/// Database statistics structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseStats {
	/// Total number of canonical addresses
	pub total_addresses: i64,
	/// Total number of address variants
	pub total_variants: i64,
	/// Total number of unique credentials
	pub total_credentials: i64,
	/// Total number of address-credential mappings
	pub total_address_credentials: i64,
	/// Total number of co-occurrence edges
	pub total_cooccurrences: i64,
	/// Total number of breach records (from HIBP)
	pub total_breaches: i64,
	/// Total number of normalized rows ingested
	pub total_rows: i64,
	/// Number of unique datasets
	pub unique_datasets: i64,
	/// Average credentials per address
	pub avg_credentials_per_address: f64,
	/// Average variants per address
	pub avg_variants_per_address: f64,
	/// Most common breach (if any)
	pub top_breach: Option<(String, i64)>,
	/// Address with most credentials
	pub address_most_credentials: Option<(String, i64)>,
}

impl DatabaseStats {
	/// Connect to database and retrieve statistics
	pub fn from_postgres(conn_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let mut client = Client::connect(conn_str, postgres::NoTls)?;

		// Total canonical addresses
		let row = client.query_one("SELECT COUNT(*) as count FROM canonical_addresses", &[])?;
		let total_addresses: i64 = row.get(0);

		// Total variants
		let row = client.query_one("SELECT COUNT(*) as count FROM address_alternates", &[])?;
		let total_variants: i64 = row.get(0);

		// Total unique credentials
		let row = client.query_one(
			"SELECT COUNT(DISTINCT credential_hash) as count FROM address_credentials",
			&[],
		)?;
		let total_credentials: i64 = row.get(0);

		// Total address-credential mappings
		let row = client.query_one("SELECT COUNT(*) as count FROM address_credentials", &[])?;
		let total_address_credentials: i64 = row.get(0);

		// Total co-occurrences
		let row = client.query_one("SELECT COUNT(*) as count FROM address_cooccurrence", &[])?;
		let total_cooccurrences: i64 = row.get(0);

		// Total breaches
		let row = client.query_one("SELECT COUNT(*) as count FROM address_breaches", &[])?;
		let total_breaches: i64 = row.get(0);

		// Total normalized rows
		let row = client.query_one("SELECT COUNT(*) as count FROM normalized_rows", &[])?;
		let total_rows: i64 = row.get(0);

		// Unique datasets
		let row = client.query_one(
			"SELECT COUNT(DISTINCT dataset) as count FROM normalized_rows",
			&[],
		)?;
		let unique_datasets: i64 = row.get(0);

		// Average credentials per address
		let avg_credentials_per_address = if total_addresses > 0 {
			total_address_credentials as f64 / total_addresses as f64
		} else {
			0.0
		};

		// Average variants per address
		let avg_variants_per_address = if total_addresses > 0 {
			total_variants as f64 / total_addresses as f64
		} else {
			0.0
		};

		// Most common breach
		let rows = client.query(
			"SELECT breach_name, COUNT(*) as count FROM address_breaches \
			 GROUP BY breach_name ORDER BY count DESC LIMIT 1",
			&[],
		)?;
		let top_breach = if !rows.is_empty() {
			let row = &rows[0];
			Some((row.get::<_, String>(0), row.get::<_, i64>(1)))
		} else {
			None
		};

		// Address with most credentials
		let rows = client.query(
			"SELECT canonical_hash, COUNT(*) as count FROM address_credentials \
			 GROUP BY canonical_hash ORDER BY count DESC LIMIT 1",
			&[],
		)?;
		let address_most_credentials = if !rows.is_empty() {
			let row = &rows[0];
			Some((row.get::<_, String>(0), row.get::<_, i64>(1)))
		} else {
			None
		};

		Ok(Self {
			total_addresses,
			total_variants,
			total_credentials,
			total_address_credentials,
			total_cooccurrences,
			total_breaches,
			total_rows,
			unique_datasets,
			avg_credentials_per_address,
			avg_variants_per_address,
			top_breach,
			address_most_credentials,
		})
	}

	/// Format statistics as human-readable text
	pub fn format_text(&self, detailed: bool) -> String {
		let mut output = String::new();

		output.push_str("=== Dumptruck Database Statistics ===\n\n");

		output.push_str(&format!(
			"Canonical Addresses:           {}\n",
			self.total_addresses
		));
		output.push_str(&format!(
			"Address Variants:              {}\n",
			self.total_variants
		));
		output.push_str(&format!(
			"Unique Credentials:            {}\n",
			self.total_credentials
		));
		output.push_str(&format!(
			"Address-Credential Mappings:   {}\n",
			self.total_address_credentials
		));
		output.push_str(&format!(
			"Co-occurrence Edges:           {}\n",
			self.total_cooccurrences
		));
		output.push_str(&format!(
			"Breach Records (HIBP):         {}\n",
			self.total_breaches
		));
		output.push_str(&format!(
			"Normalized Rows Ingested:      {}\n",
			self.total_rows
		));
		output.push_str(&format!(
			"Unique Datasets:               {}\n\n",
			self.unique_datasets
		));

		output.push_str(&format!(
			"Avg Credentials per Address:   {:.2}\n",
			self.avg_credentials_per_address
		));
		output.push_str(&format!(
			"Avg Variants per Address:      {:.2}\n\n",
			self.avg_variants_per_address
		));

		if let Some((breach_name, count)) = &self.top_breach {
			output.push_str(&format!(
				"Top Breach:                    {} ({})\n",
				breach_name, count
			));
		}

		if let Some((addr_hash, count)) = &self.address_most_credentials {
			output.push_str(&format!(
				"Most Exposed Address:          {} ({})\n",
				addr_hash, count
			));
		}

		if detailed {
			output.push_str("\n=== Detailed Analysis ===\n\n");

			let percentage_variants = if self.total_addresses > 0 {
				(self.total_variants as f64 / self.total_addresses as f64) * 100.0
			} else {
				0.0
			};

			output.push_str(&format!(
				"Variant Coverage:              {:.1}% (variants per address)\n",
				percentage_variants
			));

			if self.total_rows > 0 {
				let dedup_rate =
					(1.0 - (self.total_addresses as f64 / self.total_rows as f64)) * 100.0;
				output.push_str(&format!(
					"Deduplication Rate:            {:.1}%\n",
					dedup_rate
				));
			}

			if self.total_address_credentials > 0 {
				let avg_breach_count = if self.total_addresses > 0 {
					self.total_breaches as f64 / self.total_addresses as f64
				} else {
					0.0
				};
				output.push_str(&format!(
					"Avg Breaches per Address:      {:.2}\n",
					avg_breach_count
				));
			}
		}

		output.push_str("\n");
		output
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_format_text_basic() {
		let stats = DatabaseStats {
			total_addresses: 100,
			total_variants: 50,
			total_credentials: 200,
			total_address_credentials: 500,
			total_cooccurrences: 30,
			total_breaches: 10,
			total_rows: 1000,
			unique_datasets: 3,
			avg_credentials_per_address: 5.0,
			avg_variants_per_address: 0.5,
			top_breach: Some(("LinkedIn".to_string(), 75)),
			address_most_credentials: Some(("abc123".to_string(), 50)),
		};

		let text = stats.format_text(false);
		assert!(text.contains("Canonical Addresses:           100"));
		assert!(text.contains("Unique Credentials:            200"));
		assert!(text.contains("Top Breach:                    LinkedIn (75)"));
	}

	#[test]
	fn test_format_text_detailed() {
		let stats = DatabaseStats {
			total_addresses: 1000,
			total_variants: 200,
			total_credentials: 5000,
			total_address_credentials: 10000,
			total_cooccurrences: 500,
			total_breaches: 50,
			total_rows: 20000,
			unique_datasets: 10,
			avg_credentials_per_address: 10.0,
			avg_variants_per_address: 0.2,
			top_breach: Some(("Twitter".to_string(), 3000)),
			address_most_credentials: Some(("hash789".to_string(), 200)),
		};

		let text = stats.format_text(true);
		assert!(text.contains("=== Detailed Analysis ==="));
		assert!(text.contains("Variant Coverage:"));
		assert!(text.contains("Deduplication Rate:"));
	}

	#[test]
	fn test_average_calculations() {
		let stats = DatabaseStats {
			total_addresses: 100,
			total_variants: 50,
			total_credentials: 200,
			total_address_credentials: 500,
			total_cooccurrences: 30,
			total_breaches: 10,
			total_rows: 500,
			unique_datasets: 2,
			avg_credentials_per_address: 5.0,
			avg_variants_per_address: 0.5,
			top_breach: None,
			address_most_credentials: None,
		};

		assert_eq!(stats.avg_credentials_per_address, 5.0);
		assert_eq!(stats.avg_variants_per_address, 0.5);
	}

	#[test]
	fn test_format_text_empty_database() {
		let stats = DatabaseStats {
			total_addresses: 0,
			total_variants: 0,
			total_credentials: 0,
			total_address_credentials: 0,
			total_cooccurrences: 0,
			total_breaches: 0,
			total_rows: 0,
			unique_datasets: 0,
			avg_credentials_per_address: 0.0,
			avg_variants_per_address: 0.0,
			top_breach: None,
			address_most_credentials: None,
		};

		let text = stats.format_text(false);
		assert!(text.contains("Canonical Addresses:           0"));
		assert!(text.contains("Dumptruck Database Statistics"));
	}
}

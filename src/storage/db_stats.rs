//! Database statistics and analytics.

use rusqlite::Connection;
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
	/// Connect to SQLite database and retrieve statistics
	pub fn from_sqlite(conn: &Connection) -> Result<Self, Box<dyn std::error::Error>> {
		// Total canonical addresses
		let total_addresses: i64 = conn.query_row(
			"SELECT COUNT(*) as count FROM canonical_addresses",
			[],
			|row| row.get(0),
		)?;

		// Total variants
		let total_variants: i64 = conn.query_row(
			"SELECT COUNT(*) as count FROM address_alternates",
			[],
			|row| row.get(0),
		)?;

		// Total unique credentials
		let total_credentials: i64 = conn.query_row(
			"SELECT COUNT(DISTINCT credential_hash) as count FROM address_credentials",
			[],
			|row| row.get(0),
		)?;

		// Total address-credential mappings
		let total_address_credentials: i64 = conn.query_row(
			"SELECT COUNT(*) as count FROM address_credentials",
			[],
			|row| row.get(0),
		)?;

		// Total co-occurrences
		let total_cooccurrences: i64 = conn.query_row(
			"SELECT COUNT(*) as count FROM address_cooccurrence",
			[],
			|row| row.get(0),
		)?;

		// Total breaches
		let total_breaches: i64 = conn.query_row(
			"SELECT COUNT(*) as count FROM address_breaches",
			[],
			|row| row.get(0),
		)?;

		// Total normalized rows
		let total_rows: i64 =
			conn.query_row("SELECT COUNT(*) as count FROM normalized_rows", [], |row| {
				row.get(0)
			})?;

		// Unique datasets
		let unique_datasets: i64 = conn.query_row(
			"SELECT COUNT(DISTINCT dataset) as count FROM normalized_rows",
			[],
			|row| row.get(0),
		)?;

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
		let mut stmt = conn.prepare(
			"SELECT breach_name, COUNT(*) as count FROM address_breaches GROUP BY breach_name \
			 ORDER BY count DESC LIMIT 1",
		)?;

		let top_breach = stmt
			.query_row([], |row| {
				Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
			})
			.ok();

		// Address with most credentials (using HAVING to filter results)
		let mut stmt = conn.prepare(
			"SELECT canonical_hash, COUNT(*) as count FROM address_credentials GROUP BY \
			 canonical_hash ORDER BY count DESC LIMIT 1",
		)?;

		let address_most_credentials = stmt
			.query_row([], |row| {
				Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
			})
			.ok();

		Ok(DatabaseStats {
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

	/// Alternative constructor that takes a database path string
	pub fn from_db_path(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let conn = Connection::open(db_path)?;
		// Initialize schema if it doesn't exist (for new databases)
		crate::storage::create_schema(&conn)?;
		Self::from_sqlite(&conn)
	}

	/// Format statistics as human-readable text
	pub fn format_text(&self, detailed: bool) -> String {
		let mut output = String::new();
		output.push_str("=== Database Statistics ===\n");
		output.push_str(&format!("Total Addresses: {}\n", self.total_addresses));
		output.push_str(&format!("Total Variants: {}\n", self.total_variants));
		output.push_str(&format!("Total Credentials: {}\n", self.total_credentials));
		output.push_str(&format!(
			"Total Address-Credential Mappings: {}\n",
			self.total_address_credentials
		));
		output.push_str(&format!(
			"Total Co-occurrences: {}\n",
			self.total_cooccurrences
		));
		output.push_str(&format!("Total Breaches: {}\n", self.total_breaches));
		output.push_str(&format!("Total Rows Ingested: {}\n", self.total_rows));
		output.push_str(&format!("Unique Datasets: {}\n", self.unique_datasets));
		output.push_str(&format!(
			"Avg Credentials per Address: {:.2}\n",
			self.avg_credentials_per_address
		));
		output.push_str(&format!(
			"Avg Variants per Address: {:.2}\n",
			self.avg_variants_per_address
		));

		if detailed {
			if let Some((breach_name, count)) = &self.top_breach {
				output.push_str(&format!("Top Breach: {} ({})\n", breach_name, count));
			}
			if let Some((addr, count)) = &self.address_most_credentials {
				output.push_str(&format!(
					"Address with Most Credentials: {} ({})\n",
					addr, count
				));
			}
		}

		output
	}
}

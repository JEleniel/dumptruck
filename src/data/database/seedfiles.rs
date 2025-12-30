use std::sync::Arc;

use rusqlite::params;
use tokio::sync::Mutex;

use crate::data::{
	DatabaseError,
	database::{migrationtrait::MigrationTrait, signedconnection::SignedConnection},
};

pub struct SeedFiles {
	conn: Arc<Mutex<SignedConnection>>,
}

impl MigrationTrait for SeedFiles {
	fn create(conn: &SignedConnection) -> Result<(), DatabaseError> {
		conn.execute_batch(
			"CREATE TABLE IF NOT EXISTS seed_files (
				id INT PRIMARY KEY AUTOINCREMENT,
				file_name TEXT,
				signature TEXT UNIQUE
			);",
		)?;
		Ok(())
	}

	fn upgrade(conn: &SignedConnection) -> Result<(), DatabaseError> {
		Self::create(conn)
	}

	fn downgrade(conn: &SignedConnection) -> Result<(), DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS seed_files;")?;
		Ok(())
	}
}

impl SeedFiles {
	pub fn new(conn: Arc<Mutex<SignedConnection>>) -> Self {
		Self { conn }
	}

	pub async fn is_known(&self, signature: &str) -> rusqlite::Result<bool, DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT COUNT(1) FROM seed_files WHERE signature = ?1;")?;
		let count: i32 = stmt.query_row(params![signature], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, file_name: &str, signature: &str) -> rusqlite::Result<()> {
		self.conn.lock().await.execute(
			"INSERT OR IGNORE INTO seed_files (file_name, signature) VALUES (?1, ?2);",
			params![file_name, signature],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> rusqlite::Result<Vec<(String, String)>, DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT file_name, signature FROM seed_files;")?;
		let sf_iter = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?;

		let mut sf_entries = Vec::new();
		for sf_result in sf_iter {
			sf_entries.push(sf_result?);
		}
		Ok(sf_entries)
	}

	pub async fn write_all_to_file(&self, output_path: &std::path::Path) -> rusqlite::Result<()> {
		let seed_files = self.get_all().await?;
		let mut file = std::fs::File::create(output_path)?;

		for (file_name, signature) in seed_files {
			use std::io::Write;
			writeln!(file, "{},{}", file_name, signature)?;
		}
		Ok(())
	}
}

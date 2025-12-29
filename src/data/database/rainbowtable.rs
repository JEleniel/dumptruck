use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::data::{DatabaseError, database::migrationtrait::MigrationTrait};

pub struct RainbowTable {
	conn: Arc<Mutex<Connection>>,
}

impl MigrationTrait for RainbowTable {
	fn create(conn: &rusqlite::Connection) -> Result<(), crate::data::DatabaseError> {
		conn.execute_batch(
			"CREATE TABLE IF NOT EXISTS rainbow_table (
				id INT PRIMARY KEY AUTOINCREMENT,
				hash_type TEXT,
				hash TEXT,
				UNIQUE (hash_type, hash)
			);",
		)?;
		Ok(())
	}

	fn upgrade(conn: &rusqlite::Connection) -> Result<(), DatabaseError> {
		Self::create(conn)
	}

	fn downgrade(conn: &rusqlite::Connection) -> Result<(), DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS rainbow_table;")?;
		Ok(())
	}
}

impl RainbowTable {
	pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
		Self { conn }
	}

	pub async fn is_known(&self, hash: &str) -> Result<String, DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT hash_type FROM rainbow_table WHERE hash = ?1;")?;
		let hash_type: String = stmt.query_row(rusqlite::params![hash], |row| row.get(0))?;
		Ok(hash_type)
	}

	pub async fn add(&self, hash_type: &str, hash: &str) -> Result<(), DatabaseError> {
		self.conn.lock().await.execute(
			"INSERT OR IGNORE INTO rainbow_table (type, hash) VALUES (?1, ?2);",
			rusqlite::params![hash_type, hash],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<(String, String)>, DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT hash_type, hash FROM rainbow_table;")?;
		let rt_iter = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

		let mut rt_entries = Vec::new();
		for rt_result in rt_iter {
			rt_entries.push(rt_result?);
		}
		Ok(rt_entries)
	}
}

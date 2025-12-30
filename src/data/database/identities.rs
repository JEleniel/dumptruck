use std::sync::Arc;

use rusqlite::params;
use tokio::sync::Mutex;

use crate::data::database::{migrationtrait::MigrationTrait, signedconnection::SignedConnection};

pub struct Identities {
	conn: Arc<Mutex<SignedConnection>>,
}

impl MigrationTrait for Identities {
	fn create(conn: &SignedConnection) -> Result<(), super::DatabaseError> {
		conn.execute_batch(
			"
			CREATE TABLE IF NOT EXISTS ids (
				id INTEGER PRIMARY KEY,
				value TEXT NOT NULL UNIQUE,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
			);",
		)?;
		Ok(())
	}

	fn upgrade(conn: &SignedConnection) -> Result<(), super::DatabaseError> {
		Self.create(conn)
	}

	fn downgrade(conn: &SignedConnection) -> Result<(), super::DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS ids;")?;
		Ok(())
	}
}

impl Identities {
	pub fn new(conn: Arc<Mutex<SignedConnection>>) -> Self {
		Self { conn }
	}

	pub async fn is_known(&self, value: &str) -> Result<bool, super::DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT 1 FROM ids WHERE value = ?1;")?;
		let count = stmt.query_row(params![value], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, value: &str) -> Result<(), super::DatabaseError> {
		self.conn.lock().await.execute(
			"INSERT OR IGNORE INTO ids (value) VALUES (?1);",
			params![value],
		)?;
		Ok(())
	}

	pub async fn seen(&self, value: &str) -> Result<(), super::DatabaseError> {
		self.conn.lock().await.execute(
			"UPDATE ids SET last_seen = CURRENT_TIMESTAMP WHERE value = ?1;",
			params![value],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<String>, super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare("SELECT value FROM ids;")?;
		let ids_iter = stmt.query_map(params![], |row| row.get(0))?;

		let mut ids = Vec::new();
		for id_result in ids_iter {
			ids.push(id_result?);
		}
		Ok(ids)
	}

	pub async fn write_all(&self, values: &[String]) -> Result<(), super::DatabaseError> {
		let tx = self.conn.lock().await.transaction()?;
		{
			let mut stmt = tx.prepare("INSERT OR IGNORE INTO ids (value) VALUES (?1);")?;
			for value in values {
				stmt.execute(params![value])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

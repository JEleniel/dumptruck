use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::data::database::migrationtrait::MigrationTrait;

pub struct Pii {
	conn: Arc<Mutex<Connection>>,
}

impl MigrationTrait for Pii {
	fn create(conn: &rusqlite::Connection) -> Result<(), super::DatabaseError> {
		conn.execute_batch(
			"
			CREATE TABLE IF NOT EXISTS pii (
				id INTEGER PRIMARY KEY,
				type TEXT NOT NULL,
				hash TEXT NOT NULL,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
				UNIQUE(type, hash)
			);
			CREATE INDEX IF NOT EXISTS idx_pii_type ON pii(type);
			CREATE INDEX IF NOT EXISTS idx_pii_hash ON pii(hash);
			",
		)
	}

	fn upgrade(conn: &rusqlite::Connection) -> Result<(), super::DatabaseError> {
		Self::create(conn)
	}

	fn downgrade(conn: &rusqlite::Connection) -> Result<(), super::DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS pii;")
	}
}

impl Pii {
	pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
		Self { conn }
	}

	pub async fn is_known(
		&self,
		pii_type: &str,
		pii_hash: &str,
	) -> Result<bool, super::DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT 1 FROM pii WHERE type = ?1 AND hash = ?2 LIMIT 1;")?;
		let mut rows = stmt.query(rusqlite::params![pii_type, pii_hash])?;
		Ok(rows.next()?.is_some())
	}

	pub async fn add_pii(
		&self,
		pii_type: &str,
		pii_hash: &str,
	) -> Result<(), super::DatabaseError> {
		self.conn.lock().await.execute(
			"
			INSERT OR IGNORE INTO pii (type, hash)
			VALUES (?1, ?2);
			",
			rusqlite::params![pii_type, pii_hash],
		)?;
		Ok(())
	}

	pub async fn seen(&self, pii_type: &str, pii_hash: &str) -> Result<(), super::DatabaseError> {
		self.conn.lock().await.execute(
			"
			UPDATE pii
			SET last_seen = CURRENT_TIMESTAMP
			WHERE type = ?1 AND hash = ?2;
			",
			rusqlite::params![pii_type, pii_hash],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<(String, String)>, super::DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT type, hash FROM pii;")?;
		let pii_iter = stmt.query_map([], |row| {
			let pii_type: String = row.get(0)?;
			let pii_hash: String = row.get(1)?;
			Ok((pii_type, pii_hash))
		})?;

		let mut pii_list = Vec::new();
		for pii_result in pii_iter {
			pii_list.push(pii_result?);
		}
		Ok(pii_list)
	}

	pub async fn write_all(
		&self,
		entries: &[(String, String)],
	) -> Result<(), super::DatabaseError> {
		let tx = self.conn.lock().await.transaction()?;
		{
			let mut stmt = tx.prepare("INSERT OR IGNORE INTO pii (type, hash) VALUES (?1, ?2);")?;
			for (pii_type, pii_hash) in entries {
				stmt.execute(rusqlite::params![pii_type, pii_hash])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::data::database::migrationtrait::MigrationTrait;

pub struct Credentials {
	conn: Arc<Mutex<Connection>>,
}

impl MigrationTrait for Credentials {
	fn create(conn: &rusqlite::Connection) -> Result<(), super::DatabaseError> {
		conn.execute_batch(
			"
			CREATE TABLE IF NOT EXISTS credentials (
				id INTEGER PRIMARY KEY,
				identity_id TEXT NOT NULL,
				hash TEXT NOT NULL,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
				UNIQUE (identity_id, hash)
			);
			",
		)
	}

	fn upgrade(conn: &rusqlite::Connection) -> Result<(), super::DatabaseError> {
		Self::create(conn)
	}

	fn downgrade(conn: &rusqlite::Connection) -> Result<(), super::DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS credentials;")
	}
}

impl Credentials {
	pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
		Self { conn }
	}

	pub async fn is_known(
		&self,
		identity_id: &str,
		hash: &str,
	) -> Result<bool, super::DatabaseError> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT 1 FROM credentials WHERE identity_id = ?1 AND hash = ?2;")?;
		let count = self
			.conn
			.lock()
			.await
			.query_row(stmt, [identity_id, hash], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, identity_id: &str, hash: &str) -> Result<(), super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare(
			"INSERT OR IGNORE INTO credentials (identity_id, hash, last_seen) VALUES (?1, ?2, CURRENT_TIMESTAMP);",
		)?;
		stmt.execute([identity_id, hash])?;
		Ok(())
	}

	pub async fn seen(&self, identity_id: &str, hash: &str) -> Result<(), super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare(
			"UPDATE credentials SET last_seen = CURRENT_TIMESTAMP WHERE identity_id = ?1 AND hash = ?2;",
		)?;
		stmt.execute([identity_id, hash])?;
		Ok(())
	}

	pub async fn get_all(
		&self,
	) -> Result<Vec<(String, String, String, String)>, super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare(
			"SELECT identity_id, hash, created_at, last_seen FROM credentials ORDER BY created_at DESC;",
		)?;
		let credential_iter = stmt.query_map([], |row| {
			Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
		})?;

		let mut credentials = Vec::new();
		for credential in credential_iter {
			credentials.push(credential?);
		}
		Ok(credentials)
	}

	pub async fn write_all(
		&self,
		credentials: &[(String, String)],
	) -> Result<(), super::DatabaseError> {
		let tx = self.conn.lock().await.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR IGNORE INTO credentials (identity_id, hash, last_seen) VALUES (?1, ?2, CURRENT_TIMESTAMP);",
			)?;
			for (identity_id, hash) in credentials {
				stmt.execute([identity_id, hash])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

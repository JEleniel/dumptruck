use std::sync::Arc;

use rusqlite::params;
use tokio::sync::Mutex;

use crate::data::database::{migrationtrait::MigrationTrait, signedconnection::SignedConnection};

pub struct Dumps {
	conn: Arc<Mutex<SignedConnection>>,
}

impl MigrationTrait for Dumps {
	fn create(conn: &SignedConnection) -> Result<(), super::DatabaseError> {
		conn.execute_batch(
			"
			CREATE TABLE dumps (
				id INTEGER PRIMARY KEY,
				file_name TEXT NOT NULL,
				breach_date TEXT,
				breach_target TEXT,
				hash TEXT NOT NULL UNIQUE,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
				UNIQUE (file_name, breach_date, breach_target, hash)
			);
			",
		)?;
		Ok(())
	}

	fn upgrade(conn: &SignedConnection) -> Result<(), super::DatabaseError> {
		Self::create(conn)
	}

	fn downgrade(conn: &SignedConnection) -> Result<(), super::DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS dumps;")?;
		Ok(())
	}
}

impl Dumps {
	pub fn new(conn: Arc<Mutex<SignedConnection>>) -> Self {
		Self { conn }
	}

	pub async fn is_known(
		&self,
		breach_date: Option<&str>,
		breach_target: Option<&str>,
		hash: &str,
	) -> Result<bool, super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare(
			"SELECT 1 FROM dumps WHERE breach_date IS ?1 AND breach_target IS ?2 AND hash = ?3;",
		)?;
		let count = stmt.query_row(params![breach_date, breach_target, hash], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(
		&self,
		file_name: &str,
		breach_date: Option<&str>,
		breach_target: Option<&str>,
		hash: &str,
	) -> Result<(), super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare(
			"INSERT OR IGNORE INTO dumps (file_name, breach_date, breach_target, hash, last_seen) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP);",
		)?;
		stmt.execute(params![file_name, breach_date, breach_target, hash])?;
		Ok(())
	}

	pub async fn seen(
		conn: &SignedConnection,
		breach_date: Option<&str>,
		breach_target: Option<&str>,
		hash: &str,
	) -> Result<(), super::DatabaseError> {
		let mut stmt = conn.prepare(
			"UPDATE dumps SET last_seen = CURRENT_TIMESTAMP WHERE breach_date IS ?1 AND breach_target IS ?2 AND hash = ?3;",
		)?;
		stmt.execute(params![breach_date, breach_target, hash])?;
		Ok(())
	}

	pub async fn clear(&self) -> Result<(), super::DatabaseError> {
		self.conn.lock().await.execute_batch("DELETE FROM dumps;")?;
		Ok(())
	}

	pub async fn get_all(
		&self,
	) -> Result<Vec<(String, Option<String>, Option<String>, String)>, super::DatabaseError> {
		let mut stmt = self.conn.lock().await.prepare(
			"SELECT file_name, breach_date, breach_target, hash FROM dumps ORDER BY created_at DESC;",
		)?;
		let dump_iter = stmt.query_map(params![], |row| {
			Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
		})?;

		let mut dumps = Vec::new();
		for dump in dump_iter {
			dumps.push(dump?);
		}
		Ok(dumps)
	}

	pub async fn write_all(
		&self,
		dumps: &[(String, Option<String>, Option<String>, String)],
	) -> Result<(), super::DatabaseError> {
		let tx = self.conn.lock().await.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR IGNORE INTO dumps (file_name, breach_date, breach_target, hash, last_seen) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP);",
			)?;
			for dump in dumps {
				stmt.execute(params![
					&dump.0,
					&dump.1 as &dyn rusqlite::ToSql,
					&dump.2 as &dyn rusqlite::ToSql,
					&dump.3,
				])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

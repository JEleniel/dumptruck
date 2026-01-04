use std::collections::HashMap;

use crate::database::{DatabaseError, migrationtrait::MigrationTrait};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct Identities {
	pool: Pool<SqliteConnectionManager>,
}

impl MigrationTrait for Identities {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

		conn.execute_batch(
			"
			CREATE TABLE IF NOT EXISTS identities (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				hash TEXT NOT NULL UNIQUE,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
			);",
		)?;
		Ok(())
	}

	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		Self::create(pool)
	}

	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;
		conn.execute_batch("DROP TABLE IF EXISTS identities;")?;
		Ok(())
	}
}

impl Identities {
	pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
		Self { pool }
	}

	pub async fn is_known(&self, value: &str) -> Result<bool, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare("SELECT 1 FROM identities WHERE hash = ?1;")?;
		let count: u32 = stmt.query_row(params![value], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;

		conn.execute(
			"INSERT OR IGNORE INTO identities (hash) VALUES (?1);",
			params![hash],
		)?;
		Ok(())
	}

	pub async fn seen(&self, hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;

		conn.execute(
			"UPDATE identities SET last_seen = CURRENT_TIMESTAMP WHERE hash = ?1;",
			params![hash],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<Identity>, DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt = conn.prepare("SELECT id, hash, created_at, last_seen FROM identities;")?;
		let ids_iter = stmt.query_map(params![], |row| {
			let created_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;
			let last_seen = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;

			Ok(Identity {
				id: row.get(0)?,
				hash: row.get(1)?,
				created_at,
				last_seen,
			})
		})?;

		let mut ids = Vec::new();
		for id_result in ids_iter {
			ids.push(id_result?);
		}
		Ok(ids)
	}

	pub async fn write_all(&self, values: &[Identity]) -> Result<HashMap<i32, i32>, DatabaseError> {
		let mut conn = self.pool.get()?;

		let tx = conn.transaction()?;
		let mut ids = HashMap::new();

		for (_, value) in values.iter().enumerate() {
			{
				let mut stmt = tx.prepare(
					"INSERT OR IGNORE INTO identities (hash, created_at, last_seen) VALUES (?1, ?2, ?3);",
				)?;
				stmt.execute(params![
					value.hash,
					value.created_at.to_rfc3339(),
					value.last_seen.to_rfc3339()
				])?;
			}
			let id = tx.last_insert_rowid() as i32;
			ids.insert(value.id, id);
		}

		tx.commit()?;
		Ok(ids)
	}
}

pub struct Identity {
	pub id: i32,
	pub hash: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub last_seen: chrono::DateTime<chrono::Utc>,
}

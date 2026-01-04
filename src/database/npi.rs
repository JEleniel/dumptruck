use crate::database::{DatabaseError, migrationtrait::MigrationTrait};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct NPI {
	pool: Pool<SqliteConnectionManager>,
}

impl MigrationTrait for NPI {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

		conn.execute_batch(
			"CREATE TABLE IF NOT EXISTS npi (
				id INTEGER PRIMARY KEY,
				type TEXT NOT NULL,
				hash TEXT NOT NULL,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
				UNIQUE(type, hash)
			);",
		)?;
		Ok(())
	}

	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		Self::create(pool)
	}

	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;
		conn.execute_batch("DROP TABLE IF EXISTS npi;")?;
		Ok(())
	}
}

impl NPI {
	pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
		Self { pool }
	}

	pub async fn is_known(&self, npi_type: &str, npi_hash: &str) -> Result<bool, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare("SELECT 1 FROM npi WHERE type = ?1 AND hash = ?2;")?;
		let count: u32 = stmt.query_row(params![npi_type, npi_hash], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, npi_type: &str, npi_hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;
		conn.execute(
			"
			INSERT OR IGNORE INTO npi (type, hash)
			VALUES (?1, ?2);
			",
			params![npi_type, npi_hash],
		)?;
		Ok(())
	}

	pub async fn seen(&self, npi_type: &str, npi_hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;

		conn.execute(
			"
			UPDATE npi
			SET last_seen = CURRENT_TIMESTAMP
			WHERE type = ?1 AND hash = ?2;
			",
			params![npi_type, npi_hash],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<NPIEntry>, DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt = conn.prepare("SELECT id, type, hash, created_at, last_seen FROM npi;")?;
		let npi_iter = stmt.query_map(params![], |row| {
			let created_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;
			let last_seen = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;

			Ok(NPIEntry {
				id: row.get(0)?,
				npi_type: row.get(1)?,
				npi_hash: row.get(2)?,
				created_at,
				last_seen,
			})
		})?;

		let mut npi_list = Vec::new();
		for npi_result in npi_iter {
			npi_list.push(npi_result?);
		}
		Ok(npi_list)
	}

	pub async fn write_all(&self, entries: &[NPIEntry]) -> Result<(), DatabaseError> {
		let mut conn = self.pool.get()?;

		let tx = conn.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR IGNORE INTO npi (type, hash, created_at, last_seen) VALUES (?1, ?2, ?3, ?4);",
			)?;
			for entry in entries {
				stmt.execute(params![
					entry.npi_type,
					entry.npi_hash,
					entry.created_at.to_rfc3339(),
					entry.last_seen.to_rfc3339()
				])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

pub struct NPIEntry {
	pub id: i32,
	pub npi_type: String,
	pub npi_hash: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub last_seen: chrono::DateTime<chrono::Utc>,
}

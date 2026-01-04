use crate::database::{DatabaseError, migrationtrait::MigrationTrait};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct RainbowTable {
	pool: Pool<SqliteConnectionManager>,
}

impl MigrationTrait for RainbowTable {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

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

	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		Self::create(pool)
	}

	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

		conn.execute_batch("DROP TABLE IF EXISTS rainbow_table;")?;
		Ok(())
	}
}

impl RainbowTable {
	pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
		Self { pool }
	}

	pub async fn is_known(&self, hash: &str) -> Result<bool, DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt = conn.prepare("SELECT 1 FROM rainbow_table WHERE hash = ?1;")?;
		let count: u32 = stmt.query_row(params![hash], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, hash_type: &str, hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;

		conn.execute(
			"INSERT OR IGNORE INTO rainbow_table (hash_type, hash) VALUES (?1, ?2);",
			params![hash_type, hash],
		)?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<RainbowTableEntry>, DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt =
			conn.prepare("SELECT id, hash_type, hash, created_at, last_seen FROM rainbow_table;")?;
		let rt_iter = stmt.query_map(params![], |row| {
			let created_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;
			let last_seen = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;

			Ok(RainbowTableEntry {
				id: row.get(0)?,
				hash_type: row.get(1)?,
				hash: row.get(2)?,
				created_at,
				last_seen,
			})
		})?;

		let mut rt_entries = Vec::new();
		for rt_result in rt_iter {
			rt_entries.push(rt_result?);
		}
		Ok(rt_entries)
	}

	pub async fn write_all(&self, entries: &[RainbowTableEntry]) -> Result<(), DatabaseError> {
		let mut conn = self.pool.get()?;

		let tx = conn.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR IGNORE INTO rainbow_table (hash_type, hash, created_at, last_seen) VALUES (?1, ?2, ?3, ?4);",
			)?;
			for entry in entries {
				stmt.execute(params![
					entry.hash_type,
					entry.hash,
					entry.created_at.to_rfc3339(),
					entry.last_seen.to_rfc3339()
				])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

pub struct RainbowTableEntry {
	pub id: i32,
	pub hash_type: String,
	pub hash: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub last_seen: chrono::DateTime<chrono::Utc>,
}

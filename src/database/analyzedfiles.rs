use crate::database::{DatabaseError, migrationtrait::MigrationTrait};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct AnalyzedFiles {
	pool: Pool<SqliteConnectionManager>,
}

impl MigrationTrait for AnalyzedFiles {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

		conn.execute_batch(
			"
			CREATE TABLE analyzed_files (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				file_name TEXT NOT NULL,
				breach_date TEXT,
				breach_target TEXT,
				hash TEXT NOT NULL UNIQUE,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
			);
			",
		)?;
		Ok(())
	}

	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		Self::create(pool)
	}

	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;
		conn.execute_batch("DROP TABLE IF EXISTS analyzed_files;")?;
		Ok(())
	}
}

impl AnalyzedFiles {
	pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
		Self { pool }
	}

	pub async fn is_known(&self, hash: &str) -> Result<bool, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare("SELECT 1 FROM analyzed_files WHERE hash = ?1;")?;
		let count: u32 = stmt.query_row(params![hash], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(
		&self,
		file_name: &str,
		breach_date: Option<&str>,
		breach_target: Option<&str>,
		hash: &str,
	) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt = conn.prepare(
			"INSERT OR IGNORE INTO analyzed_files (file_name, breach_date, breach_target, hash) VALUES (?1, ?2, ?3, ?4);",
		)?;
		stmt.execute(params![file_name, breach_date, breach_target, hash])?;
		Ok(())
	}

	pub async fn seen(&self, hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt = conn
			.prepare("UPDATE analyzed_files SET last_seen = CURRENT_TIMESTAMP WHERE hash = ?1;")?;
		stmt.execute(params![hash])?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<AnalyzedFile>, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare(
			"SELECT id, file_name, breach_date, breach_target, hash, created_at, last_seen FROM analyzed_files;",
		)?;
		let dump_iter = stmt.query_map(params![], |row| {
			let created_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;
			let last_seen = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;

			Ok(AnalyzedFile {
				id: row.get(0)?,
				file_name: row.get(1)?,
				breach_date: row.get(2)?,
				breach_target: row.get(3)?,
				hash: row.get(4)?,
				created_at,
				last_seen,
			})
		})?;

		let mut analyzed_files = Vec::new();
		for analyzed_file in dump_iter {
			analyzed_files.push(analyzed_file?);
		}
		Ok(analyzed_files)
	}

	pub async fn write_all(&self, analyzed_files: &[AnalyzedFile]) -> Result<(), DatabaseError> {
		let mut conn = self.pool.get()?;

		let tx = conn.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR IGNORE INTO analyzed_files (file_name, breach_date, breach_target, hash, created_at, last_seen) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);",
			)?;
			for analyzed_file in analyzed_files {
				stmt.execute(params![
					&analyzed_file.file_name,
					&analyzed_file.breach_date,
					&analyzed_file.breach_target,
					&analyzed_file.hash,
					&analyzed_file.created_at.to_rfc3339(),
					&analyzed_file.last_seen.to_rfc3339(),
				])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

pub struct AnalyzedFile {
	pub id: i32,
	pub file_name: String,
	pub breach_date: Option<String>,
	pub breach_target: Option<String>,
	pub hash: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub last_seen: chrono::DateTime<chrono::Utc>,
}

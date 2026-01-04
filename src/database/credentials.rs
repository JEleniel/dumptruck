use std::collections::HashMap;

use crate::database::{DatabaseError, migrationtrait::MigrationTrait};
use r2d2::Pool;
use r2d2_sqlite::{SqliteConnectionManager, rusqlite::params};

pub struct Credentials {
	pool: Pool<SqliteConnectionManager>,
}

impl MigrationTrait for Credentials {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;
		conn.execute_batch(
			"CREATE TABLE IF NOT EXISTS credentials (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				identity_id INTEGER NOT NULL,
				hash TEXT NOT NULL,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
				UNIQUE (identity_id, hash),
				FOREIGN KEY (identity_id) REFERENCES identities(id) 
				ON DELETE CASCADE
			);",
		)?;
		Ok(())
	}

	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		Self::create(pool)
	}

	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;
		conn.execute_batch("DROP TABLE IF EXISTS credentials;")?;
		Ok(())
	}
}

impl Credentials {
	pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
		Self { pool }
	}

	pub async fn is_known(&self, identity_id: &str, hash: &str) -> Result<bool, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt =
			conn.prepare("SELECT 1 FROM credentials WHERE identity_id = ?1 AND hash = ?2;")?;
		let count: u32 = stmt.query_row(params![identity_id, hash], |row| row.get(0))?;
		Ok(count > 0)
	}

	pub async fn add(&self, identity_id: &str, hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare(
			"INSERT OR IGNORE INTO credentials (identity_id, hash, last_seen) VALUES (?1, ?2, CURRENT_TIMESTAMP);",
		)?;
		stmt.execute(params![identity_id, hash])?;
		Ok(())
	}

	pub async fn seen(&self, identity_id: &str, hash: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare(
			"UPDATE credentials SET last_seen = CURRENT_TIMESTAMP WHERE identity_id = ?1 AND hash = ?2;",
		)?;
		stmt.execute(params![identity_id, hash])?;
		Ok(())
	}

	pub async fn get_all(&self) -> Result<Vec<Credential>, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare(
			"SELECT identity_id, hash, created_at, last_seen FROM credentials ORDER BY created_at DESC;",
		)?;
		let credential_iter = stmt.query_map(params![], |row| {
			let created_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;
			let last_seen = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
				.map_err(|_| rusqlite::Error::InvalidQuery)
				.map(|dt| dt.with_timezone(&chrono::Utc))?;

			let cred = Credential::from((row.get(0)?, row.get(1)?, created_at, last_seen));
			Ok(cred)
		})?;

		let mut credentials = Vec::new();
		for credential in credential_iter {
			credentials.push(credential?);
		}

		Ok(credentials)
	}

	pub async fn write_all(
		&self,
		identity_map: &HashMap<i32, i32>,
		credentials: &[Credential],
	) -> Result<(), DatabaseError> {
		let mut conn = self.pool.get()?;

		let tx = conn.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR IGNORE INTO credentials (identity_id, hash, created_at, last_seen) VALUES (?1, ?2, ?3, ?4);",
			)?;
			for credential in credentials {
				let new_identity_id = identity_map[&credential.identity_id];
				stmt.execute(params![
					new_identity_id,
					credential.hash,
					credential
						.created_at
						.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
					credential
						.last_seen
						.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
				])?;
			}
		}
		tx.commit()?;
		Ok(())
	}
}

pub struct Credential {
	pub identity_id: i32,
	pub hash: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub last_seen: chrono::DateTime<chrono::Utc>,
}

impl
	From<(
		i32,
		String,
		chrono::DateTime<chrono::Utc>,
		chrono::DateTime<chrono::Utc>,
	)> for Credential
{
	fn from(
		tuple: (
			i32,
			String,
			chrono::DateTime<chrono::Utc>,
			chrono::DateTime<chrono::Utc>,
		),
	) -> Self {
		Self {
			identity_id: tuple.0,
			hash: tuple.1,
			created_at: tuple.2,
			last_seen: tuple.3,
		}
	}
}

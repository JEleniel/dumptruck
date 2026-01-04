use crate::database::{DatabaseError, migrate, migrationtrait::MigrationTrait};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct Metadata {
	pool: Pool<SqliteConnectionManager>,
}

impl MigrationTrait for Metadata {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

		conn.execute_batch(
			"CREATE TABLE IF NOT EXISTS metadata (key INT PRIMARY KEY, db_uuid TEXT NOT NULL, migration_version INT);",
		)?;
		let db_uuid = uuid::Uuid::new_v4().to_string();
		conn.execute(
			"INSERT INTO metadata (key, db_uuid, migration_version) VALUES (0, ?1, ?2);",
			rusqlite::params![db_uuid, migrate::CURRENT_MIGRATION_VERSION],
		)?;
		Ok(())
	}

	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		Self::create(pool)
	}

	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
		let conn = pool.get()?;

		conn.execute_batch("DROP TABLE IF EXISTS metadata;")?;
		Ok(())
	}
}

impl Metadata {
	pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
		Self { pool }
	}

	pub async fn get_db_uuid(&self) -> Result<String, DatabaseError> {
		let conn = self.pool.get()?;

		let mut stmt = conn.prepare("SELECT db_uuid FROM metadata WHERE key = 0;")?;
		let db_uuid = stmt.query_row(params![], |row| row.get(0))?;
		Ok(db_uuid)
	}

	pub async fn set_db_uuid(&self, db_uuid: &str) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;
		conn.execute(
			"UPDATE metadata SET db_uuid = ?1 WHERE key = 0;",
			params![db_uuid],
		)?;
		Ok(())
	}

	pub async fn set_migration_version(&self, version: i32) -> Result<(), DatabaseError> {
		let conn = self.pool.get()?;
		conn.execute(
			"UPDATE metadata SET  migration_version = ?1 WHERE key = 0;",
			params![version],
		)?;
		Ok(())
	}

	pub async fn get_migration_version(&self) -> Result<i32, DatabaseError> {
		let conn = self.pool.get()?;
		let mut stmt = conn.prepare("SELECT migration_version FROM metadata WHERE key = 0;")?;
		let version = stmt.query_row(params![], |row| row.get(0))?;
		Ok(version)
	}
}

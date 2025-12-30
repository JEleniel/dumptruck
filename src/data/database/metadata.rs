use std::sync::Arc;

use rusqlite::params;
use tokio::sync::Mutex;

use crate::data::{
	DatabaseError,
	database::{migrationtrait::MigrationTrait, signedconnection::SignedConnection},
};

pub struct Metadata {
	conn: Arc<Mutex<SignedConnection>>,
}

impl MigrationTrait for Metadata {
	fn create(conn: &SignedConnection) -> Result<(), DatabaseError> {
		conn.execute_batch(
			"CREATE TABLE IF NOT EXISTS metadata (key INT PRIMARY KEY, db_uuid TEXT NOT NULL, hash TEXT, migration_version INT);",
		)?;
		let db_uuid = uuid::Uuid::new_v4().to_string();
		conn.execute(
			"INSERT INTO metadata (key, db_uuid, hash, migration_version) VALUES (0, ?1, '', ?2);",
			rusqlite::params![db_uuid, Self::MIGRATION_VERSION],
		)?;
		Ok(())
	}

	fn upgrade(conn: &SignedConnection) -> Result<(), DatabaseError> {
		Self::create(conn)
	}

	fn downgrade(conn: &SignedConnection) -> Result<(), DatabaseError> {
		conn.execute_batch("DROP TABLE IF EXISTS metadata;")?;
		Ok(())
	}
}

impl Metadata {
	pub const MIGRATION_VERSION: i32 = 1;

	pub fn new(conn: Arc<Mutex<SignedConnection>>) -> Self {
		Self { conn }
	}

	pub async fn get_db_uuid(&self) -> rusqlite::Result<String> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT db_uuid FROM metadata WHERE key = 0;")?;
		let db_uuid = stmt.query_row(params![], |row| row.get(0))?;
		Ok(db_uuid)
	}

	pub async fn set_db_uuid(&self, db_uuid: &str) -> rusqlite::Result<()> {
		self.conn.lock().await.execute(
			"UPDATE metadata SET db_uuid = ?1 WHERE key = 0;",
			params![db_uuid],
		)?;
		Ok(())
	}

	pub async fn set_migration_version(&self, version: i32) -> rusqlite::Result<()> {
		self.conn.lock().await.execute(
			"UPDATE metadata SET  migration_version = ?1 WHERE key = 0;",
			params![version],
		)?;
		Ok(())
	}

	pub async fn get_migration_version(&self) -> rusqlite::Result<i32> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT migration_version FROM metadata WHERE key = 0;")?;
		let version = stmt.query_row(params![], |row| row.get(0))?;
		Ok(version)
	}

	pub async fn set_hash(&self, hash: &str) -> rusqlite::Result<()> {
		self.conn.lock().await.execute(
			"INSERT OR REPLACE INTO metadata (key, hash, migration_version) VALUES (?1, ?2, ?3);",
			params![0, hash, Self::MIGRATION_VERSION],
		)?;
		Ok(())
	}

	pub async fn get_hash(&self) -> rusqlite::Result<Option<String>> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT hash FROM metadata WHERE key = 0;")?;
		let mut hash = stmt.query_row(params![], |row| row.get(0))?;

		Ok(Some(hash))
	}

	pub async fn get_all(&self) -> rusqlite::Result<Vec<(i32, String)>> {
		let mut stmt = self
			.conn
			.lock()
			.await
			.prepare("SELECT key, hash FROM metadata;")?;
		let metadata_iter = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?;

		let mut metadata = Vec::new();
		for meta_result in metadata_iter {
			metadata.push(meta_result?);
		}
		Ok(metadata)
	}

	pub async fn write_all(&self, entries: &[(i32, String)]) -> rusqlite::Result<()> {
		let tx = self.conn.lock().await.transaction()?;
		{
			let mut stmt = tx.prepare(
				"INSERT OR REPLACE INTO metadata (key, hash, migration_version) VALUES (?1, ?2, ?3);",
			)?;
			for (key, hash) in entries {
				stmt.execute(params![key, hash, Self::MIGRATION_VERSION])?;
			}
		}
		tx.commit()
	}
}

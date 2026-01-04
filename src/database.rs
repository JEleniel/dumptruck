mod analyzedfiles;
mod credentials;
pub mod exportargs;
pub mod identities;
pub mod importargs;
mod metadata;
mod migrate;
mod migrationtrait;
mod npi;
mod rainbowtable;

use crate::database::{
	analyzedfiles::AnalyzedFiles, credentials::Credentials, exportargs::ExportArgs,
	identities::Identities, importargs::ImportArgs, metadata::Metadata, npi::NPI,
	rainbowtable::RainbowTable,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tracing::info;

pub struct Database {
	pub path: PathBuf,
	pool: Pool<SqliteConnectionManager>,
	pub credentials: Credentials,
	pub dumps: AnalyzedFiles,
	pub identities: Identities,
	pub metadata: Metadata,
	pub npi: NPI,
	pub rainbowtable: RainbowTable,
}

impl Database {
	/// Open a new database, creating it if it does not exist.
	pub async fn open(path: &PathBuf) -> Result<Self, DatabaseError> {
		let path = if path.is_dir() {
			path.join("dumptruck.db")
		} else {
			path.clone()
		};
		if !path.exists() {
			info!(
				"Database file not found at {:?}, creating new database",
				path
			);
			let conn = rusqlite::Connection::open(&path)?;
			conn.close()
				.map_err(|(_, e)| DatabaseError::SqliteError(e))?;
		}

		let manager = SqliteConnectionManager::file(&path);
		let pool = Pool::new(manager)?;

		let db = Self {
			path: path.clone(),
			pool: pool.clone(),
			credentials: Credentials::new(pool.clone()),
			dumps: AnalyzedFiles::new(pool.clone()),
			identities: Identities::new(pool.clone()),
			metadata: Metadata::new(pool.clone()),
			npi: NPI::new(pool.clone()),
			rainbowtable: RainbowTable::new(pool.clone()),
		};

		if db.metadata.get_migration_version().await? != 0 {
			info!("Initializing database at {:?}", path);
			migrate::create(pool.clone())?;
		} else if db.metadata.get_migration_version().await? != migrate::CURRENT_MIGRATION_VERSION {
			info!("Migrating database at {:?}", path);
			migrate::upgrade(pool.clone())?;
		}

		info!("Creating new database at {:?}", path);
		migrate::create(pool.clone())?;

		Ok(db)
	}

	pub async fn export(&mut self, arg: &ExportArgs) -> Result<(), DatabaseError> {
		info!("Exporting database to {:?}", arg.output_path);

		// Close all pool connections by dropping the pool
		drop(&self.pool);

		// "Export" by copying the database file
		fs::copy(&self.path, &arg.output_path).await?;

		// Re-open the pool
		let manager = SqliteConnectionManager::file(&self.path);
		self.pool = Pool::new(manager)?;

		// Assign the copy a new UUID
		let export = Database::open(&arg.output_path).await?;
		let uuid_str = uuid::Uuid::new_v4().to_string();
		export.metadata.set_db_uuid(&uuid_str).await?;

		Ok(())
	}

	pub async fn import(&self, arg: &ImportArgs) -> Result<(), DatabaseError> {
		if self.path == arg.input_path {
			return Err(DatabaseError::FileError(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"Cannot import from the same database file",
			)));
		}

		info!("Importing database from {:?}", arg.input_path);
		let source_db = Database::open(&arg.input_path).await?;

		// Dumps
		let dumps = source_db.dumps.get_all().await?;
		self.dumps.write_all(&dumps).await?;

		// Identities
		let identities = source_db.identities.get_all().await?;
		let identity_map = self.identities.write_all(&identities).await?;

		// Credentials: convert full rows into (identity_id, hash)
		let creds = source_db.credentials.get_all().await?;
		self.credentials.write_all(&identity_map, &creds).await?;
		// PII
		let pii_entries = source_db.npi.get_all().await?;
		self.npi.write_all(&pii_entries).await?;

		// Rainbow table entries
		let rt_entries = source_db.rainbowtable.get_all().await?;
		self.rainbowtable.write_all(&rt_entries).await?;

		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum DatabaseError {
	#[error("File operation error: {0}")]
	FileError(#[from] std::io::Error),
	#[error("SQLite operation error: {0}")]
	SqliteError(#[from] rusqlite::Error),
	#[error("An r2d2 operation failed: {0}")]
	R2D2Error(#[from] r2d2::Error),
	#[error("Error parsing date/time: {0}")]
	ParseError(#[from] chrono::ParseError),
}

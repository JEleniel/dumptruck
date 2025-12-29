mod credentials;
mod dumps;
mod identities;
mod metadata;
mod migrate;
mod migrationtrait;
mod pii;
mod rainbowtable;
mod seedfiles;
mod signedconnection;

use crate::{
	common::Hash,
	data::{
		ExportArgs, ImportArgs,
		database::{
			credentials::Credentials, dumps::Dumps, identities::Identities, metadata::Metadata,
			pii::Pii, rainbowtable::RainbowTable, seedfiles::SeedFiles,
			signedconnection::SignedConnection,
		},
	},
};
use rusqlite::Connection;
use std::{fs::File, path::PathBuf, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Database {
	pub path: PathBuf,
	conn: Arc<Mutex<SignedConnection>>,
	credentials: Credentials,
	dumps: Dumps,
	identities: Identities,
	metadata: Metadata,
	pii: Pii,
	rainbowtable: RainbowTable,
	seedfiles: SeedFiles,
}

impl Database {
	pub fn create(path: &PathBuf) -> Result<Self, DatabaseError> {
		let conn = SignedConnection::open(path)?;
		let conn = Arc::new(Mutex::new(conn));

		let db = Self {
			path,
			conn: conn.clone(),
			credentials: Credentials::new(conn.clone()),
			dumps: Dumps::new(conn.clone()),
			identities: Identities::new(conn.clone()),
			metadata: Metadata::new(conn.clone()),
			pii: Pii::new(conn.clone()),
			rainbowtable: RainbowTable::new(conn.clone()),
			seedfiles: SeedFiles::new(conn.clone()),
		};

		info!("Creating new database at {:?}", path);
		migrate::create(&db.conn)?;
		db.sign()?;

		Ok(db)
	}

	pub async fn connect(
		path: &PathBuf,
		seed_path: &Option<PathBuf>,
	) -> Result<Self, DatabaseError> {
		if !File::exists(path) {
			if let Some(seed_path) = seed_path {
				info!("Seeding database from {:?}", seed_path);
				File::copy(seed_path, path)?;
			} else {
				return Err(DatabaseError::DatabaseNotFound(path.clone()));
			}
		}
		let conn = SignedConnection::open(path)?;
		let conn = Arc::new(Mutex::new(conn));

		let db = Self {
			path,
			conn: conn.clone(),
			credentials: Credentials::new(conn.clone()),
			dumps: Dumps::new(conn.clone()),
			identities: Identities::new(conn.clone()),
			metadata: Metadata::new(conn.clone()),
			pii: Pii::new(conn.clone()),
			rainbowtable: RainbowTable::new(conn.clone()),
			seedfiles: SeedFiles::new(conn.clone()),
		};

		if !db.validate().await? {
			return Err(DatabaseError::DatabaseCorrupted(path.clone()));
		}

		if Metadata::get_migration_version(&conn) != Metadata::MIGRATION_VERSION {
			info!("Running database migrations");
			migrate::upgrade(&db.conn)?;
		}

		Ok(db)
	}

	pub async fn validate(&self) -> Result<bool, DatabaseError> {
		let hash = self.metadata.get_db_hash().await?;
		self.conn.lock().await.clone();
		let current_hash = Hash::calculate_sha256(self.path)?;
		let is_valid = hash == current_hash;
		self.conn.lock().await.open()?;
		Ok(is_valid)
	}

	pub async fn export(&self, arg: &ExportArgs) -> Result<(), DatabaseError> {
		info!("Exporting database to {:?}", arg.output_path);
		self.conn.lock().await.close()?;
		File::copy(&self.path, &arg.output_path)?;
		self.conn = Arc::new(Mutex::new(Connection::open(&self.path)?));
		let export = Database::connect(&arg.output_path, &None)?;
		export
			.metadata
			.set_db_uuid(uuid::Uuid::new_v4().to_string());
		export.dumps.clear()?;
		Ok(())
	}

	pub fn import(&self, arg: &ImportArgs) -> Result<(), DatabaseError> {
		info!("Importing database from {:?}", arg.input_path);
		let source_db = Database::connect(&arg.input_path, &None)?;

		self.dumps.write_all(source_db.dumps.get_all())?;
		self.identities.write_all(source_db.identities.get_all())?;
		self.credentials
			.write_all(source_db.credentials.get_all())?;
		self.pii.write_all(source_db.pii.get_all())?;
		self.rainbowtable
			.write_all(source_db.rainbowtable.get_all())?;
		self.seedfiles.write_all(source_db.seedfiles.get_all())?;

		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum DatabaseError {
	#[error("Database file not found, and no seed available, at path: {0}")]
	DatabaseNotFound(PathBuf),
	#[error("Failed to open database: {0}")]
	OpenError(#[from] rusqlite::Error),
	#[error("File operation error: {0}")]
	FileError(#[from] std::io::Error),
	#[error("Database is corrupted or has invalid signature: {0}")]
	DatabaseCorrupted(PathBuf),
}

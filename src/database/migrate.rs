use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::database::{
	DatabaseError, analyzedfiles::AnalyzedFiles, credentials::Credentials, identities::Identities,
	metadata::Metadata, migrationtrait::MigrationTrait, npi::NPI, rainbowtable::RainbowTable,
};

pub const CURRENT_MIGRATION_VERSION: i32 = 1;

pub fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
	Credentials::create(pool.clone())?;
	AnalyzedFiles::create(pool.clone())?;
	Identities::create(pool.clone())?;
	Metadata::create(pool.clone())?;
	NPI::create(pool.clone())?;
	RainbowTable::create(pool.clone())?;
	Ok(())
}

pub fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
	Credentials::upgrade(pool.clone())?;
	AnalyzedFiles::upgrade(pool.clone())?;
	Identities::upgrade(pool.clone())?;
	Metadata::upgrade(pool.clone())?;
	NPI::upgrade(pool.clone())?;
	RainbowTable::upgrade(pool.clone())?;
	Ok(())
}

pub fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError> {
	Credentials::downgrade(pool.clone())?;
	AnalyzedFiles::downgrade(pool.clone())?;
	Identities::downgrade(pool.clone())?;
	Metadata::downgrade(pool.clone())?;
	NPI::downgrade(pool.clone())?;
	RainbowTable::downgrade(pool.clone())?;
	Ok(())
}

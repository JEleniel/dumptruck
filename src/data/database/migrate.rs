use crate::data::database::{
	credentials::Credentials, dumps::Dumps, identities::Identities, metadata::Metadata,
	migrationtrait::MigrationTrait, pii::Pii, rainbowtable::RainbowTable, seedfiles::SeedFiles,
	signedconnection::SignedConnection,
};

pub fn create(conn: &SignedConnection) -> Result<(), crate::data::DatabaseError> {
	Credentials::create(conn)?;
	Dumps::create(conn)?;
	Identities::create(conn)?;
	Metadata::create(conn)?;
	Pii::create(conn)?;
	RainbowTable::create(conn)?;
	SeedFiles::create(conn)?;
	Ok(())
}

pub fn upgrade(conn: &SignedConnection) -> Result<(), crate::data::DatabaseError> {
	Credentials::upgrade(conn)?;
	Dumps::upgrade(conn)?;
	Identities::upgrade(conn)?;
	Metadata::upgrade(conn)?;
	Pii::upgrade(conn)?;
	RainbowTable::upgrade(conn)?;
	SeedFiles::upgrade(conn)?;
	Ok(())
}

pub fn downgrade(conn: &SignedConnection) -> Result<(), crate::data::DatabaseError> {
	Credentials::downgrade(conn)?;
	Dumps::downgrade(conn)?;
	Identities::downgrade(conn)?;
	Metadata::downgrade(conn)?;
	Pii::downgrade(conn)?;
	RainbowTable::downgrade(conn)?;
	SeedFiles::downgrade(conn)?;
	Ok(())
}

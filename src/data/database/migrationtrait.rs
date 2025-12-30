use crate::data::{DatabaseError, database::signedconnection::SignedConnection};

pub trait MigrationTrait {
	fn create(conn: &SignedConnection) -> Result<(), DatabaseError>;
	fn upgrade(conn: &SignedConnection) -> Result<(), DatabaseError>;
	fn downgrade(conn: &SignedConnection) -> Result<(), DatabaseError>;
}

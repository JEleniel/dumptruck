use crate::data::DatabaseError;

pub trait MigrationTrait {
	fn create(conn: &rusqlite::Connection) -> Result<(), DatabaseError>;
	fn upgrade(conn: &rusqlite::Connection) -> Result<(), DatabaseError>;
	fn downgrade(conn: &rusqlite::Connection) -> Result<(), DatabaseError>;
}

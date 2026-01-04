use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::database::DatabaseError;

pub trait MigrationTrait {
	fn create(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError>;
	fn upgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError>;
	fn downgrade(pool: Pool<SqliteConnectionManager>) -> Result<(), DatabaseError>;
}

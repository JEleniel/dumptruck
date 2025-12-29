use crate::{common::Hash, data::DatabaseError};
use rusqlite::Connection;
use std::ops::{Deref, DerefMut};

pub struct SignedConnection {
	conn: Connection,
}

impl SignedConnection {
	pub fn open(path: &str) -> Result<Self, DatabaseError> {
		Ok(Self {
			conn: Connection::open(path)?,
		})
	}

	fn finalize_signature(&mut self) -> Result<(), DatabaseError> {
		self.conn.close()?;
		let hash = Hash::calculate_sha256(self.conn.path())?;
		self.conn.open(self.conn.path())?;

		self.conn
			.execute("UPDATE metadata SET hash=?1 WHERE key=0", [hash])?;
		Ok(())
	}
}

impl Drop for SignedConnection {
	fn drop(&mut self) {
		self.finalize_signature();
	}
}

impl Deref for SignedConnection {
	type Target = Connection;
	fn deref(&self) -> &Self::Target {
		&self.conn
	}
}

impl DerefMut for SignedConnection {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.conn
	}
}

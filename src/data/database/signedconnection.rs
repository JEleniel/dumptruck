use crate::{common::Hash, data::DatabaseError};
use rusqlite::Connection;
use std::{
	fs::File,
	io::BufReader,
	ops::{Deref, DerefMut},
	path::PathBuf,
};

pub struct SignedConnection {
	path: PathBuf,
	conn: Connection,
}

impl SignedConnection {
	pub fn open(path: &PathBuf) -> Result<Self, DatabaseError> {
		Ok(Self {
			path: path.clone(),
			conn: Connection::open(path)?,
		})
	}

	fn finalize_signature(&mut self) -> Result<(), DatabaseError> {
		self.conn.close()?;
		let hash = Hash::calculate_sha256(&mut BufReader::new(File::open(&self.path)?))?;
		self.conn.open(&self.path)?;

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

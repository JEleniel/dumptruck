//! Storage and persistence
//!
//! This module handles data persistence across multiple backends:
//! - SQLite storage (addresses, aliases, credentials, metadata)
//! - Database import/export with compression and versioning
//! - Job queue for asynchronous processing
//! - Working copy management for transactional operations
//! - Chain of custody for cryptographic audit trails

pub mod chain_of_custody;
pub mod db;
pub mod db_export;
pub mod db_import;
pub mod db_stats;
pub mod job_queue;
pub mod working_copy;

pub use chain_of_custody::{
	ChainOfCustodyError, ChainOfCustodyRecord, CustodyAction, CustodyKeyPair,
};
pub use db::*;
pub use db_export::export_database;
pub use db_import::import_database;
pub use db_stats::DatabaseStats;
pub use job_queue::JobQueue;
pub use working_copy::WorkingCopyManager;

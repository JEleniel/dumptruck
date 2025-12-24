//! Core utilities and configuration
//!
//! This module contains fundamental utilities and configuration components:
//! - Configuration management and loading
//! - Hash utilities (SHA-256, fingerprinting)
//! - File locking mechanisms
//! - Secure deletion procedures

pub mod config;
pub mod file_lock;
pub mod hash_utils;
pub mod secure_deletion;

pub use config::Config;
pub use file_lock::FileLock;

//! Normalization and deduplication
//!
//! This module handles normalization of breach data across all variants:
//! - Unicode canonicalization (NFKC normalization)
//! - Email alias resolution (googlemail.com → gmail.com)
//! - Address deduplication and alias linking
//! - Evidence preservation and file tracking

pub mod alias_resolution;
pub mod engine;
pub mod evidence;

pub use alias_resolution::normalize_phone_e164;
pub use engine::{normalize_field, normalize_row};

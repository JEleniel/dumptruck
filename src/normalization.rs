//! Normalization and deduplication
//!
//! This module handles normalization of breach data across all variants:
//! - Unicode canonicalization (NFKC normalization)
//! - Email alias resolution (googlemail.com → gmail.com)
//! - Address deduplication and alias linking
//! - Evidence preservation and file tracking

pub mod alias_resolution;
pub mod evidence;
pub mod normalization;

pub use normalization::{normalize_field, normalize_row};

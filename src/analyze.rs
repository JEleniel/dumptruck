//! Data ingestion and format parsing
//!
//! This module handles all aspects of data ingestion:
//! - Safe ingestion with validation (binary detection, UTF-8 checking)
//! - Multiple format support (CSV, TSV, JSON, YAML, XML, Protocol Buffers, BSON)
//! - Compression detection (ZIP, gzip with nested level limits)
//! - Memory-efficient streaming pipelines

mod analyzeargs;
mod analyzeerror;

pub use analyzeargs::*;
pub use analyzeerror::*;

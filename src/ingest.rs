//! Data ingestion and format parsing
//!
//! This module handles all aspects of data ingestion:
//! - Safe ingestion with validation (binary detection, UTF-8 checking)
//! - Multiple format support (CSV, TSV, JSON, YAML, XML, Protocol Buffers, BSON)
//! - Compression detection (ZIP, gzip with nested level limits)
//! - Memory-efficient streaming pipelines

pub mod adapters;
pub mod compression;
pub mod safe_ingest;
pub mod streaming;
pub mod universal_parser;

pub use compression::{CompressionFormat, CompressionInfo};
pub use safe_ingest::FileSafetyAnalysis;
pub use streaming::{StreamingCsvParser, StreamingJsonLinesParser};

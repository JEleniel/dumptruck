//! HTTP API server and request handlers
//!
//! This module provides REST API endpoints and request handling:
//! - HTTP/2 server using Axum framework
//! - Request handlers for ingest, query, and export operations
//! - Output formatters (JSON, CSV, JSONL, text) with field classification
//! - Authentication and authorization middleware

pub mod handlers;
pub mod output;
pub mod server;

pub use handlers::{ingest, server, stats, status};
pub use output::OutputFormatter;
pub use server::AppState;

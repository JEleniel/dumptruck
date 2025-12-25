//! Deployment and orchestration
//!
//! This module manages application lifecycle and batch processing:
//! - Service management for optional Ollama Docker container (if enabled and Docker available)
//! - Async pipeline orchestration with Tokio
//! - Batch file ingestion with parallel workers
//! - Streaming data processing with backpressure

pub mod async_pipeline;
pub mod deploy_manager;
pub mod pipeline;

pub use async_pipeline::{AsyncPipeline, ProcessAddressesParams};
pub use deploy_manager::ServiceManager;
pub use pipeline::Pipeline;

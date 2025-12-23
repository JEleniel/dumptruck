//! Deployment and orchestration
//!
//! This module manages application lifecycle and batch processing:
//! - Service management for Docker/systemd deployment
//! - Async pipeline orchestration with Tokio
//! - Batch file ingestion with parallel workers
//! - Streaming data processing with backpressure

pub mod async_pipeline;
pub mod deploy_manager;
pub mod pipeline;

pub use async_pipeline::AsyncPipeline;
pub use deploy_manager::ServiceManager;
pub use pipeline::Pipeline;

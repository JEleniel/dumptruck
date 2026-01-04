//! Data enrichment and intelligence
//!
//! This module enriches breach data with contextual intelligence:
//! - Have I Been Pwned (HIBP) API integration for breach lookups
//! - Ollama embeddings for similarity search and near-duplicate detection
//! - Risk scoring based on compromise potential and historical context
//! - Rainbow table construction for pre-hashed credential detection

pub mod hibp;
pub mod ollama;
pub mod risk_scoring;

pub use hibp::HibpClient;
pub use ollama::OllamaClient;
pub use risk_scoring::RiskScore;

//! Ingest statistics and aggregation.
//!
//! Tracks statistics across files during ingest operations including row counts,
//! detections, and errors.

use crate::api::output::{DetectionFieldGroup, PiiDetectionSummary};
use std::collections::BTreeMap;

/// Statistics aggregated across files during ingest
#[derive(Default)]
pub struct IngestStats {
	pub total_rows: usize,
	pub unique_addresses: usize,
	pub hashed_credentials: usize,
	pub weak_passwords: usize,
	pub pii_summary: PiiDetectionSummary,
	pub detection_groups: BTreeMap<String, Vec<DetectionFieldGroup>>,
	pub metadata: Vec<String>,
	pub errors: Vec<String>,
}

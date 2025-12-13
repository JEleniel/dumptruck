//! Job tracking and async processing for Dumptruck server.
//!
//! Manages ingest jobs with status tracking, result storage, and cleanup.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Job errors
#[derive(Debug, Error)]
pub enum JobError {
	#[error("Job not found: {0}")]
	NotFound(String),

	#[error("Job already exists: {0}")]
	AlreadyExists(String),

	#[error("Invalid job state: {0}")]
	InvalidState(String),

	#[error("Job processing failed: {0}")]
	ProcessingFailed(String),
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
	/// Job queued waiting for processing
	Queued,
	/// Currently being processed
	Processing,
	/// Successfully completed
	Completed,
	/// Processing failed
	Failed,
	/// Job cancelled
	Cancelled,
}

impl std::fmt::Display for JobStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			JobStatus::Queued => write!(f, "queued"),
			JobStatus::Processing => write!(f, "processing"),
			JobStatus::Completed => write!(f, "completed"),
			JobStatus::Failed => write!(f, "failed"),
			JobStatus::Cancelled => write!(f, "cancelled"),
		}
	}
}

/// Job metadata and progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
	pub id: String,
	pub status: JobStatus,
	pub created_at: DateTime<Utc>,
	pub started_at: Option<DateTime<Utc>>,
	pub completed_at: Option<DateTime<Utc>>,
	pub filename: String,
	pub file_size_bytes: u64,
	pub rows_processed: usize,
	pub error_message: Option<String>,
	pub progress_percentage: u32,
}

impl Job {
	/// Create a new job
	pub fn new(filename: String, file_size_bytes: u64) -> Self {
		Self {
			id: Uuid::new_v4().to_string(),
			status: JobStatus::Queued,
			created_at: Utc::now(),
			started_at: None,
			completed_at: None,
			filename,
			file_size_bytes,
			rows_processed: 0,
			error_message: None,
			progress_percentage: 0,
		}
	}

	/// Mark job as processing
	pub fn start_processing(&mut self) -> Result<(), JobError> {
		if self.status != JobStatus::Queued {
			return Err(JobError::InvalidState(format!(
				"Cannot start processing job in {} state",
				self.status
			)));
		}
		self.status = JobStatus::Processing;
		self.started_at = Some(Utc::now());
		Ok(())
	}

	/// Mark job as completed
	pub fn complete(&mut self, rows_processed: usize) -> Result<(), JobError> {
		if self.status != JobStatus::Processing {
			return Err(JobError::InvalidState(format!(
				"Cannot complete job in {} state",
				self.status
			)));
		}
		self.status = JobStatus::Completed;
		self.rows_processed = rows_processed;
		self.progress_percentage = 100;
		self.completed_at = Some(Utc::now());
		Ok(())
	}

	/// Mark job as failed
	pub fn fail(&mut self, error: String) -> Result<(), JobError> {
		if self.status != JobStatus::Processing {
			return Err(JobError::InvalidState(format!(
				"Cannot fail job in {} state",
				self.status
			)));
		}
		self.status = JobStatus::Failed;
		self.error_message = Some(error);
		self.completed_at = Some(Utc::now());
		Ok(())
	}

	/// Update progress
	pub fn update_progress(&mut self, rows_processed: usize, total_rows: usize) {
		self.rows_processed = rows_processed;
		if total_rows > 0 {
			self.progress_percentage = ((rows_processed as f64 / total_rows as f64) * 100.0) as u32;
		}
	}
}

/// In-memory job queue
pub struct JobQueue {
	jobs: Arc<RwLock<HashMap<String, Job>>>,
}

impl JobQueue {
	/// Create a new job queue
	pub fn new() -> Self {
		Self {
			jobs: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Create and enqueue a new job
	pub async fn enqueue(&self, filename: String, file_size_bytes: u64) -> Result<String, JobError> {
		let job = Job::new(filename, file_size_bytes);
		let job_id = job.id.clone();

		let mut jobs = self.jobs.write().await;
		if jobs.contains_key(&job_id) {
			return Err(JobError::AlreadyExists(job_id));
		}

		jobs.insert(job_id.clone(), job);
		Ok(job_id)
	}

	/// Get job status
	pub async fn get_job(&self, job_id: &str) -> Result<Job, JobError> {
		let jobs = self.jobs.read().await;
		jobs
			.get(job_id)
			.cloned()
			.ok_or_else(|| JobError::NotFound(job_id.to_string()))
	}

	/// List all jobs with pagination
	pub async fn list_jobs(&self, offset: usize, limit: usize) -> (Vec<Job>, usize) {
		let jobs = self.jobs.read().await;
		let total = jobs.len();

		let mut jobs_vec: Vec<_> = jobs.values().cloned().collect();
		jobs_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));

		let paginated: Vec<_> = jobs_vec
			.into_iter()
			.skip(offset)
			.take(limit)
			.collect();

		(paginated, total)
	}

	/// Update job status
	pub async fn update_job<F>(&self, job_id: &str, update: F) -> Result<Job, JobError>
	where
		F: FnOnce(&mut Job) -> Result<(), JobError>,
	{
		let mut jobs = self.jobs.write().await;
		let job = jobs
			.get_mut(job_id)
			.ok_or_else(|| JobError::NotFound(job_id.to_string()))?;

		update(job)?;
		Ok(job.clone())
	}

	/// Cancel a job
	pub async fn cancel_job(&self, job_id: &str) -> Result<Job, JobError> {
		self.update_job(job_id, |job| {
			if job.status != JobStatus::Queued && job.status != JobStatus::Processing {
				return Err(JobError::InvalidState(format!(
					"Cannot cancel job in {} state",
					job.status
				)));
			}
			job.status = JobStatus::Cancelled;
			job.completed_at = Some(Utc::now());
			Ok(())
		})
		.await
	}
}

impl Default for JobQueue {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_job_creation() {
		let job = Job::new("test.csv".to_string(), 1024);
		assert_eq!(job.status, JobStatus::Queued);
		assert_eq!(job.filename, "test.csv");
		assert_eq!(job.file_size_bytes, 1024);
	}

	#[test]
	fn test_job_state_transitions() {
		let mut job = Job::new("test.csv".to_string(), 1024);

		assert!(job.start_processing().is_ok());
		assert_eq!(job.status, JobStatus::Processing);

		assert!(job.complete(100).is_ok());
		assert_eq!(job.status, JobStatus::Completed);
		assert_eq!(job.rows_processed, 100);
	}

	#[test]
	fn test_job_invalid_state_transition() {
		let mut job = Job::new("test.csv".to_string(), 1024);
		job.status = JobStatus::Completed;

		assert!(job.start_processing().is_err());
	}

	#[tokio::test]
	async fn test_job_queue_enqueue() {
		let queue = JobQueue::new();
		let job_id = queue
			.enqueue("test.csv".to_string(), 1024)
			.await
			.expect("enqueue failed");

		let job = queue.get_job(&job_id).await.expect("get failed");
		assert_eq!(job.status, JobStatus::Queued);
	}

	#[tokio::test]
	async fn test_job_queue_list() {
		let queue = JobQueue::new();
		queue
			.enqueue("test1.csv".to_string(), 1024)
			.await
			.expect("enqueue 1 failed");
		queue
			.enqueue("test2.csv".to_string(), 2048)
			.await
			.expect("enqueue 2 failed");

		let (jobs, total) = queue.list_jobs(0, 10).await;
		assert_eq!(jobs.len(), 2);
		assert_eq!(total, 2);
	}

	#[tokio::test]
	async fn test_job_queue_update() {
		let queue = JobQueue::new();
		let job_id = queue
			.enqueue("test.csv".to_string(), 1024)
			.await
			.expect("enqueue failed");

		queue
			.update_job(&job_id, |job| job.start_processing())
			.await
			.expect("update failed");

		let job = queue.get_job(&job_id).await.expect("get failed");
		assert_eq!(job.status, JobStatus::Processing);
	}
}

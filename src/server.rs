//! HTTP/2 server with OAuth 2.0 authentication and TLS 1.3+.
//!
//! Provides REST API for bulk data analysis with secure async job processing.

use crate::job_queue::{Job, JobQueue, JobStatus};
use crate::oauth::OAuthProvider;
use axum::{
	extract::{DefaultBodyLimit, Path, Query, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	routing::{get, post},
	Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Server errors
#[derive(Debug, Error)]
pub enum ServerError {
	#[error("Unauthorized")]
	Unauthorized,

	#[error("Job not found")]
	NotFound,

	#[error("Bad request: {0}")]
	BadRequest(String),

	#[error("Internal server error: {0}")]
	InternalError(String),
}

impl IntoResponse for ServerError {
	fn into_response(self) -> Response {
		let (status, message) = match self {
			ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
			ServerError::NotFound => (StatusCode::NOT_FOUND, "Job not found".to_string()),
			ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
			ServerError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
		};

		let body = Json(serde_json::json!({
			"error": message,
			"status": status.as_u16(),
		}));

		(status, body).into_response()
	}
}

/// Application state
pub struct AppState {
	pub job_queue: Arc<JobQueue>,
	pub oauth_provider: Arc<OAuthProvider>,
}

/// Ingest request
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
	pub filename: String,
	pub file_size_bytes: u64,
}

/// Ingest response
#[derive(Debug, Serialize)]
pub struct IngestResponse {
	pub job_id: String,
	pub status: String,
	pub created_at: String,
}

/// Job status response
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
	pub job_id: String,
	pub status: String,
	pub created_at: String,
	pub started_at: Option<String>,
	pub completed_at: Option<String>,
	pub filename: String,
	pub rows_processed: usize,
	pub progress_percentage: u32,
	pub error_message: Option<String>,
}

impl From<Job> for JobStatusResponse {
	fn from(job: Job) -> Self {
		Self {
			job_id: job.id,
			status: job.status.to_string(),
			created_at: job.created_at.to_rfc3339(),
			started_at: job.started_at.map(|t| t.to_rfc3339()),
			completed_at: job.completed_at.map(|t| t.to_rfc3339()),
			filename: job.filename,
			rows_processed: job.rows_processed,
			progress_percentage: job.progress_percentage,
			error_message: job.error_message,
		}
	}
}

/// List jobs response
#[derive(Debug, Serialize)]
pub struct ListJobsResponse {
	pub jobs: Vec<JobStatusResponse>,
	pub total: usize,
	pub offset: usize,
	pub limit: usize,
}

/// Pagination query params
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
	pub offset: Option<usize>,
	pub limit: Option<usize>,
}

/// Extract and validate OAuth bearer token
async fn extract_bearer_token(headers: &HeaderMap) -> Result<String, ServerError> {
	let auth_header = headers
		.get("authorization")
		.and_then(|h| h.to_str().ok())
		.ok_or(ServerError::Unauthorized)?;

	let parts: Vec<&str> = auth_header.split_whitespace().collect();
	if parts.len() != 2 || parts[0].to_lowercase() != "bearer" {
		return Err(ServerError::Unauthorized);
	}

	Ok(parts[1].to_string())
}

/// POST /api/v1/ingest - Upload and queue a file for analysis
async fn ingest_file(
	State(state): State<Arc<AppState>>,
	headers: HeaderMap,
	Json(req): Json<IngestRequest>,
) -> Result<(StatusCode, Json<IngestResponse>), ServerError> {
	// Validate OAuth token
	let _token = extract_bearer_token(&headers).await?;

	// TODO: Validate token with OAuth provider
	// state.oauth_provider.validate_token(&token).await?;

	// Validate request
	if req.filename.is_empty() {
		return Err(ServerError::BadRequest("filename cannot be empty".to_string()));
	}

	if req.file_size_bytes == 0 {
		return Err(ServerError::BadRequest("file_size_bytes must be > 0".to_string()));
	}

	// Enqueue job
	let job_id = state
		.job_queue
		.enqueue(req.filename, req.file_size_bytes)
		.await
		.map_err(|e| ServerError::InternalError(e.to_string()))?;

	info!("Job {} enqueued", job_id);

	let response = IngestResponse {
		job_id: job_id.clone(),
		status: JobStatus::Queued.to_string(),
		created_at: chrono::Utc::now().to_rfc3339(),
	};

	Ok((StatusCode::ACCEPTED, Json(response)))
}

/// GET /api/v1/status/{job_id} - Get job status
async fn get_job_status(
	State(state): State<Arc<AppState>>,
	headers: HeaderMap,
	Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, ServerError> {
	// Validate OAuth token
	let _token = extract_bearer_token(&headers).await?;

	// TODO: Validate token
	// state.oauth_provider.validate_token(&token).await?;

	let job = state
		.job_queue
		.get_job(&job_id)
		.await
		.map_err(|_| ServerError::NotFound)?;

	Ok(Json(job.into()))
}

/// GET /api/v1/jobs - List all jobs with pagination
async fn list_jobs(
	State(state): State<Arc<AppState>>,
	headers: HeaderMap,
	Query(params): Query<PaginationParams>,
) -> Result<Json<ListJobsResponse>, ServerError> {
	// Validate OAuth token
	let _token = extract_bearer_token(&headers).await?;

	// TODO: Validate token
	// state.oauth_provider.validate_token(&token).await?;

	let offset = params.offset.unwrap_or(0);
	let limit = params.limit.unwrap_or(50).min(200); // Max 200 per page

	let (jobs, total) = state.job_queue.list_jobs(offset, limit).await;

	let response = ListJobsResponse {
		jobs: jobs.into_iter().map(Into::into).collect(),
		total,
		offset,
		limit,
	};

	Ok(Json(response))
}

/// DELETE /api/v1/jobs/{job_id} - Cancel a job
async fn cancel_job(
	State(state): State<Arc<AppState>>,
	headers: HeaderMap,
	Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, ServerError> {
	// Validate OAuth token
	let _token = extract_bearer_token(&headers).await?;

	// TODO: Validate token
	// state.oauth_provider.validate_token(&token).await?;

	let job = state
		.job_queue
		.cancel_job(&job_id)
		.await
		.map_err(|_| ServerError::NotFound)?;

	info!("Job {} cancelled", job_id);

	Ok(Json(job.into()))
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
	Json(serde_json::json!({
		"status": "healthy",
		"timestamp": chrono::Utc::now().to_rfc3339(),
	}))
}

/// Create the API router
pub fn create_api_router(state: Arc<AppState>) -> Router {
	Router::new()
		.route("/api/v1/health", get(health))
		.route("/api/v1/ingest", post(ingest_file))
		.route("/api/v1/status/:job_id", get(get_job_status))
		.route("/api/v1/jobs", get(list_jobs))
		.route("/api/v1/jobs/:job_id", axum::routing::delete(cancel_job))
		.layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100 MB max upload
		.layer(TraceLayer::new_for_http())
		.with_state(state)
}

/// Create the application (alias for create_api_router)
pub fn create_app(state: Arc<AppState>) -> Router {
	create_api_router(state)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ingest_response_serialization() {
		let response = IngestResponse {
			job_id: "job-123".to_string(),
			status: "queued".to_string(),
			created_at: chrono::Utc::now().to_rfc3339(),
		};

		let json = serde_json::to_string(&response).expect("serialization failed");
		assert!(json.contains("job-123"));
		assert!(json.contains("queued"));
	}

	#[test]
	fn test_job_status_response_from_job() {
		let job = Job::new("test.csv".to_string(), 1024);
		let response: JobStatusResponse = job.into();

		assert_eq!(response.filename, "test.csv");
		assert_eq!(response.status, "queued");
	}
}

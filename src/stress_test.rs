//! Stress test utility for Dumptruck server
//!
//! Submits all test fixtures to the server in parallel and measures:
//! - Throughput (requests/second)
//! - Latency (min/max/avg/p95/p99)
//! - Concurrent request handling
//! - Error rates

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Statistics collected during stress test
#[derive(Debug, Clone)]
struct TestStats {
	total_requests: usize,
	successful_requests: usize,
	failed_requests: usize,
	start_time: Instant,
	end_time: Option<Instant>,
	latencies: Vec<Duration>,
}

impl TestStats {
	fn new() -> Self {
		Self {
			total_requests: 0,
			successful_requests: 0,
			failed_requests: 0,
			start_time: Instant::now(),
			end_time: None,
			latencies: Vec::new(),
		}
	}

	fn finish(&mut self) {
		self.end_time = Some(Instant::now());
	}

	fn add_latency(&mut self, latency: Duration) {
		self.latencies.push(latency);
	}

	fn throughput(&self) -> f64 {
		let elapsed = self
			.end_time
			.unwrap_or_else(Instant::now)
			.duration_since(self.start_time);
		self.successful_requests as f64 / elapsed.as_secs_f64()
	}

	fn avg_latency(&self) -> Duration {
		if self.latencies.is_empty() {
			return Duration::from_secs(0);
		}
		let sum: Duration = self.latencies.iter().sum();
		sum / self.latencies.len() as u32
	}

	fn min_latency(&self) -> Duration {
		self.latencies.iter().copied().min().unwrap_or_default()
	}

	fn max_latency(&self) -> Duration {
		self.latencies.iter().copied().max().unwrap_or_default()
	}

	fn percentile_latency(&self, p: f64) -> Duration {
		if self.latencies.is_empty() {
			return Duration::from_secs(0);
		}
		let mut sorted = self.latencies.clone();
		sorted.sort();
		let index = ((p / 100.0) * sorted.len() as f64) as usize;
		sorted[index.min(sorted.len() - 1)]
	}
}

/// Test configuration
struct Config {
	server_url: String,
	oauth_token: String,
	concurrent_requests: usize,
	total_requests: usize,
	verbose: bool,
}

impl Config {
	fn from_env() -> Self {
		let server_url = std::env::var("STRESS_TEST_URL")
			.unwrap_or_else(|_| "https://localhost:8443".to_string());
		let oauth_token =
			std::env::var("STRESS_TEST_TOKEN").unwrap_or_else(|_| "test-token-12345".to_string());
		let concurrent_requests = std::env::var("STRESS_TEST_CONCURRENT")
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(50);
		let total_requests = std::env::var("STRESS_TEST_REQUESTS")
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(1000);
		let verbose = std::env::var("STRESS_TEST_VERBOSE").is_ok();

		Self {
			server_url,
			oauth_token,
			concurrent_requests,
			total_requests,
			verbose,
		}
	}
}

/// Load all test fixture files
fn load_test_fixtures() -> Vec<(String, Vec<u8>)> {
	let fixture_dir = Path::new("tests/fixtures");
	let mut fixtures = Vec::new();

	if let Ok(entries) = std::fs::read_dir(fixture_dir) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.is_file()
				&& let Ok(content) = std::fs::read(&path)
			{
				let filename = path
					.file_name()
					.and_then(|n| n.to_str())
					.unwrap_or("unknown")
					.to_string();
				fixtures.push((filename, content));
			}
		}
	}

	fixtures
}

/// Send a single ingest request
async fn send_ingest_request(
	client: &reqwest::Client,
	server_url: &str,
	oauth_token: &str,
	filename: &str,
	file_size: u64,
) -> Result<String, String> {
	use serde_json::json;

	let url = format!("{}/api/v1/ingest", server_url);
	let body = json!({
		"filename": filename,
		"file_size_bytes": file_size,
	});

	match client
		.post(&url)
		.header("Authorization", format!("Bearer {}", oauth_token))
		.json(&body)
		.send()
		.await
	{
		Ok(response) => {
			if response.status().is_success() {
				match response.json::<serde_json::Value>().await {
					Ok(data) => {
						if let Some(job_id) = data.get("job_id").and_then(|j| j.as_str()) {
							Ok(job_id.to_string())
						} else {
							Err("No job_id in response".to_string())
						}
					}
					Err(e) => Err(format!("Failed to parse response: {}", e)),
				}
			} else {
				Err(format!("Server returned {}", response.status()))
			}
		}
		Err(e) => Err(format!("Request failed: {}", e)),
	}
}

/// Check job status
pub async fn check_job_status(
	client: &reqwest::Client,
	server_url: &str,
	oauth_token: &str,
	job_id: &str,
) -> Result<String, String> {
	let url = format!("{}/api/v1/status/{}", server_url, job_id);

	match client
		.get(&url)
		.header("Authorization", format!("Bearer {}", oauth_token))
		.send()
		.await
	{
		Ok(response) => {
			if response.status().is_success() {
				match response.json::<serde_json::Value>().await {
					Ok(data) => {
						if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
							Ok(status.to_string())
						} else {
							Err("No status in response".to_string())
						}
					}
					Err(e) => Err(format!("Failed to parse response: {}", e)),
				}
			} else {
				Err(format!("Server returned {}", response.status()))
			}
		}
		Err(e) => Err(format!("Request failed: {}", e)),
	}
}

/// Run the stress test
async fn run_stress_test(config: Config) -> Result<(), Box<dyn std::error::Error>> {
	println!("=== Dumptruck Server Stress Test ===");
	println!("Server URL: {}", config.server_url);
	println!("Concurrent Requests: {}", config.concurrent_requests);
	println!("Total Requests Target: {}", config.total_requests);
	println!();

	let fixtures = load_test_fixtures();
	println!("Loaded {} test fixtures", fixtures.len());
	if config.verbose {
		for (name, content) in &fixtures {
			println!("  - {} ({} bytes)", name, content.len());
		}
	}
	println!();

	// Create HTTP client that ignores TLS verification (for testing)
	let client = reqwest::Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?;

	let stats = Arc::new(Mutex::new(TestStats::new()));
	let mut handles = Vec::new();

	// Submit requests in batches
	let mut fixture_cycle = fixtures.iter().cycle();
	let fixture_count = fixtures.len();

	for request_num in 0..config.total_requests {
		// Wait for a slot to become available if we have too many concurrent requests
		if handles.len() >= config.concurrent_requests
			&& let Ok(_) = tokio::select! {
				result = handles.remove(0) => result,
			} {
			// Handle completed task
		}

		let (filename, _content) = fixture_cycle.next().unwrap();
		let filename = filename.clone();
		let file_size = _content.len() as u64;

		let client = client.clone();
		let server_url = config.server_url.clone();
		let oauth_token = config.oauth_token.clone();
		let stats = stats.clone();
		let verbose = config.verbose;

		let handle = tokio::spawn(async move {
			let start = Instant::now();
			let fixture_idx = request_num % fixture_count;

			match send_ingest_request(&client, &server_url, &oauth_token, &filename, file_size)
				.await
			{
				Ok(job_id) => {
					let latency = start.elapsed();
					let mut s = stats.lock().await;
					s.total_requests += 1;
					s.successful_requests += 1;
					s.add_latency(latency);

					if verbose {
						println!(
							"[{}] OK - {} → job_id: {} ({:.2}ms)",
							fixture_idx,
							filename,
							job_id,
							latency.as_secs_f64() * 1000.0
						);
					}
				}
				Err(e) => {
					let latency = start.elapsed();
					let mut s = stats.lock().await;
					s.total_requests += 1;
					s.failed_requests += 1;
					s.add_latency(latency);

					if verbose {
						eprintln!(
							"[{}] FAIL - {} - {} ({:.2}ms)",
							fixture_idx,
							filename,
							e,
							latency.as_secs_f64() * 1000.0
						);
					}
				}
			}
		});

		handles.push(handle);

		// Small delay between requests to avoid overwhelming the server immediately
		if request_num % 10 == 0 {
			tokio::time::sleep(Duration::from_millis(10)).await;
		}
	}

	// Wait for all remaining requests to complete
	println!("Waiting for {} pending requests...", handles.len());
	for handle in handles {
		let _ = handle.await;
	}

	let mut final_stats = stats.lock().await;
	final_stats.finish();

	// Print results
	println!("\n=== Results ===");
	println!("Total Requests: {}", final_stats.total_requests);
	println!(
		"Successful: {} ({:.1}%)",
		final_stats.successful_requests,
		(final_stats.successful_requests as f64 / final_stats.total_requests as f64) * 100.0
	);
	println!(
		"Failed: {} ({:.1}%)",
		final_stats.failed_requests,
		(final_stats.failed_requests as f64 / final_stats.total_requests as f64) * 100.0
	);
	println!();
	println!(
		"Throughput: {:.2} requests/second",
		final_stats.throughput()
	);
	println!();
	println!("Latency (ms):");
	println!(
		"  Min:  {:.2}",
		final_stats.min_latency().as_secs_f64() * 1000.0
	);
	println!(
		"  Avg:  {:.2}",
		final_stats.avg_latency().as_secs_f64() * 1000.0
	);
	println!(
		"  P95:  {:.2}",
		final_stats.percentile_latency(95.0).as_secs_f64() * 1000.0
	);
	println!(
		"  P99:  {:.2}",
		final_stats.percentile_latency(99.0).as_secs_f64() * 1000.0
	);
	println!(
		"  Max:  {:.2}",
		final_stats.max_latency().as_secs_f64() * 1000.0
	);

	let elapsed = final_stats
		.end_time
		.unwrap()
		.duration_since(final_stats.start_time);
	println!();
	println!("Total Time: {:.2}s", elapsed.as_secs_f64());

	Ok(())
}

#[tokio::main]
async fn main() {
	let config = Config::from_env();

	if let Err(e) = run_stress_test(config).await {
		eprintln!("Stress test failed: {}", e);
		std::process::exit(1);
	}
}

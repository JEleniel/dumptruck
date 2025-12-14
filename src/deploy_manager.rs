use std::io;

use tokio::{
	process::Command,
	time::{Duration, Instant, sleep},
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
const STARTUP_RETRY_DELAY: Duration = Duration::from_secs(2);
const DB_CONTAINER: &str = "dumptruck-db";
const OLLAMA_CONTAINER: &str = "dumptruck-ollama";
const DB_PORT: u16 = 5432;
const OLLAMA_PORT: u16 = 11434;

/// Tracks which services were started by the application
pub struct ServiceManager {
	started_containers: Vec<String>,
}

impl ServiceManager {
	/// Create a new service manager
	pub fn new() -> Self {
		Self {
			started_containers: Vec::new(),
		}
	}

	/// Check and start required services (PostgreSQL and Ollama)
	pub async fn ensure_services_running(&mut self, verbose: u32) -> Result<(), String> {
		// Check and start PostgreSQL
		if !is_service_running("localhost", DB_PORT, verbose).await {
			if verbose >= 1 {
				eprintln!("[INFO] PostgreSQL not available, starting containers...");
			}
			self.start_containers(verbose).await?;
			self.started_containers.push(DB_CONTAINER.to_string());
		} else if verbose >= 2 {
			eprintln!("[DEBUG] PostgreSQL is already running");
		}

		// Check and start Ollama
		if !is_service_running("localhost", OLLAMA_PORT, verbose).await {
			if verbose >= 1 {
				eprintln!("[INFO] Ollama not available, starting containers...");
			}
			if !self.started_containers.iter().any(|c| c == DB_CONTAINER) {
				// Only start if we haven't already
				self.start_containers(verbose).await?;
			}
			self.started_containers.push(OLLAMA_CONTAINER.to_string());
		} else if verbose >= 2 {
			eprintln!("[DEBUG] Ollama is already running");
		}

		// Wait for both services to be ready
		if !self.started_containers.is_empty() {
			if verbose >= 2 {
				eprintln!("[DEBUG] Waiting for services to be ready...");
			}
			self.wait_for_services_ready(verbose).await?;
		}

		Ok(())
	}

	/// Check if a service is running by attempting a TCP connection
	async fn is_service_available(host: &str, port: u16) -> bool {
		match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
			Ok(_) => true,
			Err(_) => false,
		}
	}

	/// Start individual docker compose stacks from their respective directories
	async fn start_containers(&self, verbose: u32) -> Result<(), String> {
		if verbose >= 2 {
			eprintln!("[DEBUG] Attempting to start containers with docker compose from subdirectories");
		}

		// Start PostgreSQL from docker/postgres/
		if verbose >= 1 {
			eprintln!("[INFO] Starting PostgreSQL from docker/postgres/");
		}
		self.start_service("docker/postgres", "postgres", verbose).await?;

		// Start Ollama from docker/ollama/
		if verbose >= 1 {
			eprintln!("[INFO] Starting Ollama from docker/ollama/");
		}
		self.start_service("docker/ollama", "ollama", verbose).await?;

		if verbose >= 1 {
			eprintln!("[INFO] All services started successfully");
		}
		Ok(())
	}

	/// Start a docker compose service from its directory
	async fn start_service(&self, dir: &str, service: &str, verbose: u32) -> Result<(), String> {
		let up_cmds = [
			("docker", vec!["compose", "up", "--build", "-d"]),
			("docker-compose", vec!["up", "--build", "-d"]),
		];

		let mut last_err: Option<String> = None;
		for (bin, args) in up_cmds.iter() {
			let mut cmd = Command::new(bin);
			cmd.current_dir(dir);
			cmd.args(args);

			if verbose >= 2 {
				eprintln!("[DEBUG] Running: {} {} (in {})", bin, args.join(" "), dir);
			}

			match cmd.status().await {
				Ok(st) if st.success() => {
					if verbose >= 2 {
						eprintln!("[DEBUG] {} service started successfully", service);
					}
					return Ok(());
				}
				Ok(st) => {
					last_err = Some(format!("{} exited with {}", bin, st));
				}
				Err(e) => {
					last_err = Some(format!("{}: {}", bin, e));
				}
			}
		}

		Err(last_err.unwrap_or_else(|| format!("docker compose up failed for {}", service)))
	}

	/// Wait for all services to be ready
	async fn wait_for_services_ready(&self, verbose: u32) -> Result<(), String> {
		let deadline = Instant::now() + STARTUP_TIMEOUT;

		loop {
			let pg_ready = Self::is_service_available("localhost", DB_PORT).await;
			let ollama_ready = Self::is_service_available("localhost", OLLAMA_PORT).await;

			if pg_ready && ollama_ready {
				if verbose >= 1 {
					eprintln!("[INFO] All services are ready");
				}
				return Ok(());
			}

			if Instant::now() > deadline {
				let pg_status = if pg_ready { "ready" } else { "not ready" };
				let ollama_status = if ollama_ready { "ready" } else { "not ready" };
				return Err(format!(
					"Services failed to start within timeout. PostgreSQL: {}, Ollama: {}",
					pg_status, ollama_status
				));
			}

			if verbose >= 3 {
				let pg_status = if pg_ready { "ready" } else { "not ready" };
				let ollama_status = if ollama_ready { "ready" } else { "not ready" };
				eprintln!("[DEBUG] Service status - PostgreSQL: {}, Ollama: {}", pg_status, ollama_status);
			}

			sleep(STARTUP_RETRY_DELAY).await;
		}
	}

	/// Stop containers if they were started by this manager
	pub async fn stop_started_containers(&self, verbose: u32) -> Result<(), String> {
		if self.started_containers.is_empty() {
			if verbose >= 2 {
				eprintln!("[DEBUG] No containers to stop");
			}
			return Ok(());
		}

		if verbose >= 1 {
			eprintln!("[INFO] Stopping {} container(s) that were started", self.started_containers.len());
		}

		// Stop PostgreSQL if it was started
		if self.started_containers.iter().any(|c| c == DB_CONTAINER) {
			if verbose >= 2 {
				eprintln!("[DEBUG] Stopping PostgreSQL service");
			}
			let _ = self.stop_service("docker/postgres", "postgres", verbose).await;
		}

		// Stop Ollama if it was started
		if self.started_containers.iter().any(|c| c == OLLAMA_CONTAINER) {
			if verbose >= 2 {
				eprintln!("[DEBUG] Stopping Ollama service");
			}
			let _ = self.stop_service("docker/ollama", "ollama", verbose).await;
		}

		if verbose >= 1 {
			eprintln!("[INFO] All services stopped successfully");
		}
		Ok(())
	}

	/// Stop all known services (PostgreSQL and Ollama) regardless of tracking
	/// Used for graceful shutdown when containers may have been started outside this session
	pub async fn stop_all_services(&self, verbose: u32) -> Result<(), String> {
		if verbose >= 2 {
			eprintln!("[DEBUG] Stopping all known docker services (PostgreSQL and Ollama)");
		}

		// Stop PostgreSQL
		if verbose >= 2 {
			eprintln!("[DEBUG] Stopping PostgreSQL service");
		}
		let pg_result = self.stop_service("docker/postgres", "postgres", verbose).await;

		// Stop Ollama
		if verbose >= 2 {
			eprintln!("[DEBUG] Stopping Ollama service");
		}
		let ollama_result = self.stop_service("docker/ollama", "ollama", verbose).await;

		// Report results
		if pg_result.is_err() && verbose >= 2 {
			eprintln!("[DEBUG] PostgreSQL service stop failed or not running (this is OK)");
		}
		if ollama_result.is_err() && verbose >= 2 {
			eprintln!("[DEBUG] Ollama service stop failed or not running (this is OK)");
		}

		// Succeed even if individual services fail to stop (they may not be running)
		Ok(())
	}

	/// Stop a docker compose service in its directory
	async fn stop_service(&self, dir: &str, service: &str, verbose: u32) -> Result<(), String> {
		let down_cmds = [
			("docker", vec!["compose", "down", "-v"]),
			("docker-compose", vec!["down", "-v"]),
		];

		let mut last_err: Option<String> = None;
		for (bin, args) in down_cmds.iter() {
			let mut cmd = Command::new(bin);
			cmd.current_dir(dir);
			cmd.args(args);

			if verbose >= 2 {
				eprintln!("[DEBUG] Running: {} {} (in {})", bin, args.join(" "), dir);
			}

			match cmd.status().await {
				Ok(st) if st.success() => {
					if verbose >= 2 {
						eprintln!("[DEBUG] {} service stopped successfully", service);
					}
					return Ok(());
				}
				Ok(st) => {
					last_err = Some(format!("{} exited with {}", bin, st));
				}
				Err(e) => {
					last_err = Some(format!("{}: {}", bin, e));
				}
			}
		}

		Err(last_err.unwrap_or_else(|| format!("docker compose down failed for {}", service)))
	}
}

/// Check if a service is running on the given host and port
async fn is_service_running(host: &str, port: u16, verbose: u32) -> bool {
	if verbose >= 3 {
		eprintln!("[DEBUG] Checking if service is running on {}:{}", host, port);
	}
	ServiceManager::is_service_available(host, port).await
}

/// Start the docker compose stacks from their respective directories and wait until services are ready.
pub async fn start() -> Result<(), io::Error> {
	// Start PostgreSQL from docker/postgres/
	let up_cmds = [
		("docker", vec!["compose", "up", "--build", "-d"]),
		("docker-compose", vec!["up", "--build", "-d"]),
	];

	let mut last_err: Option<io::Error> = None;

	// Start PostgreSQL
	for (bin, args) in up_cmds.iter() {
		let status = Command::new(bin)
			.current_dir("docker/postgres")
			.args(args)
			.status()
			.await;

		match status {
			Ok(st) if st.success() => {
				// wait for postgres inside container to become ready
				if wait_for_pg_ready().await {
					// Now start Ollama
					for (bin2, args2) in up_cmds.iter() {
						match Command::new(bin2)
							.current_dir("docker/ollama")
							.args(args2)
							.status()
							.await
						{
							Ok(st2) if st2.success() => return Ok(()),
							Ok(st2) => {
								last_err = Some(io::Error::new(
									io::ErrorKind::Other,
									format!("{} exited with {}", bin2, st2),
								));
							}
							Err(e) => {
								last_err = Some(e);
							}
						}
					}
					return Err(last_err.unwrap_or_else(|| io::Error::new(
						io::ErrorKind::Other,
						"ollama service failed to start",
					)));
				} else {
					return Err(io::Error::new(
						io::ErrorKind::TimedOut,
						"postgres did not become ready",
					));
				}
			}
			Ok(st) => {
				last_err = Some(io::Error::new(
					io::ErrorKind::Other,
					format!("{} exited with {}", bin, st),
				));
			}
			Err(e) => {
				last_err = Some(e);
			}
		}
	}

	Err(last_err
		.unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "docker compose up failed")))
}

async fn wait_for_pg_ready() -> bool {
	let deadline = Instant::now() + STARTUP_TIMEOUT;
	// container name as defined in docker-compose.yml
	let container = "dumptruck-db";

	while Instant::now() < deadline {
		// Use `docker exec` to call `pg_isready` inside container
		let status = Command::new("docker")
			.args(["exec", container, "pg_isready", "-U", "dumptruck"])
			.status()
			.await;

		match status {
			Ok(st) if st.success() => return true,
			_ => {
				sleep(STARTUP_RETRY_DELAY).await;
			}
		}
	}

	false
}

/// Stop and remove the docker compose stack.
pub async fn stop() -> Result<(), io::Error> {
	let down_cmds = [
		("docker", vec!["compose", "down", "-v"]),
		("docker-compose", vec!["down", "-v"]),
	];

	let mut last_err: Option<io::Error> = None;
	for (bin, args) in down_cmds.iter() {
		let status = Command::new(bin).args(args).status().await;
		match status {
			Ok(st) if st.success() => return Ok(()),
			Ok(st) => {
				last_err = Some(io::Error::new(
					io::ErrorKind::Other,
					format!("{} exited with {}", bin, st),
				))
			}
			Err(e) => last_err = Some(e),
		}
	}

	Err(last_err
		.unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "docker compose down failed")))
}

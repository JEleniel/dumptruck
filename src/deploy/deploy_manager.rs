use std::io;

use tokio::{
	process::Command,
	time::{Duration, Instant, sleep},
};

use crate::core::config::Config;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
const STARTUP_RETRY_DELAY: Duration = Duration::from_secs(2);
const OLLAMA_CONTAINER: &str = "dumptruck-ollama";
const OLLAMA_PORT: u16 = 11435;

/// Tracks which services were started by the application
#[derive(Default)]
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

	/// Check and start services based on configuration
	///
	/// Ollama only starts if enabled in config.
	pub async fn ensure_services_running(
		&mut self,
		verbose: u32,
		config: Option<&Config>,
	) -> Result<(), String> {
		// Determine if Ollama should be started
		let start_ollama = config.map(|c| c.ollama_enabled()).unwrap_or(false);

		// Check and start Ollama only if enabled
		if start_ollama {
			if !is_service_running("localhost", OLLAMA_PORT, verbose).await {
				if verbose >= 1 {
					eprintln!("[INFO] Ollama not available, starting Ollama...");
				}
				self.start_service("docker/ollama", "ollama", verbose)
					.await?;
				self.started_containers.push(OLLAMA_CONTAINER.to_string());
			} else if verbose >= 2 {
				eprintln!("[DEBUG] Ollama is already running");
			}
		} else if verbose >= 2 {
			eprintln!("[DEBUG] Ollama is disabled in configuration, skipping startup");
		}

		// Wait for services to be ready
		if !self.started_containers.is_empty() {
			if verbose >= 2 {
				eprintln!("[DEBUG] Waiting for services to be ready...");
			}
			self.wait_for_services_ready(verbose, start_ollama).await?;
		}

		Ok(())
	}

	/// Check if a service is running by attempting a TCP connection
	async fn is_service_available(host: &str, port: u16) -> bool {
		tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await.is_ok()
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

	/// Wait for services to be ready based on which services are being started
	async fn wait_for_services_ready(
		&self,
		verbose: u32,
		check_ollama: bool,
	) -> Result<(), String> {
		let deadline = Instant::now() + STARTUP_TIMEOUT;
		let mut attempt = 0;

		loop {
			let ollama_ready = if check_ollama {
				Self::is_service_available("localhost", OLLAMA_PORT).await
			} else {
				true // Skip Ollama check if not enabled
			};

			if ollama_ready {
				if verbose >= 1 {
					eprintln!("[INFO] All services are ready");
				}
				return Ok(());
			}

			if Instant::now() > deadline {
				let ollama_status = if check_ollama {
					if ollama_ready { "ready" } else { "not ready" }
				} else {
					"skipped"
				};
				return Err(format!(
					"Services failed to start within timeout. Ollama: {}",
					ollama_status
				));
			}

			attempt += 1;
			if verbose >= 1 {
				let ollama_status = if check_ollama {
					if ollama_ready { "ready" } else { "waiting" }
				} else {
					"disabled"
				};
				eprintln!(
					"[INFO] Waiting for services... (attempt {}). Ollama: {}",
					attempt, ollama_status
				);
			} else if verbose >= 2 {
				let ollama_status = if check_ollama {
					if ollama_ready { "ready" } else { "not ready" }
				} else {
					"disabled"
				};
				eprintln!("[DEBUG] Service status - Ollama: {}", ollama_status);
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
			eprintln!(
				"[INFO] Stopping {} container(s) that were started",
				self.started_containers.len()
			);
		}

		// Stop Ollama if it was started
		if self
			.started_containers
			.iter()
			.any(|c| c == OLLAMA_CONTAINER)
		{
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

	/// Stop all known services (Ollama only) regardless of tracking
	/// Used for graceful shutdown when containers may have been started outside this session
	pub async fn stop_all_services(&self, verbose: u32) -> Result<(), String> {
		if verbose >= 2 {
			eprintln!("[DEBUG] Stopping all known docker services (Ollama)");
		}

		// Stop Ollama
		if verbose >= 2 {
			eprintln!("[DEBUG] Stopping Ollama service");
		}
		let ollama_result = self.stop_service("docker/ollama", "ollama", verbose).await;

		// Report results
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
		eprintln!(
			"[DEBUG] Checking if service is running on {}:{}",
			host, port
		);
	}
	ServiceManager::is_service_available(host, port).await
}

/// Start the docker compose stack for Ollama from its directory.
pub async fn start() -> Result<(), io::Error> {
	let up_cmds = [
		("docker", vec!["compose", "up", "--build", "-d"]),
		("docker-compose", vec!["up", "--build", "-d"]),
	];

	let mut last_err: Option<io::Error> = None;

	// Start Ollama
	for (bin, args) in up_cmds.iter() {
		let status = Command::new(bin)
			.current_dir("docker/ollama")
			.args(args)
			.status()
			.await;

		match status {
			Ok(st) if st.success() => return Ok(()),
			Ok(st) => {
				last_err = Some(io::Error::other(
					format!("{} exited with {}", bin, st),
				));
			}
			Err(e) => {
				last_err = Some(e);
			}
		}
	}

	Err(last_err
		.unwrap_or_else(|| io::Error::other("docker compose up failed")))
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
				last_err = Some(io::Error::other(
					format!("{} exited with {}", bin, st),
				))
			}
			Err(e) => last_err = Some(e),
		}
	}

	Err(last_err
		.unwrap_or_else(|| io::Error::other("docker compose down failed")))
}

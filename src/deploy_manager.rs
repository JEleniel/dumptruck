use std::io;

use tokio::{
	process::Command,
	time::{Duration, Instant, sleep},
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
const STARTUP_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Start the docker compose stack under `docker/` and wait until Postgres is ready.
pub async fn start() -> Result<(), io::Error> {
	// Try to bring up the compose stack
	let up_cmds = [
		("docker", vec!["compose", "up", "--build", "-d"]),
		("docker-compose", vec!["up", "--build", "-d"]),
	];

	let mut last_err: Option<io::Error> = None;
	for (bin, args) in up_cmds.iter() {
		let status = Command::new(bin).args(args).status().await;

		match status {
			Ok(st) if st.success() => {
				// wait for postgres inside container to become ready
				if wait_for_pg_ready().await {
					return Ok(());
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

use std::os::linux::process;

#[tokio::main]
async fn main() {
	if let Err(e) = dumptruck::run().await {
		error!(
			"An unrecoverable error has occurred and the application will exit: {}",
			e
		);
		process::exit(1);
	}
}

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn prints_help_without_command() {
	// Use the cargo_bin macro to avoid deprecated API warnings when
	// the workspace uses a custom build directory.
	let bin = assert_cmd::cargo::cargo_bin!("dumptruck");
	let mut cmd = Command::new(bin);
	// Tests should not manage docker. Disable lifecycle management for test runs.
	cmd.env("DUMPTRUCK_MANAGE_DOCKER", "0");
	cmd.assert()
		.failure()
		.stderr(predicate::str::contains("Usage:").or(predicate::str::contains("Commands:")));
}

#[test]
fn ingest_command_with_valid_file() {
	let bin = assert_cmd::cargo::cargo_bin!("dumptruck");
	let mut cmd = Command::new(bin);
	cmd.env("DUMPTRUCK_MANAGE_DOCKER", "0");
	cmd.arg("ingest")
		.arg("tests/fixtures/well_formed_credentials.csv")
		.arg("--output-format")
		.arg("text");
	cmd.assert()
		.success()
		.stdout(predicate::str::contains("Dumptruck Analysis Results"));
}

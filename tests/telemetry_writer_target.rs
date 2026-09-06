//! Proves `telemetry::init_with_writer` actually targets the writer it is
//! given, and that `telemetry::init`'s default target is unchanged (stdout).
//!
//! Own file, self-reexec per test: `try_init()` sets the global subscriber
//! once per process, and the property under test — which real OS stream a
//! log line lands on — is only observable from outside the process whose
//! stdout/stderr file descriptors are being written to. Each test spawns a
//! fresh child process (`std::env::current_exe()`, filtered to just that
//! test) and inspects the child's captured stdout/stderr pipes directly,
//! rather than anything Rust's own test-harness capture does or does not
//! intercept.
#![cfg(feature = "telemetry")]

use std::env;
use std::process::Command;

const MARKER: &str = "koinon-telemetry-writer-target-marker";
const CHILD_ENV_VAR: &str = "KOINON_TELEMETRY_WRITER_TEST_CHILD";

#[expect(
    clippy::expect_used,
    reason = "test-only self-reexec helper: current_exe()/spawning the same already-running test binary has no recoverable failure mode worth a Result return, and this file is never compiled into the published library"
)]
fn run_child(test_name: &str, child_env_value: &str) -> (String, String) {
    let exe = env::current_exe().expect("test binary has a current_exe path");
    let output = Command::new(exe)
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_ENV_VAR, child_env_value)
        .output()
        .expect("spawning the self-reexec child process");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn stderr_writer_receives_events_and_stdout_stays_empty() {
    if let Ok(mode) = env::var(CHILD_ENV_VAR) {
        assert_eq!(mode, "stderr_writer");
        koinon::telemetry::init_with_writer("info", std::io::stderr);
        tracing::info!(marker = MARKER, "writer target marker");
        return;
    }

    let (stdout, stderr) = run_child(
        "stderr_writer_receives_events_and_stdout_stays_empty",
        "stderr_writer",
    );
    assert!(
        stderr.contains(MARKER),
        "init_with_writer(_, io::stderr) must send events to stderr; got stderr={stderr:?}"
    );
    assert!(
        !stdout.contains(MARKER),
        "init_with_writer(_, io::stderr) must not leak events onto stdout; got stdout={stdout:?}"
    );
}

#[test]
fn default_init_still_targets_stdout_unchanged() {
    if let Ok(mode) = env::var(CHILD_ENV_VAR) {
        assert_eq!(mode, "default_init");
        koinon::telemetry::init("info");
        tracing::info!(marker = MARKER, "writer target marker");
        return;
    }

    let (stdout, stderr) = run_child(
        "default_init_still_targets_stdout_unchanged",
        "default_init",
    );
    assert!(
        stdout.contains(MARKER),
        "init()'s default writer target must remain stdout; got stdout={stdout:?}"
    );
    assert!(
        !stderr.contains(MARKER),
        "init() must not write events to stderr; got stderr={stderr:?}"
    );
}

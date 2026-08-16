//! External-consumer fixture for [`koinon::bootstrap::run`]: proves the
//! integrated policy and lifecycle koinon#28 asks for — CLI/environment
//! precedence, config loading, and telemetry initialization resolving
//! through one call — rather than only that `cli`, `config`, and
//! `telemetry` each compile in isolation. Written entirely against the
//! public API, the way a real fleet binary would call it.
//!
//! Scope note: `Bootstrap::verbosity()`/`log_json()`, asserted below, are
//! pure getters over the already-parsed `GlobalArgs` — they prove the CLI
//! leaf's precedence resolved correctly, not that telemetry was actually
//! initialized with it. That leg is `tests/bootstrap_telemetry_activation.rs`,
//! in its own process because it observes the real global subscriber.
#![cfg(feature = "bootstrap")]

use clap::Parser;
use koinon::bootstrap;
use koinon::cli::{GlobalArgs, Verbosity};
use koinon::error::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    port: u16,
}

#[test]
fn run_composes_cli_precedence_config_loading_and_telemetry_in_one_call() {
    figment::Jail::expect_with(|jail| {
        jail.create_file("app.toml", "port = 9090")?;
        let cli = Cli::parse_from(["app", "-vv"]);

        let boot: bootstrap::Bootstrap<AppConfig> =
            bootstrap::run(&cli.global, "app.toml", "KOINON_CONFORMANCE", "info")
                .map_err(|e| figment::Error::from(e.to_string()))?;

        assert_eq!(
            boot.config.port, 9090,
            "config leaf loaded through the sequence"
        );
        assert_eq!(
            boot.verbosity(),
            Verbosity::Trace,
            "the CLI leaf's -vv resolved through the sequence, not re-derived from config"
        );
        assert!(!boot.log_json());
        Ok(())
    });
}

#[test]
fn run_is_idempotent_with_identical_input() {
    // WHY: bootstrap::run is a state-modifying operation (it initializes a
    // process-global tracing subscriber), so TESTING.md's idempotency
    // pattern applies: call it, capture resulting state, call it again with
    // IDENTICAL input, assert the state is unchanged.
    figment::Jail::expect_with(|jail| {
        jail.create_file("app.toml", "port = 7")?;
        let cli = Cli::parse_from(["app", "-v"]);

        let first: bootstrap::Bootstrap<AppConfig> =
            bootstrap::run(&cli.global, "app.toml", "KOINON_CONFORMANCE_IDEMP", "info")
                .map_err(|e| figment::Error::from(e.to_string()))?;
        let second: bootstrap::Bootstrap<AppConfig> =
            bootstrap::run(&cli.global, "app.toml", "KOINON_CONFORMANCE_IDEMP", "info")
                .map_err(|e| figment::Error::from(e.to_string()))?;

        assert_eq!(
            first.config.port, second.config.port,
            "identical input must produce identical config across repeated calls"
        );
        assert_eq!(
            first.verbosity(),
            second.verbosity(),
            "identical input must produce identical verbosity across repeated calls"
        );
        assert_eq!(
            first.log_json(),
            second.log_json(),
            "identical input must produce identical log_json across repeated calls"
        );
        Ok(())
    });
}

#[test]
fn run_reloads_config_on_each_call_rather_than_caching() {
    // WHY: a distinct property from idempotency above — this proves run()
    // does not memoize the first call's result, by giving it DIFFERENT
    // input (a rewritten config file) between calls and asserting the
    // second call observes the change.
    figment::Jail::expect_with(|jail| {
        jail.create_file("app.toml", "port = 1")?;
        let cli = Cli::parse_from(["app"]);

        let first: bootstrap::Bootstrap<AppConfig> =
            bootstrap::run(&cli.global, "app.toml", "KOINON_CONFORMANCE_RELOAD", "info")
                .map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(first.config.port, 1);

        jail.create_file("app.toml", "port = 2")?;
        let second: bootstrap::Bootstrap<AppConfig> =
            bootstrap::run(&cli.global, "app.toml", "KOINON_CONFORMANCE_RELOAD", "info")
                .map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(second.config.port, 2, "re-running must reload, not cache");
        Ok(())
    });
}

#[test]
fn run_surfaces_the_typed_config_error_it_owns() {
    // Negative-case fixture: a malformed config file must reach the caller
    // as the real ConfigError::Parse variant through the public bootstrap
    // API, proving the integration does not swallow or stringify the error
    // koinon's config leaf already produces.
    figment::Jail::expect_with(|jail| {
        jail.create_file("app.toml", "port = ")?;
        let cli = Cli::parse_from(["app"]);

        let result: Result<bootstrap::Bootstrap<AppConfig>, ConfigError> =
            bootstrap::run(&cli.global, "app.toml", "KOINON_CONFORMANCE_ERR", "info");

        match result {
            Err(ConfigError::Parse { .. }) => {}
            other => panic!("expected ConfigError::Parse, got {other:?}"),
        }
        Ok(())
    });
}

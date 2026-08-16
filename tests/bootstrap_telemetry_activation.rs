//! Proves `bootstrap::run` actually reaches the tracing subscriber it claims
//! to configure — the leg `bootstrap_conformance.rs` cannot observe.
//!
//! `Bootstrap::verbosity()`/`log_json()` are pure getters over the
//! already-parsed `GlobalArgs`, populated independently of whatever `run`
//! did with telemetry (`src/bootstrap.rs`). A regression that swaps
//! `global.init_tracing(default_directive)` for
//! `crate::telemetry::init(default_directive)` — silently dropping the
//! `-vv`/`RUST_LOG` resolution telemetry is supposed to receive — leaves
//! those getters, and every assertion in `bootstrap_conformance.rs`,
//! unchanged. This proves the property those cannot: that the process's
//! installed tracing filter reflects the CLI-resolved verbosity, not just
//! that `GlobalArgs::verbosity()` reports it correctly.
//!
//! Own file (own process): `telemetry::init`'s `try_init()` sets the global
//! subscriber once per process. A test sharing a process with any other
//! `bootstrap::run` caller would race for which installation wins.
#![cfg(feature = "bootstrap")]

use clap::Parser;
use koinon::bootstrap;
use koinon::cli::GlobalArgs;
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
fn run_wires_cli_verbosity_into_the_installed_subscriber() {
    figment::Jail::expect_with(|jail| {
        // WHY: RUST_LOG unset so the -vv flag, not the environment, is the
        // only source that can make the assertion below true.
        jail.set_env("RUST_LOG", "");
        jail.create_file("app.toml", "port = 1")?;
        let cli = Cli::parse_from(["app", "-vv"]);

        // default_directive is deliberately far below trace. If run()
        // reached telemetry through anything other than
        // GlobalArgs::init_tracing (the koinon#28 regression this fixture
        // exists to catch), the installed filter's max level would be
        // Error, not Trace, and the assertion below would fail.
        let _boot: bootstrap::Bootstrap<AppConfig> = bootstrap::run(
            &cli.global,
            "app.toml",
            "KOINON_TELEMETRY_ACTIVATION",
            "error",
        )
        .map_err(|e| figment::Error::from(e.to_string()))?;

        assert!(
            tracing::enabled!(tracing::Level::TRACE),
            "the installed subscriber must accept TRACE-level events after \
             -vv resolves to Verbosity::Trace — got a filter that still \
             reflects the low-verbosity default_directive param, meaning \
             run() never threaded -vv into telemetry initialization"
        );
        Ok(())
    });
}

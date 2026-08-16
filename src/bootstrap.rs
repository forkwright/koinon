//! The integrated application-bootstrap sequence — koinon's one invariant.
//!
//! [`crate::cli`], [`crate::config`], and [`crate::telemetry`] are leaves
//! this sequence composes, not three independently useful peer
//! conveniences. [`run`] is what makes them one thing: it resolves
//! `--verbose` / `RUST_LOG` precedence, initializes telemetry from that
//! resolution, loads a typed configuration through the same policy
//! (defaults → TOML file → env-var override), and — once both steps
//! succeed — emits a `tracing::info!` record of what was actually decided
//! (config path, env prefix, resolved verbosity, JSON mode). That record is
//! the startup evidence: proof the sequence ran as one unit, not a
//! post-hoc log line a caller remembered to add.
//!
//! Telemetry is initialized *before* config is loaded specifically so a
//! config failure is reported through the subscriber [`run`] just set up,
//! rather than racing a caller's own fallback logging.
//!
//! # Usage
//!
//! ```rust,no_run
//! use clap::Parser;
//! use koinon::bootstrap;
//! use koinon::cli::GlobalArgs;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     global: GlobalArgs,
//! }
//!
//! #[derive(Debug, Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     port: u16,
//! }
//!
//! # fn main() -> Result<(), koinon::error::ConfigError> {
//! let cli = Cli::parse();
//! let boot: bootstrap::Bootstrap<AppConfig> =
//!     bootstrap::run(&cli.global, "app.toml", "APP", "my_crate=info")?;
//! tracing::info!(port = boot.config.port, "listening");
//! # Ok(())
//! # }
//! ```

use serde::Deserialize;
use serde::Serialize;

use crate::cli::GlobalArgs;
use crate::cli::Verbosity;
use crate::config;
use crate::error::ConfigError;

/// Outcome of a successful [`run`]: the loaded configuration plus the
/// telemetry evidence resolved while loading it.
///
/// `T` is the caller's own config type — the same one passed to
/// [`crate::config::load`]. [`verbosity`][Self::verbosity] and
/// [`log_json`][Self::log_json] are not re-derived from `config`; they are
/// the exact values [`run`] used to initialize telemetry, so a caller
/// inspecting them sees what actually happened, not a value that could
/// drift from it — the same evidence-over-declaration shape as
/// [`GlobalArgs::verbosity`].
#[derive(Debug)]
#[non_exhaustive]
pub struct Bootstrap<T> {
    /// The typed configuration loaded via [`crate::config::load`].
    pub config: T,
    verbosity: Verbosity,
    log_json: bool,
}

impl<T> Bootstrap<T> {
    /// The verbosity level telemetry was initialized with.
    #[must_use]
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Whether telemetry was initialized in JSON mode.
    #[must_use]
    pub fn log_json(&self) -> bool {
        self.log_json
    }
}

/// Run the integrated bootstrap sequence.
///
/// Initializes telemetry from `global` (see [`GlobalArgs::init_tracing`]),
/// then loads `T` from `config_path` + `env_prefix` (see
/// [`crate::config::load`]), in that order. On success, emits one
/// `tracing::info!` event recording `config_path`, `env_prefix`, the
/// resolved verbosity, and `log_json`.
///
/// # Errors
///
/// Returns [`ConfigError`] exactly as [`crate::config::load`] would.
/// Config loading is the only step in this sequence that can fail:
/// [`GlobalArgs::verbosity`] is a pure function of an already-parsed flag
/// count, and telemetry initialization is a no-op after the first
/// successful call rather than a fallible one — see
/// [`crate::telemetry::init`].
// WHY: genuine cross-crate API; basanos' workspace-library exemption needs a
// [workspace] ancestor, which this standalone published crate does not have.
// kanon:ignore RUST/pub-visibility -- documented public bootstrap-sequence API
pub fn run<T>(
    global: &GlobalArgs,
    config_path: &str,
    env_prefix: &str,
    default_directive: &str,
) -> Result<Bootstrap<T>, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize + Default,
{
    global.init_tracing(default_directive);
    let config: T = config::load(config_path, env_prefix)?;

    let verbosity = global.verbosity();
    let log_json = global.log_json();
    tracing::info!(
        config_path,
        env_prefix,
        ?verbosity,
        log_json,
        "bootstrap complete"
    );

    Ok(Bootstrap {
        config,
        verbosity,
        log_json,
    })
}

// NOTE: run() has no private helper functions of its own to white-box test
// — it is a thin composition over cli/config/telemetry, each already unit
// tested in their own module. Its coverage lives entirely in
// tests/bootstrap_conformance.rs, exercised through the public API the way
// an external consumer would call it — the "external-consumer conformance
// fixture" this module exists to satisfy (koinon#28).

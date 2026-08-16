//! κοινόν — the typed application-bootstrap sequence for forkwright Rust
//! crates.
//!
//! [`bootstrap::run`] is koinon's one invariant: it resolves CLI/environment
//! verbosity precedence, initializes telemetry from that resolution, loads
//! a typed configuration through the same defaults → TOML → env-var policy,
//! and reports what it decided. [`telemetry`], [`config`], and [`cli`] are
//! the leaves that sequence composes:
//!
//! - [`cli`] — a `Verbosity` flag and `GlobalArgs` struct every CLI binary
//!   embeds via `#[command(flatten)]`, resolving `--verbose` / `RUST_LOG`
//!   precedence.
//! - [`telemetry`] — `tracing` subscriber initialization from that
//!   resolution.
//! - [`config`] — a `figment`-backed loader: defaults → TOML file →
//!   env-var override → typed struct.
//! - [`error`] — [`ConfigError`], the one error type koinon semantically
//!   owns. A binary's top-level error sum is not koinon's to define; it
//!   stays in the consumer, which wraps `ConfigError` into it.
//!
//! A crate that only needs one leaf — a library example initializing
//! telemetry with no config or CLI of its own, say — can still depend on
//! that module alone via `default-features = false`; [`bootstrap::run`] is
//! the integrated path, not the only door in.
//!
//! # Feature flags
//!
//! `telemetry`, `config`, and `cli` are Cargo features, all enabled by
//! default; `bootstrap` requires both `cli` and `config` and is enabled by
//! default alongside them. Downstream crates that only need a subset can
//! use `default-features = false` and re-enable individually; each feature
//! pulls in only its own dependencies. The [`error`] module is always
//! available and carries only the `snafu` dependency.
//!
//! | Feature | Enables |
//! |---------|---------|
//! | `telemetry` | [`telemetry`] module: `tracing` + `tracing-subscriber` |
//! | `config` | [`config`] module: `figment` + `serde` |
//! | `cli` | [`cli`] module: `clap` (implies `telemetry`) |
//! | `bootstrap` | [`bootstrap`] module (implies `cli` + `config`) |
//!
//! # Minimal usage
//!
//! Requires the default `bootstrap` feature; see [`bootstrap`] for the full
//! example with a config type and CLI struct.
//!
//! ```rust,no_run
//! use clap::Parser;
//! use koinon::bootstrap;
//! use koinon::cli::GlobalArgs;
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     global: GlobalArgs,
//! }
//!
//! #[derive(Debug, serde::Deserialize, serde::Serialize, Default)]
//! struct AppConfig {}
//!
//! # fn main() -> Result<(), koinon::error::ConfigError> {
//! let cli = Cli::parse();
//! let boot: bootstrap::Bootstrap<AppConfig> =
//!     bootstrap::run(&cli.global, "app.toml", "APP", "my_crate=info")?;
//! tracing::info!("ready");
//! # let _ = boot;
//! # Ok(())
//! # }
//! ```
//!
//! [`ConfigError`]: error::ConfigError

#![deny(missing_docs)]
#![deny(unsafe_code)]

#[cfg(feature = "bootstrap")]
pub mod bootstrap;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "config")]
pub mod config;
pub mod error;
#[cfg(feature = "telemetry")]
pub mod telemetry;

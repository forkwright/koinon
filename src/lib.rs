//! κοινόν — fleet-common scaffolding for forkwright Rust crates.
//!
//! Provides four shared concerns that every forkwright binary or library
//! needs, without forcing a monolithic dependency tree:
//!
//! - [`telemetry`] — `tracing` subscriber initialization with `RUST_LOG`
//!   / `EnvFilter` and an optional process-wide default directive.
//! - [`error`] — re-exported `snafu` items and a common
//!   [`AppError`]/[`ConfigError`] base type for binary crates.
//! - [`config`] — `figment`-backed loader: defaults → TOML file → env-var
//!   override → typed struct.
//! - [`cli`] — `clap`-based prelude: a `Verbosity` flag and a
//!   `GlobalArgs` struct that every CLI binary can embed.
//!
//! # Feature flags
//!
//! `telemetry`, `config`, and `cli` are Cargo features, all enabled by
//! default. Downstream crates that only need a subset can use
//! `default-features = false` and re-enable individually; each feature pulls
//! in only its own dependencies. The [`error`] module (and its `snafu`
//! re-exports) is always available and carries only the `snafu` dependency.
//!
//! | Feature | Enables |
//! |---------|---------|
//! | `telemetry` | [`telemetry`] module: `tracing` + `tracing-subscriber` |
//! | `config` | [`config`] module: `figment` + `serde` |
//! | `cli` | [`cli`] module: `clap` (implies `telemetry`) |
//!
//! # Minimal usage
//!
//! Requires the default `telemetry` feature:
//!
//! ```rust,no_run
//! use koinon::telemetry;
//!
//! telemetry::init("my_crate=info");
//! tracing::info!("ready");
//! ```
//!
//! [`AppError`]: error::AppError
//! [`ConfigError`]: error::ConfigError

#![deny(missing_docs)]
#![deny(unsafe_code)]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "config")]
pub mod config;
pub mod error;
#[cfg(feature = "telemetry")]
pub mod telemetry;

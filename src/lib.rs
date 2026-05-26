//! κοινόν — fleet-common scaffolding for forkwright Rust crates.
//!
//! Provides three shared concerns that every forkwright binary or library
//! needs, without forcing a monolithic dependency tree:
//!
//! - [`telemetry`] — `tracing` subscriber initialization with `RUST_LOG`
//!   / `EnvFilter` and an optional process-wide default directive.
//! - [`error`] — re-exported `snafu` items and a common
//!   [`AppError`]/[`ConfigError`] base type for binary crates.
//! - [`config`] — `figment`-backed loader: TOML file → env-var override →
//!   typed struct, with a `RUST_LOG`-style path convention.
//! - [`cli`] — `clap`-based prelude: a `Verbosity` flag and a
//!   `GlobalArgs` struct that every CLI binary can embed.
//!
//! # Feature flags
//!
//! All features are enabled by default. Downstream crates that only need a
//! subset can use `default-features = false` and re-enable individually.
//!
//! | Feature | Enables |
//! |---------|---------|
//! | `telemetry` | `tracing` + `tracing-subscriber` |
//! | `config` | `figment` + `serde` |
//! | `cli` | `clap` |
//!
//! # Minimal usage
//!
//! ```rust,no_run
//! use koinon::telemetry;
//!
//! telemetry::init("my_crate=info");
//! tracing::info!("ready");
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod cli;
pub mod config;
pub mod error;
pub mod telemetry;

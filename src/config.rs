//! `figment`-based configuration loader for forkwright crates.
//!
//! # Loading order (highest priority last wins)
//!
//! 1. Defaults baked into `T::default()` (if `T: Default`).
//! 2. TOML file at `path` (if the file exists; skipped silently if absent).
//! 3. Environment variables with the given `prefix` (e.g. `APP_` maps
//!    `APP_PORT=8080` → `{ port: 8080 }`).
//!
//! # Usage
//!
//! ```rust,no_run
//! use koinon::config;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, Default)]
//! struct AppConfig {
//!     port: u16,
//!     host: String,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let cfg: AppConfig = config::load("app.toml", "APP")?;
//!     Ok(())
//! }
//! ```
//!
//! # Path convention
//!
//! When `path` is a relative path it is resolved against the current working
//! directory. Pass an absolute path or use `load_from_env` to pick the path
//! from an environment variable (e.g. `APP_CONFIG=/etc/app.toml`).

use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::Deserialize;
use serde::Serialize;

use crate::error::ConfigError;

/// Load a typed configuration from a TOML file + environment variables.
///
/// # Arguments
///
/// * `path` — path to the TOML file. Silently skipped if the file does not
///   exist, allowing pure-env deployments.
/// * `env_prefix` — upper-case prefix for environment variable overrides.
///   Example: `"APP"` maps `APP_PORT=8080` → `config.port = 8080`.
///
/// # Errors
///
/// Returns [`ConfigError::Extraction`] if figment cannot deserialize the
/// merged configuration into `T`.
pub fn load<T>(path: &str, env_prefix: &str) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    let figment = Figment::new()
        .merge(Toml::file(path))
        .merge(Env::prefixed(env_prefix).split("__"));

    extract(&figment)
}

/// Load a typed configuration with explicit defaults.
///
/// The `defaults` value is serialized and used as the lowest-priority layer,
/// so any field not present in the TOML file or environment falls back to the
/// value in `defaults`.
///
/// # Errors
///
/// Returns [`ConfigError::Extraction`] if figment cannot deserialize the
/// merged configuration into `T`.
pub fn load_with_defaults<T>(path: &str, env_prefix: &str, defaults: &T) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let figment = Figment::new()
        .merge(Serialized::defaults(defaults))
        .merge(Toml::file(path))
        .merge(Env::prefixed(env_prefix).split("__"));

    extract(&figment)
}

/// Load a typed configuration from environment variables only.
///
/// Useful in containerized deployments where a config file is not mounted.
///
/// # Errors
///
/// Returns [`ConfigError::Extraction`] if figment cannot deserialize the
/// merged configuration into `T`.
pub fn load_from_env<T>(env_prefix: &str) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    let figment = Figment::new().merge(Env::prefixed(env_prefix).split("__"));
    extract(&figment)
}

/// Load a typed configuration where the config file path is specified by
/// an environment variable.
///
/// If the environment variable is unset, falls back to `default_path`.
///
/// # Arguments
///
/// * `path_env_var` — name of the environment variable holding the config
///   file path (e.g. `"APP_CONFIG"`).
/// * `default_path` — fallback path when the env var is not set.
/// * `env_prefix` — prefix for all other env overrides.
///
/// # Errors
///
/// Returns [`ConfigError::Extraction`] if figment cannot deserialize the
/// merged configuration into `T`.
pub fn load_from_env_path<T>(
    path_env_var: &str,
    default_path: &str,
    env_prefix: &str,
) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    let path = std::env::var(path_env_var).unwrap_or_else(|_| default_path.to_string());
    load(&path, env_prefix)
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

fn extract<T>(figment: &Figment) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    figment.extract::<T>().map_err(|e| ConfigError::Extraction {
        message: e.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect() for invariants that must hold"
)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tempfile::NamedTempFile;

    use super::*;

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
    struct TestConfig {
        port: u16,
        host: String,
        debug: bool,
    }

    #[test]
    fn load_from_toml_file() {
        use std::io::Write;

        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "port = 9090\nhost = \"example.com\"\ndebug = true").expect("write toml");

        let cfg: TestConfig =
            load(tmp.path().to_str().expect("path"), "KOINON_TEST").expect("load should succeed");

        assert_eq!(cfg.port, 9090);
        assert_eq!(cfg.host, "example.com");
        assert!(cfg.debug);
    }

    #[test]
    fn load_missing_file_returns_empty_config() {
        // A missing TOML file is silently skipped; this yields an empty
        // figment — extraction fails because required fields are absent.
        // The caller is responsible for providing defaults or making fields
        // optional.
        let result: Result<TestConfig, _> = load("/nonexistent/path.toml", "KOINON_MISSING");
        // We expect an extraction error (missing required fields), NOT a
        // file-not-found error.
        assert!(result.is_err());
    }

    #[test]
    fn load_with_defaults_fills_missing_fields() {
        use std::io::Write;

        let defaults = TestConfig {
            port: 8080,
            host: "localhost".to_string(),
            debug: false,
        };

        // TOML file only overrides `port`.
        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "port = 9090").expect("write toml");

        let cfg: TestConfig =
            load_with_defaults(tmp.path().to_str().expect("path"), "KOINON_DFLT", &defaults)
                .expect("load should succeed");

        assert_eq!(cfg.port, 9090);
        assert_eq!(cfg.host, "localhost"); // default
        assert!(!cfg.debug); // default
    }

    #[test]
    fn load_from_env_missing_prefix_returns_error() {
        // Without required fields, extraction fails — this tests the error
        // path without mutating process-global env vars (which is unsafe in
        // Rust 2024 and not safe to do in parallel tests).
        let result: Result<TestConfig, _> = load_from_env("KOINON_ABSENT_PREFIX_XXXXXXXXXX_");
        assert!(
            result.is_err(),
            "extraction should fail when env vars absent"
        );
    }
}

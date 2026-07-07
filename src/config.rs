//! `figment`-based configuration loader for forkwright crates.
//!
//! # Loading order (highest priority last wins)
//!
//! For [`load`] and [`load_from_env_path`]:
//!
//! 1. Defaults from `T::default()`.
//! 2. TOML file at `path` (if the file exists; skipped silently if absent).
//! 3. Environment variables with the given `prefix` (e.g. `APP_` maps
//!    `APP_PORT=8080` → `{ port: 8080 }`).
//!
//! [`load_with_defaults`] replaces layer 1 with a caller-supplied value.
//! [`load_from_env`] applies layer 3 only — no defaults, no file — for
//! types that cannot or should not implement [`Default`].
//!
//! The `env_prefix` works with or without the trailing underscore: `"APP"`
//! and `"APP_"` both map `APP_PORT`. Nested keys use a double underscore:
//! `APP_DB__URL` maps to `{ db: { url: ... } }`.
//!
//! # Usage
//!
//! ```rust,no_run
//! use koinon::config;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     port: u16,
//!     host: String,
//! }
//!
//! fn main() -> Result<(), koinon::error::ConfigError> {
//!     let cfg: AppConfig = config::load("app.toml", "APP")?;
//!     Ok(())
//! }
//! ```
//!
//! # Path convention
//!
//! When `path` is a relative path it is resolved against the current working
//! directory. Pass an absolute path or use [`load_from_env_path`] to pick the
//! path from an environment variable (e.g. `APP_CONFIG=/etc/app.toml`).

use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::Deserialize;
use serde::Serialize;

use crate::error::ConfigError;

/// Load a typed configuration from a TOML file + environment variables,
/// with `T::default()` as the lowest-priority layer.
///
/// # Arguments
///
/// * `path` — path to the TOML file. Silently skipped if the file does not
///   exist, allowing pure-env deployments.
/// * `env_prefix` — upper-case prefix for environment variable overrides,
///   with or without the trailing underscore: `"APP"` and `"APP_"` both map
///   `APP_PORT=8080` → `config.port = 8080`.
///
/// # Errors
///
/// * [`ConfigError::Parse`] — the TOML file exists but is not valid TOML.
/// * [`ConfigError::InvalidValue`] — a field is present but has the wrong
///   type or an out-of-range value.
/// * [`ConfigError::MissingKey`] — a required field is absent from every
///   layer (unreachable through this function unless `T::default()` itself
///   skips fields via custom serialization).
/// * [`ConfigError::Extraction`] — any other figment failure.
///
/// # Examples
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize, Default)]
/// struct AppConfig {
///     port: u16,
/// }
///
/// # #[allow(unsafe_code)]
/// # fn set_env(key: &str, value: &str) {
/// #     // SAFETY: each doctest runs as its own single-threaded process.
/// #     unsafe { std::env::set_var(key, value) }
/// # }
/// # set_env("APP_PORT", "8080");
/// let cfg: AppConfig = koinon::config::load("app.toml", "APP")?;
/// assert_eq!(cfg.port, 8080);
/// # Ok::<(), koinon::error::ConfigError>(())
/// ```
// WHY: genuine cross-crate API; basanos' workspace-library exemption needs a
// [workspace] ancestor, which this standalone published crate does not have.
// kanon:ignore RUST/pub-visibility -- documented public config-loading API
pub fn load<T>(path: &str, env_prefix: &str) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize + Default,
{
    let figment = Figment::new()
        .merge(Serialized::defaults(T::default()))
        .merge(Toml::file(path))
        .merge(env_layer(env_prefix));

    extract(&figment)
}

/// Load a typed configuration with explicit defaults.
///
/// The `defaults` value is serialized and used as the lowest-priority layer,
/// so any field not present in the TOML file or environment falls back to the
/// value in `defaults`. Use this instead of [`load`] when the fallback values
/// are computed at runtime rather than baked into `T::default()`.
///
/// # Errors
///
/// Same taxonomy as [`load`]: [`ConfigError::Parse`],
/// [`ConfigError::InvalidValue`], [`ConfigError::MissingKey`], or
/// [`ConfigError::Extraction`].
// WHY: genuine cross-crate API; basanos' workspace-library exemption needs a
// [workspace] ancestor, which this standalone published crate does not have.
// kanon:ignore RUST/pub-visibility -- documented public config-loading API
pub fn load_with_defaults<T>(path: &str, env_prefix: &str, defaults: &T) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let figment = Figment::new()
        .merge(Serialized::defaults(defaults))
        .merge(Toml::file(path))
        .merge(env_layer(env_prefix));

    extract(&figment)
}

/// Load a typed configuration from environment variables only.
///
/// Useful in containerized deployments where a config file is not mounted.
/// No defaults layer is applied: every required field must come from the
/// environment or carry a serde default.
///
/// # Errors
///
/// * [`ConfigError::MissingKey`] — a required field has no matching
///   environment variable.
/// * [`ConfigError::InvalidValue`] — a variable is set but has the wrong
///   type or an out-of-range value.
/// * [`ConfigError::Extraction`] — any other figment failure.
// WHY: genuine cross-crate API; basanos' workspace-library exemption needs a
// [workspace] ancestor, which this standalone published crate does not have.
// kanon:ignore RUST/pub-visibility -- documented public config-loading API
pub fn load_from_env<T>(env_prefix: &str) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    let figment = Figment::new().merge(env_layer(env_prefix));
    extract(&figment)
}

/// Load a typed configuration where the config file path is specified by
/// an environment variable.
///
/// If the environment variable is unset, falls back to `default_path`.
/// Otherwise identical to [`load`], including the `T::default()` layer.
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
/// * [`ConfigError::InvalidValue`] — `path_env_var` is set but its value is
///   not valid UTF-8.
/// * Otherwise the same taxonomy as [`load`].
// WHY: genuine cross-crate API; basanos' workspace-library exemption needs a
// [workspace] ancestor, which this standalone published crate does not have.
// kanon:ignore RUST/pub-visibility -- documented public config-loading API
pub fn load_from_env_path<T>(
    path_env_var: &str,
    default_path: &str,
    env_prefix: &str,
) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize + Default,
{
    // WHY: the sole direct env read in this crate — figment's Env provider
    // cannot observe a non-UTF-8 value, and this function's contract IS the
    // path variable. Matching VarError distinguishes unset (fall back to
    // default_path) from non-UTF-8 (typed error) instead of conflating both.
    let path = match std::env::var(path_env_var) {
        Ok(path) => path,
        Err(std::env::VarError::NotPresent) => default_path.to_string(),
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::InvalidValue {
                key: path_env_var.to_string(),
                message: "environment variable value is not valid UTF-8".to_string(),
            });
        }
    };
    load(&path, env_prefix)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

// WHY: figment strips the literal prefix string, so a bare "APP" against
// APP_PORT would leave "_PORT" as the key and silently drop every override;
// normalizing to a trailing underscore makes "APP" and "APP_" equivalent.
fn env_layer(env_prefix: &str) -> Env {
    if env_prefix.is_empty() || env_prefix.ends_with('_') {
        Env::prefixed(env_prefix).split("__")
    } else {
        Env::prefixed(&format!("{env_prefix}_")).split("__")
    }
}

fn extract<T>(figment: &Figment) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    figment.extract::<T>().map_err(|e| classify(&e))
}

// WHY: figment reports a structured Kind per failure; mapping it onto the
// ConfigError taxonomy keeps MissingKey/InvalidValue/Parse machine-matchable
// instead of collapsing every failure into a stringified Extraction.
fn classify(error: &figment::Error) -> ConfigError {
    use figment::error::Kind;

    match &error.kind {
        Kind::MissingField(field) => ConfigError::MissingKey {
            key: join_key(&error.path, field),
        },
        Kind::InvalidType(..)
        | Kind::InvalidValue(..)
        | Kind::InvalidLength(..)
        | Kind::UnknownVariant(..)
        | Kind::UnknownField(..)
        | Kind::DuplicateField(..)
        | Kind::ISizeOutOfRange(..)
        | Kind::USizeOutOfRange(..)
        | Kind::Unsupported(..)
        | Kind::UnsupportedKey(..) => ConfigError::InvalidValue {
            key: if error.path.is_empty() {
                "<root>".to_string()
            } else {
                error.path.join(".")
            },
            message: error.kind.to_string(),
        },
        Kind::Message(message) => match source_file(error) {
            Some(path) => ConfigError::Parse {
                path,
                message: message.clone(),
            },
            None => ConfigError::Extraction {
                message: error.to_string(),
            },
        },
    }
}

fn join_key(path: &[String], field: &str) -> String {
    if path.is_empty() {
        field.to_string()
    } else {
        format!("{}.{field}", path.join("."))
    }
}

fn source_file(error: &figment::Error) -> Option<String> {
    match error.metadata.as_ref()?.source.as_ref()? {
        figment::Source::File(path) => Some(path.display().to_string()),
        _ => None,
    }
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

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct TestConfig {
        port: u16,
        host: String,
        debug: bool,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                port: 8080,
                host: "localhost".to_string(),
                debug: false,
            }
        }
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
    fn load_missing_file_falls_back_to_defaults() {
        // A missing TOML file is silently skipped; with no file and no env
        // overrides, every field comes from the T::default() layer.
        let cfg: TestConfig =
            load("/nonexistent/path.toml", "KOINON_MISSING").expect("defaults should fill");
        assert_eq!(cfg, TestConfig::default());
    }

    #[test]
    fn load_defaults_layer_fills_fields_absent_from_file() {
        use std::io::Write;

        // TOML file only overrides `port`; host/debug come from T::default().
        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "port = 9090").expect("write toml");

        let cfg: TestConfig =
            load(tmp.path().to_str().expect("path"), "KOINON_LAYER").expect("load should succeed");

        assert_eq!(cfg.port, 9090);
        assert_eq!(cfg.host, "localhost");
        assert!(!cfg.debug);
    }

    #[test]
    fn load_with_defaults_fills_missing_fields() {
        use std::io::Write;

        let defaults = TestConfig {
            port: 7070,
            host: "explicit.example".to_string(),
            debug: true,
        };

        // TOML file only overrides `port`.
        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "port = 9090").expect("write toml");

        let cfg: TestConfig =
            load_with_defaults(tmp.path().to_str().expect("path"), "KOINON_DFLT", &defaults)
                .expect("load should succeed");

        assert_eq!(cfg.port, 9090);
        assert_eq!(cfg.host, "explicit.example"); // explicit default
        assert!(cfg.debug); // explicit default
    }

    #[test]
    fn load_from_env_missing_prefix_returns_missing_key() {
        // load_from_env applies no defaults layer, so an absent prefix must
        // surface the first missing required field as MissingKey, not a
        // stringified Extraction.
        let result: Result<TestConfig, _> = load_from_env("KOINON_ABSENT_PREFIX_XXXXXXXXXX_");
        match result {
            Err(ConfigError::MissingKey { key }) => assert_eq!(key, "port"),
            other => panic!("expected MissingKey, got {other:?}"),
        }
    }

    #[test]
    fn env_override_with_bare_prefix() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("KOINON_BARE_PORT", "9191");
            jail.set_env("KOINON_BARE_HOST", "bare.example");
            jail.set_env("KOINON_BARE_DEBUG", "true");

            let cfg: TestConfig =
                load_from_env("KOINON_BARE").map_err(|e| figment::Error::from(e.to_string()))?;
            assert_eq!(cfg.port, 9191);
            assert_eq!(cfg.host, "bare.example");
            assert!(cfg.debug);
            Ok(())
        });
    }

    #[test]
    fn env_override_with_trailing_underscore_prefix() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("KOINON_UND_PORT", "9292");
            jail.set_env("KOINON_UND_HOST", "und.example");
            jail.set_env("KOINON_UND_DEBUG", "false");

            let cfg: TestConfig =
                load_from_env("KOINON_UND_").map_err(|e| figment::Error::from(e.to_string()))?;
            assert_eq!(cfg.port, 9292);
            assert_eq!(cfg.host, "und.example");
            assert!(!cfg.debug);
            Ok(())
        });
    }

    #[test]
    fn env_overrides_file_overrides_defaults() {
        figment::Jail::expect_with(|jail| {
            jail.create_file("app.toml", "port = 9090\nhost = \"file.example\"")?;
            jail.set_env("KOINON_ORDER_PORT", "9393");

            let cfg: TestConfig = load("app.toml", "KOINON_ORDER")
                .map_err(|e| figment::Error::from(e.to_string()))?;
            assert_eq!(cfg.port, 9393); // env beats file
            assert_eq!(cfg.host, "file.example"); // file beats default
            assert!(!cfg.debug); // default
            Ok(())
        });
    }

    #[test]
    fn invalid_type_in_file_returns_invalid_value() {
        use std::io::Write;

        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "port = \"not-a-number\"").expect("write toml");

        let result: Result<TestConfig, _> = load(tmp.path().to_str().expect("path"), "KOINON_IV");
        match result {
            Err(ConfigError::InvalidValue { key, message }) => {
                assert!(key.contains("port"), "key: {key}");
                assert!(message.contains("invalid type"), "message: {message}");
            }
            other => panic!("expected InvalidValue, got {other:?}"),
        }
    }

    #[test]
    fn malformed_toml_returns_parse_error() {
        use std::io::Write;

        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "port = ").expect("write toml");

        let result: Result<TestConfig, _> = load(tmp.path().to_str().expect("path"), "KOINON_PE");
        match result {
            Err(ConfigError::Parse { path, .. }) => {
                assert!(!path.is_empty());
            }
            other => panic!("expected Parse, got {other:?}"),
        }
    }
}

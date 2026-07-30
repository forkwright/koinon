//! `figment`-based configuration loader for forkwright crates.
//!
//! # Loading order (highest priority last wins)
//!
//! For [`load`] and [`load_from_env_path`]:
//!
//! 1. Defaults from `T::default()`.
//! 2. TOML file at exactly `path` (if the file exists; skipped silently if
//!    absent). Parent/ancestor directories are never searched — a same-named
//!    file elsewhere in the tree is never adopted.
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

use std::path::Path;

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
/// * `path` — path to the TOML file, read at exactly this location (parent
///   directories are never searched). Silently skipped if the file does not
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
    load_figment(path, T::default(), env_layer(env_prefix))
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
    load_figment(path, defaults, env_layer(env_prefix))
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
    // WHY: path_env_var is a control-plane selector, not application data —
    // delegating to plain env_layer(env_prefix) would re-ingest it as a
    // same-named config key whenever it falls under env_prefix (koinon#17).
    load_figment(
        &path,
        T::default(),
        env_layer_excluding(env_prefix, path_env_var),
    )
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn load_figment<T, D>(path: &str, defaults: D, env: Env) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de>,
    D: Serialize,
{
    let figment =
        merge_toml_exact(Figment::new().merge(Serialized::defaults(defaults)), path).merge(env);

    extract(&figment)
}

// WHY: figment's Toml::file() walks the CWD and every ancestor directory
// looking for `path`, silently loading an unrelated same-named file from a
// parent directory when the exact target is absent from the CWD (koinon#16)
// — directly contradicting the "resolved against the current working
// directory" contract documented at the top of this module. Toml::file_exact
// never searches, but it also turns a missing file into a hard error (it
// unconditionally reads the path), which would break the documented
// "silently skipped if absent" pure-env-deployment contract tested by
// load_missing_file_falls_back_to_defaults. Checking existence first and
// only merging the exact-path provider when the file is actually there
// reproduces the original missing-file behavior while a present-but-
// unreadable file (permissions, race) still surfaces as a typed error from
// the read itself, instead of being silently absorbed as "not found".
fn merge_toml_exact(figment: Figment, path: &str) -> Figment {
    if Path::new(path).is_file() {
        figment.merge(Toml::file_exact(path))
    } else {
        figment
    }
}
// WHY: figment strips the literal prefix string, so a bare "APP" against
// APP_PORT would leave "_PORT" as the key and silently drop every override;
// normalizing to a trailing underscore makes "APP" and "APP_" equivalent.
fn normalized_prefix(env_prefix: &str) -> String {
    if env_prefix.is_empty() || env_prefix.ends_with('_') {
        env_prefix.to_string()
    } else {
        format!("{env_prefix}_")
    }
}

fn env_layer(env_prefix: &str) -> Env {
    Env::prefixed(&normalized_prefix(env_prefix)).split("__")
}

// WHY: `load_from_env_path`'s path selector and its data prefix share one
// process-wide env namespace; when path_env_var falls under env_prefix,
// exclude it from the data layer instead of letting it double as both the
// file selector and a same-named config key (koinon#17).
fn env_layer_excluding(env_prefix: &str, path_env_var: &str) -> Env {
    let prefix = normalized_prefix(env_prefix);
    let env = Env::prefixed(&prefix);
    let env = match strip_prefix_ci(&prefix, path_env_var) {
        Some(key) if !key.is_empty() => env.ignore(&[key]),
        _ => env,
    };
    env.split("__")
}

// WHY: figment's Env compares keys case-insensitively after stripping the
// prefix; `split_at_checked` yields both halves under one bounds-and-boundary
// test, so the remainder cannot be sliced with an index the check did not
// already prove valid.
fn strip_prefix_ci<'a>(prefix: &str, value: &'a str) -> Option<&'a str> {
    let (head, rest) = value.split_at_checked(prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then_some(rest)
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
    fn load_does_not_adopt_a_same_named_file_from_an_ancestor_directory() {
        // WHY(koinon#16): figment's Toml::file() walks the CWD and every
        // ancestor directory looking for `path`. A config file placed only
        // in a PARENT of the process's CWD must not be silently ingested
        // when the caller asks for an exact relative path that does not
        // exist in the CWD itself — the semantics under test, not just the
        // end value, is "was the ancestor file even consulted".
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "app.toml",
                "port = 1111\nhost = \"ancestor.example\"\ndebug = true",
            )?;
            jail.change_dir(jail.create_dir("child")?)?;

            // `app.toml` exists only in the parent; `child/` (the CWD) has
            // no such file.
            let cfg: TestConfig = load("app.toml", "KOINON_NO_ANCESTOR")
                .map_err(|e| figment::Error::from(e.to_string()))?;

            // A pre-fix load() would return the ancestor's overridden
            // values (port 1111 / "ancestor.example" / true). The fixed
            // exact-path loader must treat the file as absent and fall
            // through to T::default() instead.
            assert_eq!(cfg, TestConfig::default());
            Ok(())
        });
    }

    #[test]
    fn load_reads_the_exact_path_even_when_an_ancestor_has_a_decoy_file() {
        // Companion to the ancestor-non-adoption test above: proves the CWD's
        // own file is still read correctly (not that file loading broke
        // entirely) even in the presence of a differently-valued ancestor
        // file of the same name.
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "app.toml",
                "port = 1111\nhost = \"ancestor.example\"\ndebug = false",
            )?;
            let child = jail.create_dir("child")?;
            jail.change_dir(&child)?;
            jail.create_file(
                "app.toml",
                "port = 2222\nhost = \"child.example\"\ndebug = true",
            )?;

            let cfg: TestConfig = load("app.toml", "KOINON_EXACT_PATH")
                .map_err(|e| figment::Error::from(e.to_string()))?;

            assert_eq!(cfg.port, 2222);
            assert_eq!(cfg.host, "child.example");
            assert!(cfg.debug);
            Ok(())
        });
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

    #[test]
    fn strip_prefix_ci_matches_case_insensitively_and_returns_the_remainder() {
        assert_eq!(strip_prefix_ci("KOINON_", "koinon_config"), Some("config"));
        assert_eq!(strip_prefix_ci("koinon_", "KOINON_CONFIG"), Some("CONFIG"));
        assert_eq!(strip_prefix_ci("", "KOINON_CONFIG"), Some("KOINON_CONFIG"));
        assert_eq!(strip_prefix_ci("KOINON_", "KOINON_"), Some(""));
    }

    #[test]
    fn strip_prefix_ci_returns_none_for_a_non_match_or_short_value() {
        assert_eq!(strip_prefix_ci("KOINON_", "OTHER_CONFIG"), None);
        assert_eq!(strip_prefix_ci("KOINON_", "KOI"), None);
    }

    #[test]
    fn strip_prefix_ci_returns_none_when_the_prefix_length_splits_a_char() {
        // WHY: env_prefix reaches this helper unvalidated, so prefix.len() is a
        // byte count that can land inside a multi-byte char of value. "aéb" has
        // boundaries at 0, 1, 3, 4 — so a 2-byte prefix splits "é". Slicing at
        // that index panics; the boundary check must reject it instead.
        assert_eq!("é".len(), 2);
        assert_eq!(strip_prefix_ci("ab", "aéb"), None);
        // Positive control at the next valid boundary, so the None above is
        // attributable to the split and not to a prefix mismatch.
        assert_eq!(strip_prefix_ci("aé", "aéb"), Some("b"));
    }

    #[test]
    fn load_from_env_path_selector_does_not_leak_into_config_field() {
        // The documented pairing path_env_var="KOINON_SELECTOR_CONFIG",
        // env_prefix="KOINON_SELECTOR" strips to data key "config" — exactly
        // the field name a target struct is likely to have. The selector
        // must select the file, not also overwrite that field.
        #[derive(Debug, Deserialize, Serialize, Default)]
        struct ConfigFieldTarget {
            config: String,
            marker: bool,
        }

        figment::Jail::expect_with(|jail| {
            jail.create_file("selected.toml", "config = \"from-file\"\nmarker = true")?;
            jail.set_env("KOINON_SELECTOR_CONFIG", "selected.toml");

            let cfg: ConfigFieldTarget = load_from_env_path(
                "KOINON_SELECTOR_CONFIG",
                "unused-default.toml",
                "KOINON_SELECTOR",
            )
            .map_err(|e| figment::Error::from(e.to_string()))?;

            assert_eq!(cfg.config, "from-file");
            assert!(cfg.marker);
            Ok(())
        });
    }

    #[test]
    fn load_from_env_path_selector_does_not_trip_deny_unknown_fields() {
        // A target with no `config` field and deny_unknown_fields must not
        // reject the selector variable as an unexpected key.
        #[derive(Debug, Deserialize, Serialize, Default)]
        #[serde(deny_unknown_fields)]
        struct StrictTarget {
            marker: bool,
        }

        figment::Jail::expect_with(|jail| {
            jail.create_file("strict.toml", "marker = true")?;
            jail.set_env("KOINON_STRICT_CONFIG", "strict.toml");

            let cfg: StrictTarget = load_from_env_path(
                "KOINON_STRICT_CONFIG",
                "unused-default.toml",
                "KOINON_STRICT",
            )
            .map_err(|e| figment::Error::from(e.to_string()))?;

            assert!(cfg.marker);
            Ok(())
        });
    }
}

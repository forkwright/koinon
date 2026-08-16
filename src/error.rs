//! Component error types koinon semantically owns.
//!
//! # Pattern
//!
//! Koinon exposes only errors produced by its own components — currently
//! [`ConfigError`], returned by [`crate::config`] and by the `bootstrap`
//! module's `run` (feature-gated; not a doc link here since this module is
//! always available and `bootstrap` is not). It does not define or claim a
//! binary's top-level application error: that sum belongs to the consumer,
//! which wraps koinon's component errors into it the same way any
//! consumer-owned `snafu` enum wraps a source.
//!
//! ```rust
//! use koinon::error::ConfigError;
//! use snafu::{ResultExt, Snafu};
//!
//! #[derive(Debug, Snafu)]
//! enum MainError {
//!     #[snafu(display("config: {source}"))]
//!     Config { source: ConfigError },
//!     #[snafu(display("domain: {source}"))]
//!     Domain { source: WidgetError },
//! }
//!
//! #[derive(Debug, Snafu)]
//! #[snafu(display("widget failure"))]
//! struct WidgetError;
//!
//! fn run() -> Result<(), MainError> {
//!     let result: Result<(), WidgetError> = WidgetSnafu.fail();
//!     result.context(DomainSnafu)?;
//!     Ok(())
//! }
//!
//! use std::error::Error as _;
//! let err = run().unwrap_err();
//! assert!(err.source().is_some(), "domain error stays in the source chain");
//! ```
//!
//! `snafu` is not re-exported here: a re-export whose only contract is
//! shortening `use snafu::...` to `use koinon::error::...` is import
//! reduction, not behavior, and every consumer that defines its own `snafu`
//! enum already needs a direct `snafu` dependency to do so. Import the
//! macros from `snafu` directly, as the example above does.

use snafu::Snafu;

/// Error returned by the `config` module when a configuration file cannot
/// be loaded or validated.
///
/// Consumers that want to wrap this error in their own enum use:
///
/// ```rust
/// # use koinon::error::ConfigError;
/// # use snafu::Snafu;
/// #[derive(Debug, Snafu)]
/// enum MyError {
///     #[snafu(display("config: {source}"))]
///     Config { source: ConfigError },
/// }
/// ```
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum ConfigError {
    /// The config file was found but failed to parse.
    #[snafu(display("parse error in '{path}': {message}"))]
    Parse {
        /// The config file path.
        path: String,
        /// Description of the parse failure.
        message: String,
    },

    /// A required configuration key is missing.
    #[snafu(display("missing required key '{key}'"))]
    MissingKey {
        /// The missing key name.
        key: String,
    },

    /// A value failed validation (type mismatch, out-of-range, etc.).
    #[snafu(display("invalid value for '{key}': {message}"))]
    InvalidValue {
        /// The key whose value is invalid.
        key: String,
        /// Description of why the value is invalid.
        message: String,
    },

    /// The figment extraction step failed.
    #[snafu(display("extraction failed: {message}"))]
    Extraction {
        /// The extraction error message.
        message: String,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_display_parse() {
        let err = ConfigError::Parse {
            path: "/etc/app.toml".to_string(),
            message: "missing field `port`".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("/etc/app.toml"));
        assert!(s.contains("missing field `port`"));
    }

    #[test]
    fn config_error_display_missing_key() {
        let err = ConfigError::MissingKey {
            key: "database.url".to_string(),
        };
        assert!(err.to_string().contains("database.url"));
    }
}

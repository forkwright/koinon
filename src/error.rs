//! Common error types and `snafu` re-exports for forkwright crates.
//!
//! # Pattern
//!
//! Library crates define their own domain-specific error enums with `snafu`.
//! Binary crates with no domain errors of their own can return
//! [`AppError`] directly from `main` to wrap initialization failures
//! (config loading, argument parsing) that precede any domain-specific
//! work.
//!
//! [`AppError`] is `#[non_exhaustive]` — a downstream crate cannot add a
//! variant to it. Binary crates that do have domain errors keep their own
//! top-level error enum and wrap koinon's component errors into it, the
//! same way any consumer-owned `snafu` enum wraps a source (see
//! [`ConfigError`] below):
//!
//! ```rust
//! use koinon::error::{ConfigError, ResultExt, Snafu};
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
//! # Re-exports
//!
//! `snafu::{ResultExt, Snafu, ensure, whatever}` are re-exported so crates
//! that add `koinon` do not also need a direct `snafu` dependency for the
//! common macros.
//!
//! # Usage
//!
//! ```rust
//! use koinon::error::{ConfigError, ResultExt, Snafu};
//!
//! #[derive(Debug, Snafu)]
//! enum MyError {
//!     #[snafu(display("config: {source}"))]
//!     Config { source: ConfigError },
//! }
//! ```

pub use snafu::{Location, ResultExt, Snafu, ensure, whatever};

/// Top-level error for binary `main` functions that have no
/// domain-specific errors of their own.
///
/// Wraps the two most common initialization failures (config loading,
/// argument parsing) that precede any domain-specific work. Returned as
/// `Box<dyn std::error::Error>` would hide variant structure; use this
/// typed enum instead.
///
/// `AppError` is `#[non_exhaustive]`, so a downstream crate cannot add a
/// variant to it. A binary with its own domain errors should define its
/// own top-level error enum and wrap koinon's component errors (such as
/// [`ConfigError`]) into it instead of trying to route a domain error
/// through `AppError` — see the module-level example above.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum AppError {
    /// Configuration loading failed.
    #[snafu(display("configuration error: {source}"))]
    Config {
        /// The underlying config error.
        source: ConfigError,
    },

    /// An argument is missing or invalid.
    #[snafu(display("argument error: {message}"))]
    Argument {
        /// Description of the problem.
        message: String,
    },

    /// An otherwise unclassified startup error.
    #[snafu(display("startup error: {message}"))]
    Startup {
        /// Description of the problem.
        message: String,
    },
}

/// Error returned by the `config` module when a configuration file cannot
/// be loaded or validated.
///
/// Consumers that want to wrap this error in their own enum use:
///
/// ```rust
/// # use koinon::error::{ConfigError, Snafu};
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
    fn app_error_display() {
        let err = AppError::Argument {
            message: "foo must be positive".to_string(),
        };
        assert!(err.to_string().contains("foo must be positive"));
    }

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

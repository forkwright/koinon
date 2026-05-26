//! Tracing subscriber initialization for forkwright binaries.
//!
//! Wraps `tracing_subscriber::fmt` + `EnvFilter` in a one-call setup that
//! respects `RUST_LOG` and falls back to a caller-supplied default directive.
//!
//! # Usage
//!
//! ```rust,no_run
//! koinon::telemetry::init("my_crate=info");
//! ```
//!
//! The `RUST_LOG` environment variable overrides the default at runtime:
//!
//! ```text
//! RUST_LOG=debug ./my-binary
//! ```

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

/// Initialize the global `tracing` subscriber.
///
/// Reads `RUST_LOG` from the environment; if unset, falls back to
/// `default_directive`. The directive syntax follows the
/// [`EnvFilter` directives format][ef].
///
/// Calling this function more than once is a no-op after the first successful
/// initialization (the subscriber is set globally via
/// `tracing_subscriber::fmt().try_init()`).
///
/// [ef]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
///
/// # Panics
///
/// Does not panic. If `default_directive` is not a valid filter directive,
/// the subscriber falls back to the `EnvFilter` default (`"error"`).
pub fn init(default_directive: &str) {
    let filter = build_filter(default_directive);
    let _ = fmt().with_env_filter(filter).try_init();
}

/// Initialize the tracing subscriber with a `compact` formatter.
///
/// Equivalent to [`init`] but uses the compact single-line format, which
/// reduces output verbosity in long-running services.
pub fn init_compact(default_directive: &str) {
    let filter = build_filter(default_directive);
    let _ = fmt().compact().with_env_filter(filter).try_init();
}

/// Initialize the tracing subscriber with JSON output.
///
/// Useful in production services where logs are consumed by a log aggregator
/// (Loki, Elasticsearch, etc.).
pub fn init_json(default_directive: &str) {
    let filter = build_filter(default_directive);
    let _ = fmt().json().with_env_filter(filter).try_init();
}

/// Build an [`EnvFilter`] that respects `RUST_LOG` and falls back to
/// `default_directive`.
///
/// Exported so callers that want to compose their own `fmt()` subscriber
/// can still benefit from the `RUST_LOG`-with-fallback pattern.
#[must_use]
pub fn build_filter(default_directive: &str) -> EnvFilter {
    if default_directive.is_empty() {
        EnvFilter::from_default_env()
    } else {
        match default_directive.parse() {
            Ok(directive) => EnvFilter::from_default_env().add_directive(directive),
            Err(_) => EnvFilter::from_default_env(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_filter_empty_directive() {
        // Should not panic when directive is empty.
        let _filter = build_filter("");
    }

    #[test]
    fn build_filter_valid_directive() {
        let _filter = build_filter("koinon=debug");
    }

    #[test]
    fn build_filter_invalid_directive_fallback() {
        // Malformed directive: falls back gracefully.
        let _filter = build_filter("!@#$invalid");
    }
}

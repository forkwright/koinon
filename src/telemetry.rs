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
/// `default_directive`, which may contain multiple comma-separated
/// directives (e.g. `"my_crate=info,hyper=warn"`). The directive syntax
/// follows the [`EnvFilter` directives format][ef].
///
/// Calling this function more than once is a no-op after the first successful
/// initialization (the subscriber is set globally via
/// `tracing_subscriber::fmt().try_init()`).
///
/// [ef]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
///
/// # Panics
///
/// Does not panic. Invalid segments in `default_directive` are skipped, each
/// with a stderr note from `tracing-subscriber`'s lossy parser; if no valid
/// segment remains, the filter falls back to the `EnvFilter` default
/// (`"error"`).
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

/// Build an [`EnvFilter`] where a set `RUST_LOG` wins outright and
/// `default_directive` applies only as the fallback.
///
/// `RUST_LOG` is parsed lossily (invalid segments skipped, each with a
/// stderr note) and takes precedence whenever it contributes at least one
/// directive — including bare levels like `RUST_LOG=warn`, which fully
/// replace the default rather than being merged with it. Only when
/// `RUST_LOG` is unset, empty, or entirely invalid is `default_directive`
/// parsed instead; it may contain multiple comma-separated directives
/// (e.g. `"my_crate=info,hyper=warn"`).
///
/// Exported so callers that want to compose their own `fmt()` subscriber
/// can still benefit from the `RUST_LOG`-with-fallback pattern.
#[must_use]
pub fn build_filter(default_directive: &str) -> EnvFilter {
    prefer_env(EnvFilter::builder().from_env_lossy(), default_directive)
}

// WHY: an env-derived filter that renders empty contributed no directives
// (RUST_LOG unset, empty, or entirely invalid) — only then does the caller
// default apply. The previous add_directive-over-from_default_env layering
// let an equal-specificity default clobber a set RUST_LOG.
fn prefer_env(env_filter: EnvFilter, default_directive: &str) -> EnvFilter {
    if env_filter.to_string().is_empty() {
        EnvFilter::new(default_directive)
    } else {
        env_filter
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: an EnvFilter built from an empty string via the plain builder
    // (no default directive) carries zero directives — the same shape
    // from_env_lossy() yields when RUST_LOG is unset. End-to-end RUST_LOG
    // coverage lives in tests/rust_log_precedence.rs (own process, so env
    // mutation cannot race other tests).
    fn empty_env_filter() -> EnvFilter {
        EnvFilter::builder().parse_lossy("")
    }

    #[test]
    fn env_directives_win_over_default() {
        let filter = prefer_env(EnvFilter::new("warn"), "info");
        assert_eq!(filter.to_string(), "warn");
    }

    #[test]
    fn empty_env_falls_back_to_default() {
        let filter = prefer_env(empty_env_filter(), "info");
        assert_eq!(filter.to_string(), "info");
    }

    #[test]
    fn multi_directive_default_applies_all_segments() {
        let filter = prefer_env(empty_env_filter(), "koinon=debug,hyper=warn");
        let rendered = filter.to_string();
        assert!(rendered.contains("koinon=debug"), "got: {rendered}");
        assert!(rendered.contains("hyper=warn"), "got: {rendered}");
    }

    #[test]
    fn invalid_default_segment_skipped_valid_kept() {
        let filter = prefer_env(empty_env_filter(), "koinon=debug,!@#$invalid");
        assert!(filter.to_string().contains("koinon=debug"));
    }

    #[test]
    fn fully_invalid_default_falls_back_to_error() {
        let filter = prefer_env(empty_env_filter(), "!@#$invalid");
        assert_eq!(filter.to_string(), "error");
    }

    #[test]
    fn empty_default_falls_back_to_error() {
        let filter = prefer_env(empty_env_filter(), "");
        assert_eq!(filter.to_string(), "error");
    }
}

//! `clap` prelude for forkwright CLI binaries.
//!
//! Provides a `Verbosity` flag and a `GlobalArgs` struct that every CLI
//! binary can embed via `#[command(flatten)]`. This keeps tracing-level
//! configuration out of each crate's argument parser.
//!
//! # Usage
//!
//! ```rust,no_run
//! use clap::Parser;
//! use koinon::cli::GlobalArgs;
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     global: GlobalArgs,
//!
//!     #[arg(short, long)]
//!     config: Option<String>,
//! }
//!
//! let cli = Cli::parse();
//! cli.global.init_tracing("my_crate=info");
//! tracing::info!("started");
//! ```

use clap::Args;

/// Verbosity level for `--verbose` / `-v` flags.
///
/// Each additional `-v` increases the log level. In combination with
/// [`GlobalArgs::init_tracing`], a non-zero flag count replaces the caller's
/// `default_directive` as the fallback filter; `RUST_LOG`, when set, takes
/// precedence over both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Verbosity {
    /// Log only `error` messages.
    Error = 0,
    /// Log `warn` and above.
    Warn = 1,
    /// Log `info` and above (default when `-v` is not passed).
    Info = 2,
    /// Log `debug` and above (`-v`).
    Debug = 3,
    /// Log everything (`-vv`).
    Trace = 4,
}

impl Verbosity {
    /// Convert a raw count of `-v` flags to a [`Verbosity`] level.
    ///
    /// 0 → `Info`, 1 → `Debug`, 2+ → `Trace`.
    #[must_use]
    pub fn from_flag_count(count: u8) -> Self {
        match count {
            0 => Self::Info,
            1 => Self::Debug,
            _ => Self::Trace,
        }
    }

    /// Return the `tracing` directive string for this verbosity level.
    ///
    /// The returned string is suitable as the default directive for
    /// [`crate::telemetry::init`].
    #[must_use]
    pub fn as_directive(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// Global arguments shared across all forkwright CLI binaries.
///
/// Embed via `#[command(flatten)]` in your top-level `clap` struct, then
/// call [`GlobalArgs::init_tracing`] at the start of `main`.
///
/// # Environment
///
/// `RUST_LOG` still takes precedence over `--verbose` flags, because
/// [`crate::telemetry::build_filter`] uses a set `RUST_LOG` outright and
/// only falls back to the flag-derived directive when `RUST_LOG` is unset.
#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    /// Increase log verbosity. Pass once for debug, twice for trace.
    ///
    /// `RUST_LOG` takes precedence if set.
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Emit logs as JSON (for log aggregators).
    #[arg(long = "log-json", global = true, default_value_t = false)]
    log_json: bool,
}

impl GlobalArgs {
    /// Initialize the global `tracing` subscriber using the verbosity flag.
    ///
    /// The `default_directive` is used when both `RUST_LOG` and `--verbose`
    /// are unset. If `--verbose` is set and `RUST_LOG` is absent, the
    /// flag count determines the level.
    ///
    /// If `log_json` is set, uses the JSON formatter.
    pub fn init_tracing(&self, default_directive: &str) {
        let directive = if self.verbose > 0 {
            Verbosity::from_flag_count(self.verbose).as_directive()
        } else {
            default_directive
        };

        if self.log_json {
            crate::telemetry::init_json(directive);
        } else {
            crate::telemetry::init(directive);
        }
    }

    /// Return the verbosity level derived from the `--verbose` flag count.
    ///
    /// This is the fallback level used when `RUST_LOG` is unset; it does not
    /// consult `RUST_LOG`. Filter-level resolution — where a set `RUST_LOG`
    /// wins — happens in [`crate::telemetry::build_filter`].
    #[must_use]
    pub fn verbosity(&self) -> Verbosity {
        Verbosity::from_flag_count(self.verbose)
    }

    /// Whether JSON log output was requested.
    #[must_use]
    pub fn log_json(&self) -> bool {
        self.log_json
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbosity_from_flag_count() {
        assert_eq!(Verbosity::from_flag_count(0), Verbosity::Info);
        assert_eq!(Verbosity::from_flag_count(1), Verbosity::Debug);
        assert_eq!(Verbosity::from_flag_count(2), Verbosity::Trace);
        assert_eq!(Verbosity::from_flag_count(255), Verbosity::Trace);
    }

    #[test]
    fn verbosity_directives() {
        assert_eq!(Verbosity::Error.as_directive(), "error");
        assert_eq!(Verbosity::Warn.as_directive(), "warn");
        assert_eq!(Verbosity::Info.as_directive(), "info");
        assert_eq!(Verbosity::Debug.as_directive(), "debug");
        assert_eq!(Verbosity::Trace.as_directive(), "trace");
    }

    #[test]
    fn verbosity_ordering() {
        assert!(Verbosity::Trace > Verbosity::Debug);
        assert!(Verbosity::Debug > Verbosity::Info);
        assert!(Verbosity::Info > Verbosity::Warn);
        assert!(Verbosity::Warn > Verbosity::Error);
    }

    #[test]
    fn global_args_verbosity_propagates() {
        let args = GlobalArgs {
            verbose: 1,
            log_json: false,
        };
        assert_eq!(args.verbosity(), Verbosity::Debug);
    }

    #[test]
    fn global_args_no_verbose_defaults_info() {
        let args = GlobalArgs {
            verbose: 0,
            log_json: false,
        };
        assert_eq!(args.verbosity(), Verbosity::Info);
    }
}

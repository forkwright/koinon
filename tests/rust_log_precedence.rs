//! End-to-end `RUST_LOG` precedence for `koinon::telemetry::build_filter`.
#![cfg(feature = "telemetry")]

use koinon::telemetry::build_filter;

// NOTE: env mutation lives in this integration binary (own process) via
// figment::Jail, which serializes env access under a global lock and
// restores prior values — the lib test binary never sees these variables.

#[test]
#[expect(
    clippy::result_large_err,
    reason = "figment::Jail::expect_with fixes the closure's Err type to figment::Error, a third-party type this crate does not own and cannot box at the API boundary"
)]
fn rust_log_set_wins_over_default_directive() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("RUST_LOG", "warn");
        let filter = build_filter("info");
        assert_eq!(filter.to_string(), "warn");
        Ok(())
    });
}

#[test]
#[expect(
    clippy::result_large_err,
    reason = "figment::Jail::expect_with fixes the closure's Err type to figment::Error, a third-party type this crate does not own and cannot box at the API boundary"
)]
fn rust_log_set_wins_over_multi_directive_default() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("RUST_LOG", "error");
        let filter = build_filter("koinon=debug,hyper=warn");
        assert_eq!(filter.to_string(), "error");
        Ok(())
    });
}

#[test]
#[expect(
    clippy::result_large_err,
    reason = "figment::Jail::expect_with fixes the closure's Err type to figment::Error, a third-party type this crate does not own and cannot box at the API boundary"
)]
fn rust_log_empty_falls_back_to_multi_directive_default() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("RUST_LOG", "");
        let filter = build_filter("koinon=debug,hyper=warn");
        let rendered = filter.to_string();
        assert!(rendered.contains("koinon=debug"), "got: {rendered}");
        assert!(rendered.contains("hyper=warn"), "got: {rendered}");
        Ok(())
    });
}

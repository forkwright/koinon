//! A consumer-owned top-level error wraps a local domain error and
//! propagates it through `?` while keeping it in the source chain.
//!
//! `koinon::error::AppError` cannot do this: it is `#[non_exhaustive]`, so
//! a downstream crate cannot add a domain variant to it (kanon#18). The
//! error module doc on `koinon::error::AppError` shows the same pattern
//! also declaring a `Config { source: ConfigError }` variant alongside
//! this one in a single enum.

use koinon::error::{ResultExt, Snafu};
use std::error::Error as _;

#[derive(Debug, Snafu)]
enum MainError {
    #[snafu(display("domain: {source}"))]
    Domain { source: WidgetError },
}

#[derive(Debug, Snafu)]
#[snafu(display("widget failure"))]
struct WidgetError;

fn run() -> Result<(), MainError> {
    let result: Result<(), WidgetError> = WidgetSnafu.fail();
    result.context(DomainSnafu)?;
    Ok(())
}

#[test]
fn domain_error_propagates_through_question_mark_and_keeps_its_source() {
    let err = run();
    let Err(err) = err else {
        panic!("run() must fail: WidgetSnafu.fail() always returns Err");
    };
    assert!(matches!(err, MainError::Domain { .. }));
    assert!(
        err.source().is_some(),
        "domain error must stay in the source chain, not collapse to a string"
    );
}

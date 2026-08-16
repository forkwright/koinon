"""Tests for `koinon_adoption.freshness.check_freshness`.

Every negative test here is the "fixture proving that failure" the unit
brief demands: each asserts the check actually returns a non-empty
violation list of the expected `kind` against a known-bad committed
`ADOPTION.md`, not merely that a fresh one passes. `test_...badsha...`
reproduces #19's own reported bug shape (a prose word standing in for a
derived fact) verbatim.
"""

from __future__ import annotations

from datetime import UTC, datetime, timedelta

from koinon_adoption import freshness
from koinon_adoption.model import AdoptionState
from koinon_adoption.render import BEGIN_MARKER, END_MARKER, NO_VALUE
from tests.conftest import read_fixture

_REGISTRY_ORDER = [
    "hamma",
    "gnomon",
    "akroasis",
    "aletheia",
    "kanon",
    "logismos",
    "harmonia",
    "thumos",
    "epistole",
    "theatron",
    "dioptron",
]

_NOW = datetime(2026, 8, 10, tzinfo=UTC)
_MAX_AGE = timedelta(days=14)
_HEADER = "| Repo | Dependency | Features | Consumer reference | Source SHA | Observed (UTC) |"
_SEPARATOR = "|---|---|---|---|---|---|"


def _marker_wrapped(*row_lines: str) -> str:
    return "\n".join([BEGIN_MARKER, _HEADER, _SEPARATOR, *row_lines, END_MARKER])


def test_fresh_document_has_no_violations() -> None:
    doc = read_fixture("adoption_fresh.md")
    violations = freshness.check_freshness(doc, _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE)
    assert violations == []


def test_resolved_empty_features_fixture_has_no_violations() -> None:
    # NOTE: `tests/fixtures/adoption_resolved_empty_features.md` carries a
    # real, legitimately-derived RESOLVED row with blank features and a
    # populated reference — see the fixture's own trailing WHY note.
    doc = read_fixture("adoption_resolved_empty_features.md")
    violations = freshness.check_freshness(doc, _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE)
    assert violations == []


def test_stale_age_fixture_fails_with_stale_violation() -> None:
    doc = read_fixture("adoption_stale_age.md")
    violations = freshness.check_freshness(doc, _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE)
    assert violations != []
    assert all(v.kind == "stale" for v in violations)
    assert len(violations) == len(_REGISTRY_ORDER)  # every row is ~101 days old


def test_stale_rowcount_fixture_fails_with_row_mismatch() -> None:
    doc = read_fixture("adoption_stale_rowcount.md")
    violations = freshness.check_freshness(doc, _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE)
    assert violations != []
    assert any(v.kind == "row-mismatch" for v in violations)


def test_stale_badsha_fixture_fails_with_malformed_sha() -> None:
    # NOTE: reproduces #19's exact reported bug shape: Hamma marked `done`
    # (prose) while only telemetry had moved. Here the SHA cell holds "done"
    # instead of a hex SHA — same defect class, mechanically detected.
    doc = read_fixture("adoption_stale_badsha.md")
    violations = freshness.check_freshness(doc, _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE)
    assert violations != []
    sha_violations = [v for v in violations if v.kind == "malformed-sha"]
    assert len(sha_violations) == 1
    assert "hamma" in sha_violations[0].detail
    assert "done" in sha_violations[0].detail


def test_missing_block_fails_with_missing_block_violation() -> None:
    violations = freshness.check_freshness(
        "# no markers at all\n", _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE
    )
    assert len(violations) == 1
    assert violations[0].kind == "missing-block"


def test_malformed_table_fails_with_malformed_block_violation() -> None:
    doc = _marker_wrapped("| only | four | cells |")
    violations = freshness.check_freshness(doc, _REGISTRY_ORDER, now=_NOW, max_age=_MAX_AGE)
    assert len(violations) == 1
    assert violations[0].kind == "malformed-block"


def test_resolved_row_without_evidence_is_inconsistent() -> None:
    row = (
        f"| hamma | {AdoptionState.RESOLVED.value} | {NO_VALUE} | {NO_VALUE} "
        "| `abcdef1234` | 2026-08-01T00:00:00Z |"
    )
    doc = _marker_wrapped(row)
    violations = freshness.check_freshness(doc, ["hamma"], now=_NOW, max_age=_MAX_AGE)
    assert any(v.kind == "inconsistent-resolved" for v in violations)


def test_resolved_row_with_empty_features_and_real_reference_is_not_inconsistent() -> None:
    # WHY: this is the false-positive shape, not a hypothetical. A consumer
    # on `default-features = false` referencing only
    # `koinon::error::AppError` (`error` is not in koinon's own
    # `[features].default`, see test_derive.py's `_OWN_DEFAULTS`) derives a
    # real RESOLVED row with `features=frozenset()` and a populated
    # `reference`. Features and reference are independent facts (Cargo
    # feature resolution vs. source-text scanning); neither alone being
    # blank means the row is hand-edited.
    row = (
        f"| hamma | {AdoptionState.RESOLVED.value} | {NO_VALUE} "
        "| `crates/dictyon/src/lib.rs:12` (`koinon::error`) "
        "| `abcdef1234` | 2026-08-01T00:00:00Z |"
    )
    doc = _marker_wrapped(row)
    violations = freshness.check_freshness(doc, ["hamma"], now=_NOW, max_age=_MAX_AGE)
    assert not any(v.kind == "inconsistent-resolved" for v in violations)


def test_resolved_row_with_features_and_missing_reference_is_not_inconsistent() -> None:
    # NOTE: symmetric case. A consumer whose Cargo.lock resolves koinon but
    # whose source hasn't yet reached a matched `koinon::<module>` call
    # site legitimately derives `reference=None` alongside real features.
    row = (
        f"| hamma | {AdoptionState.RESOLVED.value} | telemetry, config | {NO_VALUE} "
        "| `abcdef1234` | 2026-08-01T00:00:00Z |"
    )
    doc = _marker_wrapped(row)
    violations = freshness.check_freshness(doc, ["hamma"], now=_NOW, max_age=_MAX_AGE)
    assert not any(v.kind == "inconsistent-resolved" for v in violations)


def test_malformed_timestamp_is_reported_and_does_not_crash() -> None:
    row = "| hamma | not adopted | — | — | — | not-a-timestamp |"
    doc = _marker_wrapped(row)
    violations = freshness.check_freshness(doc, ["hamma"], now=_NOW, max_age=_MAX_AGE)
    assert any(v.kind == "malformed-timestamp" for v in violations)

"""Tests for `koinon_adoption.facts.find_references`.

The negative case (`test_...rejects_akroasis_local_homonym`) is the one
that matters most: it proves the narrow regex does not fire on
`koinon::GeoSignal` / `koinon::Timestamp`, which a naive `koinon::` grep
would have counted as evidence of adoption forkwright/koinon does not have.
"""

from __future__ import annotations

from koinon_adoption import facts
from tests.conftest import read_fixture


def test_find_references_matches_hamma_telemetry_import() -> None:
    text = read_fixture("hamma_connect_excerpt.rs")
    refs = facts.find_references("crates/dictyon/examples/connect.rs", text)
    assert len(refs) >= 1
    assert refs[0].module == "telemetry"
    assert "koinon::telemetry" in refs[0].line_text


def test_find_references_rejects_akroasis_local_homonym() -> None:
    text = read_fixture("akroasis_pipeline_excerpt.rs")
    refs = facts.find_references("crates/semaino/src/pipeline.rs", text)
    assert refs == []


def test_find_references_matches_brace_group_single_line() -> None:
    # Greedy `[^}]*` backtracks to the rightmost module name in the group,
    # so only one of {cli, config} is guaranteed — proving "at least one
    # reference" needs no more than that (see facts.py's module NOTE).
    text = "use koinon::{cli, config};\n"
    refs = facts.find_references("src/lib.rs", text)
    modules = {r.module for r in refs}
    assert modules & {"cli", "config"}


def test_find_references_empty_for_unrelated_text() -> None:
    assert facts.find_references("src/lib.rs", "fn main() {}\n") == []


def test_find_references_reports_correct_line_number() -> None:
    text = "// line 1\n// line 2\nuse koinon::error::AppError;\n"
    refs = facts.find_references("src/lib.rs", text)
    assert len(refs) == 1
    assert refs[0].line_number == 3
    assert refs[0].module == "error"

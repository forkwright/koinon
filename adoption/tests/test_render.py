"""Tests for `koinon_adoption.render`: rendering, splicing, and the
idempotency property TESTING.md requires of every state-modifying operation.
"""

from __future__ import annotations

import pytest

from koinon_adoption import render
from koinon_adoption.model import AdoptionState, ConsumerReference, RepoAdoption

_RESOLVED_ROW = RepoAdoption(
    repo="hamma",
    state=AdoptionState.RESOLVED,
    features=frozenset({"telemetry", "config"}),
    reference=ConsumerReference(
        module="telemetry",
        relative_path="crates/dictyon/examples/connect.rs",
        line_number=16,
        line_text="use koinon::telemetry;",
    ),
    source_sha="d928b96ea3",
    observed_at="2026-08-01T00:00:00Z",
)

_NOT_ADOPTED_ROW = RepoAdoption(
    repo="aletheia",
    state=AdoptionState.NOT_ADOPTED,
    features=frozenset({"config"}),  # deliberately non-empty: must still render blanked
    reference=None,
    source_sha="1234567890",
    observed_at="2026-08-01T00:00:00Z",
)

_DOCUMENT = f"""# Title

Some prose.

{render.BEGIN_MARKER}
old stale content
{render.END_MARKER}

Trailing prose.
"""


def test_render_row_shows_evidence_for_resolved() -> None:
    line = render.render_row(_RESOLVED_ROW)
    assert "resolved" in line
    assert "telemetry" in line
    assert "config" in line
    assert "connect.rs:16" in line
    assert "d928b96ea3" in line


def test_render_row_blanks_features_and_reference_for_non_resolved() -> None:
    # WHY: this fixture carries a non-empty `features` set. A non-resolved
    # state must never claim evidence it did not observe, even when the
    # underlying row has real data to blank.
    line = render.render_row(_NOT_ADOPTED_ROW)
    cells = [c.strip() for c in line.strip("|").split("|")]
    assert cells[1] == AdoptionState.NOT_ADOPTED.value
    assert cells[2] == render.NO_VALUE
    assert cells[3] == render.NO_VALUE
    assert cells[4] == "`1234567890`"  # sha is independent of adoption state


def test_extract_block_returns_text_between_markers() -> None:
    body = render.extract_block(_DOCUMENT)
    assert body == "old stale content"


def test_extract_block_raises_on_missing_markers() -> None:
    with pytest.raises(render.BlockNotFoundError):
        render.extract_block("# no markers here\n")


def test_extract_block_raises_on_duplicate_begin_marker() -> None:
    doc = f"{render.BEGIN_MARKER}\na\n{render.BEGIN_MARKER}\nb\n{render.END_MARKER}\n"
    with pytest.raises(render.BlockNotFoundError):
        render.extract_block(doc)


def test_replace_block_splices_new_rows_between_markers() -> None:
    updated = render.replace_block(_DOCUMENT, [_RESOLVED_ROW])
    assert "old stale content" not in updated
    assert "hamma" in updated
    assert updated.startswith("# Title")
    assert updated.rstrip().endswith("Trailing prose.")


def test_replace_block_is_idempotent() -> None:
    once = render.replace_block(_DOCUMENT, [_RESOLVED_ROW, _NOT_ADOPTED_ROW])
    twice = render.replace_block(once, [_RESOLVED_ROW, _NOT_ADOPTED_ROW])
    assert once == twice


def test_replace_block_raises_when_markers_absent() -> None:
    with pytest.raises(render.BlockNotFoundError):
        render.replace_block("no markers", [_RESOLVED_ROW])

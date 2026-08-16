"""Renders `RepoAdoption` rows to Markdown and splices the result into
`ADOPTION.md` between fixed marker comments.

Splicing (not whole-file generation) is what lets `ADOPTION.md` carry
hand-written prose above and below the derived block — the prose is not
this module's concern, and this module never reads or writes it.
"""

from __future__ import annotations

from typing import Final

from koinon_adoption.model import AdoptionState, RepoAdoption

BEGIN_MARKER: Final = "<!-- koinon-adoption:generated:start -->"
END_MARKER: Final = "<!-- koinon-adoption:generated:end -->"

#: Placeholder for a table cell that has no meaningful value at this row's state.
NO_VALUE: Final = "—"

_HEADER: Final = (
    "| Repo | Dependency | Features | Consumer reference | Source SHA | Observed (UTC) |",
    "|------|-----------|----------|---------------------|-----------|------------------|",
)


class BlockNotFoundError(Exception):
    """Raised when a document is missing a well-formed marker pair."""


def render_row(row: RepoAdoption) -> str:
    features = ", ".join(sorted(row.features)) if row.features else NO_VALUE
    reference = row.reference.render() if row.reference is not None else NO_VALUE
    sha = f"`{row.source_sha}`" if row.source_sha else NO_VALUE
    if row.state is not AdoptionState.RESOLVED:
        features = NO_VALUE
        reference = NO_VALUE
    cells = (row.repo, row.state.value, features, reference, sha, row.observed_at)
    return f"| {' | '.join(cells)} |"


def render_block(rows: list[RepoAdoption]) -> str:
    """The full derived block, markers included, ready to splice into the doc."""
    lines = [BEGIN_MARKER, *_HEADER, *(render_row(r) for r in rows), END_MARKER]
    return "\n".join(lines)


def extract_block(document: str) -> str:
    """The exact text strictly between the marker lines (markers excluded).

    Raises `BlockNotFoundError` on a missing marker, a duplicated marker, or
    END appearing before BEGIN — every one of those means the document is
    not in a state this tool can safely round-trip, so callers must fail
    rather than guess.
    """
    begin_count = document.count(BEGIN_MARKER)
    end_count = document.count(END_MARKER)
    if begin_count != 1 or end_count != 1:
        msg = f"expected exactly one BEGIN and one END marker, found {begin_count} and {end_count}"
        raise BlockNotFoundError(msg)
    begin_at = document.index(BEGIN_MARKER) + len(BEGIN_MARKER)
    end_at = document.index(END_MARKER)
    if end_at < begin_at:
        msg = "END marker appears before BEGIN marker"
        raise BlockNotFoundError(msg)
    return document[begin_at:end_at].strip("\n")


def replace_block(document: str, rows: list[RepoAdoption]) -> str:
    """Return `document` with the marker-delimited block replaced by `rows`.

    Idempotent: calling this twice with the same `rows` on the already-updated
    document yields byte-identical output to calling it once (see
    `tests/test_render.py::test_replace_block_is_idempotent`) — replacement
    is a pure function of the markers' positions, never of prior content
    between them.
    """
    begin_count = document.count(BEGIN_MARKER)
    end_count = document.count(END_MARKER)
    if begin_count != 1 or end_count != 1:
        msg = f"expected exactly one BEGIN and one END marker, found {begin_count} and {end_count}"
        raise BlockNotFoundError(msg)
    begin_at = document.index(BEGIN_MARKER)
    end_at = document.index(END_MARKER) + len(END_MARKER)
    if end_at < begin_at:
        msg = "END marker appears before BEGIN marker"
        raise BlockNotFoundError(msg)
    return document[:begin_at] + render_block(rows) + document[end_at:]

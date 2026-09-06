"""The row shape shared by derivation, rendering, and the freshness check."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


class AdoptionState(StrEnum):
    """The mechanical fact a repo's koinon dependency is in, never a judgment.

    Every variant is derived from Cargo.lock / Cargo.toml content actually
    read at observation time — none is inferred from a repo's name, README
    prose, or prior table state. `RESOLVED` is the only state where the
    reported features/reference fields are meaningful; every other state
    forces them to a placeholder (see `render.NO_VALUE`).
    """

    RESOLVED = "resolved"
    DECLARED_NO_LOCKFILE = "declared (no lockfile committed)"
    DECLARED_LOCK_MISMATCH = "declared (Cargo.lock has no matching entry)"
    LOCAL_HOMONYM_ONLY = "not adopted (local homonym only)"
    NOT_ADOPTED = "not adopted"
    UNOBSERVED_PRIVATE = "unobserved (private, no cross-repo credential)"


@dataclass(frozen=True, slots=True)
class ConsumerReference:
    """One source-level `koinon::<module>` usage site."""

    module: str
    relative_path: str
    line_number: int
    line_text: str

    def render(self) -> str:
        return f"`{self.relative_path}:{self.line_number}` (`koinon::{self.module}`)"


@dataclass(frozen=True, slots=True)
class RepoAdoption:
    """One row of the derived adoption table."""

    repo: str
    state: AdoptionState
    features: frozenset[str]
    reference: ConsumerReference | None
    # koinon's own resolved commit — see cargo.LockEntry.resolved_commit_sha's
    # docstring. Never the consumer repo's own HEAD commit.
    source_sha: str | None
    observed_at: str  # ISO 8601 UTC, e.g. "2026-08-15T21:03:00Z"

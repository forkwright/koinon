"""Offline staleness check for the derived block already committed in
`ADOPTION.md`.

Deliberately network-free — TESTING.md forbids network calls in tests, and
this is the check meant to run on every push/PR, not only on the scheduled
refresh (see `koinon_adoption.derive` for the live cross-repo fetch). Every
violation here is a *mechanical* fact about the committed document: it does
not re-derive anyone's adoption state, it only proves the committed state is
well-formed, internally consistent, and recent enough to still be trusted.

Two independent failure classes:

- **Structural** (`row-mismatch`, `malformed-sha`, `malformed-timestamp`,
  `inconsistent-resolved`): the block was hand-edited, or drifted from
  `registry.toml`. This is the exact regression #19 reports — Hamma marked
  `done` by prose while only telemetry had moved — reproduced as a
  malformed-sha fixture in `tests/fixtures/adoption_stale_badsha.md`.
- **Age** (`stale`): nobody re-ran `derive` recently enough for the block to
  still be trustworthy, independent of whether its content is well-formed.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import TYPE_CHECKING, Final

from koinon_adoption.model import AdoptionState
from koinon_adoption.render import NO_VALUE, BlockNotFoundError, extract_block

if TYPE_CHECKING:
    from collections.abc import Sequence

# WHY: 14 days. `adoption-refresh.yml` runs weekly; two full cycles of
# headroom before this fires keeps one missed/delayed scheduled run from
# paging anyone.
DEFAULT_MAX_AGE: Final = timedelta(days=14)

_SHA_CELL_RE: Final = re.compile(r"^`[0-9a-f]{7,40}`$")
_TIMESTAMP_FMT: Final = "%Y-%m-%dT%H:%M:%SZ"
_ROW_CELL_COUNT: Final = 6


class FreshnessCheckError(Exception):
    """Raised when the block's Markdown cannot be parsed as a row table at all."""


@dataclass(frozen=True, slots=True)
class Violation:
    """One reason the committed block failed the freshness check."""

    kind: str
    detail: str

    def render(self) -> str:
        return f"[{self.kind}] {self.detail}"


def _parse_rows(block_body: str) -> list[dict[str, str]]:
    lines = [ln for ln in block_body.splitlines() if ln.strip()]
    if len(lines) < 2:
        msg = "block has no header and separator lines"
        raise FreshnessCheckError(msg)
    rows: list[dict[str, str]] = []
    for line in lines[2:]:  # skip header + separator
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != _ROW_CELL_COUNT:
            msg = f"expected {_ROW_CELL_COUNT} cells, got {len(cells)}: {line!r}"
            raise FreshnessCheckError(msg)
        rows.append(
            {
                "repo": cells[0],
                "state": cells[1],
                "features": cells[2],
                "reference": cells[3],
                "sha": cells[4],
                "observed": cells[5],
            }
        )
    return rows


def check_freshness(
    document: str,
    registry_repo_names: Sequence[str],
    *,
    now: datetime,
    max_age: timedelta = DEFAULT_MAX_AGE,
) -> list[Violation]:
    """Every way `document`'s derived block fails to be fresh and well-formed.

    Empty return means fresh. `now` and `max_age` are always caller-supplied
    — this function never reads the live wall clock itself — so it stays
    deterministic and testable without real time passing, per TESTING.md.
    """
    try:
        block = extract_block(document)
    except BlockNotFoundError as exc:
        return [Violation("missing-block", str(exc))]

    try:
        rows = _parse_rows(block)
    except FreshnessCheckError as exc:
        return [Violation("malformed-block", str(exc))]

    violations: list[Violation] = []

    row_repos = [r["repo"] for r in rows]
    expected = list(registry_repo_names)
    if row_repos != expected:
        violations.append(
            Violation(
                "row-mismatch",
                f"table rows {row_repos!r} do not match registry.toml order {expected!r} "
                "— the block was hand-edited, or `derive` was not re-run after "
                "registry.toml changed",
            )
        )

    for row in rows:
        sha = row["sha"]
        if sha != NO_VALUE and _SHA_CELL_RE.match(sha) is None:
            violations.append(
                Violation(
                    "malformed-sha",
                    f"{row['repo']}: source SHA cell is not a hex SHA or {NO_VALUE!r}: {sha!r}",
                )
            )

        try:
            observed = datetime.strptime(row["observed"], _TIMESTAMP_FMT).replace(tzinfo=UTC)
        except ValueError:
            violations.append(
                Violation(
                    "malformed-timestamp",
                    f"{row['repo']}: observed cell is not an ISO-8601 UTC timestamp: "
                    f"{row['observed']!r}",
                )
            )
            continue

        age = now - observed
        if age > max_age:
            violations.append(
                Violation(
                    "stale",
                    f"{row['repo']}: observed {row['observed']} is {age.days}d old, "
                    f"exceeds max age {max_age.days}d",
                )
            )

        # WHY: features and reference are checked together with AND, not
        # separately or with OR. `derive.effective_features` legitimately
        # returns an empty set for a RESOLVED consumer on
        # `default-features = false` with no extra features (koinon's own
        # `[features].default` excludes `error`, so a consumer referencing
        # only `koinon::error::AppError` derives a real RESOLVED row with
        # `features = frozenset()` — see test_derive.py's `_OWN_DEFAULTS`).
        # Flagging either field alone treats two independent facts (Cargo
        # feature resolution vs. source-text reference scanning) as
        # co-required and cries wolf on correct derive output (koinon#19
        # review). A row with EVERY derived-evidence field blank has no
        # signal a real `derive_repo()` call could have produced it, so
        # that combination alone is still a sound violation.
        if (
            row["state"] == AdoptionState.RESOLVED.value
            and row["features"] == NO_VALUE
            and row["reference"] == NO_VALUE
        ):
            violations.append(
                Violation(
                    "inconsistent-resolved",
                    f"{row['repo']}: state is {AdoptionState.RESOLVED.value!r} but "
                    f"both features and reference are {NO_VALUE!r}",
                )
            )

    return violations

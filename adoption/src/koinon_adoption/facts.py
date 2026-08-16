"""Source-level scan for a compiled consumer reference to koinon's public API.

The regex is deliberately narrow — `koinon::` followed by one of the four
real top-level modules (`telemetry`, `config`, `cli`, `error`), not bare
`koinon::`. A bare match would also fire on akroasis's local homonym crate
(`koinon::GeoSignal`, `koinon::Timestamp`, `koinon::signal::...` — verified
against `forkwright/akroasis` `crates/semaino/src/pipeline.rs`), which
exports none of those four names. Callers additionally scope the scan to
files inside a crate whose manifest resolved the *external* dependency
(`cargo.resolve_workspace_reference` / `is_external_candidate`) — this
module's narrow regex is defense in depth, not the sole guard.
"""

from __future__ import annotations

import re
from typing import Final

from koinon_adoption.model import ConsumerReference

#: Real top-level modules of the published `koinon` crate (`src/lib.rs`).
_REAL_MODULES: Final = ("telemetry", "config", "cli", "error")
# NOTE: the optional `\{[^}]*\b` prefix matches a same-line brace-group
# import (`use koinon::{telemetry, config};`) as well as a direct path
# (`koinon::telemetry::init(...)`, `use koinon::error::AppError;`). Greedy
# backtracking means a brace group naming more than one real module yields
# only its rightmost name, not every name in the group — irrelevant to this
# module's actual job, which is proving *at least one* reference exists,
# not enumerating all of them. A brace-group import that wraps onto
# multiple lines is NOT matched at all — a known, deliberate gap:
# under-counting a real reference only makes this tool conservative
# (reports no evidence where some exists), never wrong in the dangerous
# direction (claiming evidence that is not there).
_REFERENCE_RE: Final = re.compile(r"\bkoinon::(?:\{[^}]*\b)?(" + "|".join(_REAL_MODULES) + r")\b")


def find_references(relative_path: str, text: str) -> list[ConsumerReference]:
    """Every `koinon::<real module>` usage site in one file's text, in line order."""
    found: list[ConsumerReference] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        match = _REFERENCE_RE.search(line)
        if match is not None:
            found.append(
                ConsumerReference(
                    module=match.group(1),
                    relative_path=relative_path,
                    line_number=line_number,
                    line_text=line.strip(),
                )
            )
    return found

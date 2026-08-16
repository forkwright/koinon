"""Loads `adoption/registry.toml`: the SSOT list of tracked fleet repos.

The registry declares *scope* (which repos are candidate koinon consumers) —
a thin-config judgment call (which repos, TEKHNE.md rung 2). It declares
nothing about any repo's adoption state; every fact about a listed repo is
derived at observation time by `derive.py`.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


class RegistryError(Exception):
    """Raised when `registry.toml` is missing, malformed, or empty."""


@dataclass(frozen=True, slots=True)
class TrackedRepo:
    """One fleet repo in scope for koinon-adoption derivation."""

    name: str
    owner: str


def load_registry(path: Path) -> list[TrackedRepo]:
    """Parse `registry.toml` into an ordered list of tracked repos.

    Order is preserved from the file — it is the canonical row order for
    the rendered table and for `freshness.check_freshness`'s row-identity
    comparison.
    """
    try:
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        msg = f"registry not found: {path}"
        raise RegistryError(msg) from exc
    except tomllib.TOMLDecodeError as exc:
        msg = f"registry is not valid TOML: {path}: {exc}"
        raise RegistryError(msg) from exc

    entries = raw.get("repo")
    if not isinstance(entries, list) or not entries:
        msg = f"registry has no [[repo]] entries: {path}"
        raise RegistryError(msg)

    repos: list[TrackedRepo] = []
    seen: set[str] = set()
    for entry in entries:
        if not isinstance(entry, dict) or "name" not in entry:
            msg = f"registry entry missing 'name': {entry!r}"
            raise RegistryError(msg)
        name = str(entry["name"])
        owner = str(entry.get("owner", "forkwright"))
        if name in seen:
            msg = f"registry lists {name!r} more than once"
            raise RegistryError(msg)
        seen.add(name)
        repos.append(TrackedRepo(name=name, owner=owner))
    return repos

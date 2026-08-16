"""Pure Cargo.toml / Cargo.lock parsing: does a manifest resolve the
*external* forkwright/koinon crate, distinct from a same-named local crate.

WHY this module exists at all: `crates/koinon` inside forkwright/akroasis is
an unrelated workspace-local crate (κοινόν — "common/shared types for the
Akroasis workspace"; see `standards/substrate.toml` id="koinon" WHY note,
kanon#3030). A text search for `koinon` or even `koinon::` alone conflates
it with the fleet-common scaffolding crate this repo publishes. Every
function here distinguishes the two mechanically: a `path` dependency, or a
Cargo.lock package entry with no `source` key, is always the local homonym
— never the external crate — because Cargo never records a `source` for a
workspace-member package.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from typing import Final

#: Cargo.lock `source` prefixes that identify the real forkwright/koinon
#: crate. A git dependency's source is pinned to this exact repository URL;
#: a future crates.io publish would carry a `registry+` source, and since
#: crate names are globally unique on crates.io, any `registry+` source on a
#: package literally named "koinon" is unambiguously this crate.
_EXTERNAL_SOURCE_PREFIXES: Final = ("git+https://github.com/forkwright/koinon", "registry+")


@dataclass(frozen=True, slots=True)
class LockEntry:
    """One `[[package]]` block from a Cargo.lock, filtered to name == "koinon"."""

    version: str
    source: str | None  # None means a path-only / workspace-member package.

    @property
    def is_external(self) -> bool:
        """True iff this entry can only be the real forkwright/koinon crate."""
        return self.source is not None and self.source.startswith(_EXTERNAL_SOURCE_PREFIXES)


class CargoParseError(Exception):
    """Raised when a Cargo.toml / Cargo.lock file is not valid TOML."""


def parse_lock_koinon_entries(lock_text: str, *, source_path: str) -> list[LockEntry]:
    """Every `[[package]]` entry named "koinon" in a Cargo.lock's text.

    A repo with zero koinon dependency (real or homonym) returns `[]`. A
    repo with only the akroasis-style local homonym returns one entry whose
    `is_external` is False.
    """
    try:
        data = tomllib.loads(lock_text)
    except tomllib.TOMLDecodeError as exc:
        msg = f"{source_path} is not valid TOML: {exc}"
        raise CargoParseError(msg) from exc
    packages = data.get("package", [])
    return [
        LockEntry(version=str(pkg["version"]), source=pkg.get("source"))
        for pkg in packages
        if isinstance(pkg, dict) and pkg.get("name") == "koinon"
    ]


@dataclass(frozen=True, slots=True)
class ManifestDependency:
    """A `koinon` entry under one manifest's `[dependencies]` table."""

    is_path: bool
    is_workspace_ref: bool
    features: frozenset[str]
    default_features: bool

    @property
    def is_external_candidate(self) -> bool:
        """True unless the entry is unambiguously the local path homonym.

        Still True for an unresolved `workspace = true` reference — the
        caller must resolve it against the workspace root before trusting
        this as "external."
        """
        return not self.is_path


def _parse_dependency_value(value: object) -> ManifestDependency | None:
    if isinstance(value, str):
        # `koinon = "0.1"` — bare version, always external (crates.io/registry).
        return ManifestDependency(
            is_path=False, is_workspace_ref=False, features=frozenset(), default_features=True
        )
    if isinstance(value, dict):
        features = frozenset(str(f) for f in value.get("features", []))
        default_features = bool(value.get("default-features", True))
        if value.get("workspace") is True:
            return ManifestDependency(
                is_path=False,
                is_workspace_ref=True,
                features=features,
                default_features=default_features,
            )
        return ManifestDependency(
            is_path="path" in value,
            is_workspace_ref=False,
            features=features,
            default_features=default_features,
        )
    return None


#: Tables Cargo resolves a dependency edge from, checked in this priority
#: order. WHY dev-dependencies is in scope at all: hamma's actual koinon
#: reference (crates/dictyon, the #19 evidence repo) is declared under
#: `[dev-dependencies]` — koinon backs dictyon's *example* binary, not the
#: library — and Cargo.lock resolves dev-dependencies into the same graph
#: as normal ones, so excluding this table would make `RESOLVED` disagree
#: with Cargo.lock's own ground truth for the one repo #19 cites as proof.
_DEPENDENCY_TABLES: Final = ("dependencies", "dev-dependencies", "build-dependencies")


def parse_manifest_koinon_dependency(
    manifest_text: str, *, source_path: str
) -> ManifestDependency | None:
    """The raw `koinon` entry from one Cargo.toml's dependency tables, if present.

    Returns `None` when no dependency table declares `koinon` at all.
    Does not resolve `workspace = true` — see `resolve_workspace_reference`.
    """
    try:
        data = tomllib.loads(manifest_text)
    except tomllib.TOMLDecodeError as exc:
        msg = f"{source_path} is not valid TOML: {exc}"
        raise CargoParseError(msg) from exc
    for table_name in _DEPENDENCY_TABLES:
        table = data.get(table_name, {})
        if isinstance(table, dict) and "koinon" in table:
            return _parse_dependency_value(table["koinon"])
    return None


def parse_workspace_dependency(
    workspace_manifest_text: str, *, source_path: str
) -> ManifestDependency | None:
    """The `[workspace.dependencies].koinon` entry from a workspace root manifest."""
    try:
        data = tomllib.loads(workspace_manifest_text)
    except tomllib.TOMLDecodeError as exc:
        msg = f"{source_path} is not valid TOML: {exc}"
        raise CargoParseError(msg) from exc
    ws = data.get("workspace", {})
    deps = ws.get("dependencies", {}) if isinstance(ws, dict) else {}
    if not isinstance(deps, dict) or "koinon" not in deps:
        return None
    return _parse_dependency_value(deps["koinon"])


def resolve_workspace_reference(
    member: ManifestDependency, workspace_root: ManifestDependency | None
) -> ManifestDependency | None:
    """Resolve a member's `koinon = { workspace = true }` against the root spec.

    Returns `None` when the root has no koinon entry, or the root entry is
    itself the local path homonym (a member cannot promote a path dep to
    external by referencing it). Member-declared `features` add to the
    workspace set (Cargo's actual union semantics); `default_features` is
    the AND of both — WHY: either side opting out of defaults means the
    effective default set is not fully applied.
    """
    if not member.is_workspace_ref:
        return member
    if workspace_root is None or workspace_root.is_path:
        return None
    return ManifestDependency(
        is_path=False,
        is_workspace_ref=False,
        features=member.features | workspace_root.features,
        default_features=member.default_features and workspace_root.default_features,
    )


def effective_features(
    dep: ManifestDependency, *, own_default_features: frozenset[str]
) -> frozenset[str]:
    """The full feature set Cargo would activate for this dependency edge.

    `own_default_features` is koinon's *own* `[features].default` list, read
    from this repo's own Cargo.toml at HEAD — always available locally,
    never fetched, so it can never itself go stale relative to what it
    describes.
    """
    base = own_default_features if dep.default_features else frozenset()
    return base | dep.features

"""Live derivation: clone every registered repo's default branch, compute
its adoption facts, and produce one `RepoAdoption` row per repo.

This is the module `freshness.py` deliberately has no equivalent of: it
touches the network (git clone over HTTPS) and is therefore integration-level,
run by the scheduled `adoption-refresh` workflow, never by a unit test. Every
decision this module *makes* is pushed into `cargo.py` / `facts.py`, which
are pure and unit-tested against real fixture content captured from
forkwright/hamma and forkwright/akroasis.
"""

from __future__ import annotations

import base64
import os
import subprocess
import tomllib
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from koinon_adoption import cargo, facts
from koinon_adoption.model import AdoptionState, ConsumerReference, RepoAdoption

if TYPE_CHECKING:
    from pathlib import Path

    from koinon_adoption.registry import TrackedRepo

_CLONE_TIMEOUT_SECONDS = 120
_TIMESTAMP_FMT = "%Y-%m-%dT%H:%M:%SZ"

# WHY: this is a distinct env var, not `GITHUB_TOKEN`. An operator can set
# it to a token with cross-repo read access (a GitHub App installation
# token or a scoped PAT) — the default per-repo `GITHUB_TOKEN` in Actions
# cannot read a *different* private repo no matter how it is passed. A
# private tracked repo with no such token set reports `UNOBSERVED_PRIVATE`
# — an honest gap, not a guess — until one is provisioned, which is an
# operator action, not something this tool does.
CROSS_REPO_TOKEN_ENV_VAR = "KOINON_ADOPTION_TOKEN"


def _auth_env(token: str | None) -> dict[str, str]:
    """Extra process env that makes `git clone` send `token` as a Bearer
    credential, via git's `GIT_CONFIG_*` injection rather than a `-c` CLI
    flag or an embedded URL — both of the latter land in argv, which any
    co-tenant process can read from `/proc/<pid>/cmdline`.
    """
    if not token:
        return {}
    basic = base64.b64encode(f":{token}".encode()).decode()
    return {
        "GIT_CONFIG_COUNT": "1",
        "GIT_CONFIG_KEY_0": "http.https://github.com/.extraheader",
        "GIT_CONFIG_VALUE_0": f"AUTHORIZATION: basic {basic}",
    }


def iso_timestamp(moment: datetime) -> str:
    """Render a UTC `datetime` in the exact format `freshness.py` parses."""
    return moment.astimezone(UTC).strftime(_TIMESTAMP_FMT)


def read_own_default_features(koinon_manifest_path: Path) -> frozenset[str]:
    """koinon's own `[features].default` list, read from this repo's Cargo.toml.

    Always a local, uncloned read — this repo's own manifest at HEAD is
    never "stale" relative to what it describes.
    """
    data = tomllib.loads(koinon_manifest_path.read_text(encoding="utf-8"))
    features = data.get("features", {})
    default = features.get("default", []) if isinstance(features, dict) else []
    return frozenset(str(f) for f in default)


def clone_default_branch(
    owner: str, name: str, dest: Path, *, token: str | None = None
) -> str | None:
    """Shallow-clone `owner/name` into `dest`; return the HEAD SHA, or `None`.

    `None` covers every reason the clone did not succeed — private repo with
    no cross-repo credential, repo renamed/deleted, transient network
    failure, timeout. All are reported identically as `UNOBSERVED_PRIVATE`:
    this tool cannot distinguish "genuinely private" from "temporarily
    unreachable" from a failed `git clone` alone, and asserting a specific
    cause it has not verified would violate the same discipline this tool
    exists to enforce on the old hand-written table.
    """
    env = {**os.environ, **_auth_env(token)}
    try:
        result = subprocess.run(
            [
                "git",
                "clone",
                "--depth",
                "1",
                "--quiet",
                f"https://github.com/{owner}/{name}.git",
                str(dest),
            ],
            capture_output=True,
            text=True,
            timeout=_CLONE_TIMEOUT_SECONDS,
            check=False,
            env=env,
        )
    except subprocess.TimeoutExpired:
        return None
    if result.returncode != 0:
        return None
    sha = subprocess.run(
        ["git", "-C", str(dest), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    )
    return sha.stdout.strip()


@dataclass(frozen=True, slots=True)
class _ExternalHit:
    crate_dir: Path
    dependency: cargo.ManifestDependency


def _find_external_hits(
    checkout: Path,
) -> tuple[list[_ExternalHit], bool]:
    """External (non-path) koinon dependency edges found under `checkout`.

    Returns `(hits, any_koinon_mention)` — `any_koinon_mention` is True even
    when every hit turned out to be the local-homonym path form, so callers
    can report `LOCAL_HOMONYM_ONLY` instead of `NOT_ADOPTED`.
    """
    root_manifest = checkout / "Cargo.toml"
    workspace_root_dep: cargo.ManifestDependency | None = None
    if root_manifest.is_file():
        workspace_root_dep = cargo.parse_workspace_dependency(
            root_manifest.read_text(encoding="utf-8"), source_path=str(root_manifest)
        )

    hits: list[_ExternalHit] = []
    any_mention = False
    for manifest_path in sorted(checkout.rglob("Cargo.toml")):
        text = manifest_path.read_text(encoding="utf-8")
        dep = cargo.parse_manifest_koinon_dependency(text, source_path=str(manifest_path))
        if dep is None:
            continue
        any_mention = True
        resolved = (
            cargo.resolve_workspace_reference(dep, workspace_root_dep)
            if dep.is_workspace_ref
            else dep
        )
        if resolved is not None and not resolved.is_path:
            hits.append(_ExternalHit(crate_dir=manifest_path.parent, dependency=resolved))
    return hits, any_mention


def _find_first_reference(hits: list[_ExternalHit], checkout: Path) -> ConsumerReference | None:
    all_refs: list[ConsumerReference] = []
    for hit in hits:
        for rs_path in sorted(hit.crate_dir.rglob("*.rs")):
            relative = str(rs_path.relative_to(checkout))
            text = rs_path.read_text(encoding="utf-8", errors="replace")
            all_refs.extend(facts.find_references(relative, text))
    if not all_refs:
        return None
    all_refs.sort(key=lambda r: (r.relative_path, r.line_number))
    return all_refs[0]


def derive_repo(
    tracked: TrackedRepo,
    *,
    own_default_features: frozenset[str],
    clone_root: Path,
    now: datetime,
    token: str | None = None,
) -> RepoAdoption:
    """The full derived row for one registered repo, cloning its default branch."""
    observed_at = iso_timestamp(now)
    dest = clone_root / tracked.name
    sha = clone_default_branch(tracked.owner, tracked.name, dest, token=token)
    if sha is None:
        return RepoAdoption(
            repo=tracked.name,
            state=AdoptionState.UNOBSERVED_PRIVATE,
            features=frozenset(),
            reference=None,
            source_sha=None,
            observed_at=observed_at,
        )

    hits, any_mention = _find_external_hits(dest)

    lock_path = dest / "Cargo.lock"
    external_lock: list[cargo.LockEntry] = []
    if lock_path.is_file():
        entries = cargo.parse_lock_koinon_entries(
            lock_path.read_text(encoding="utf-8"), source_path=str(lock_path)
        )
        external_lock = [e for e in entries if e.is_external]

    if hits and external_lock:
        state = AdoptionState.RESOLVED
    elif hits and lock_path.is_file():
        state = AdoptionState.DECLARED_LOCK_MISMATCH
    elif hits:
        state = AdoptionState.DECLARED_NO_LOCKFILE
    elif any_mention:
        state = AdoptionState.LOCAL_HOMONYM_ONLY
    else:
        state = AdoptionState.NOT_ADOPTED

    features = frozenset[str]()
    for hit in hits:
        features |= cargo.effective_features(
            hit.dependency, own_default_features=own_default_features
        )

    reference = _find_first_reference(hits, dest) if hits else None

    return RepoAdoption(
        repo=tracked.name,
        state=state,
        features=features,
        reference=reference,
        source_sha=sha[:10],
        observed_at=observed_at,
    )


def derive_all(
    repos: list[TrackedRepo],
    *,
    own_default_features: frozenset[str],
    clone_root: Path,
    now: datetime,
    token: str | None = None,
) -> list[RepoAdoption]:
    """One row per registered repo, in registry order."""
    return [
        derive_repo(
            repo,
            own_default_features=own_default_features,
            clone_root=clone_root,
            now=now,
            token=token,
        )
        for repo in repos
    ]

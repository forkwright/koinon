"""Integration-shaped tests for `koinon_adoption.derive`, network-free.

`clone_default_branch` is monkeypatched to a fixed SHA; the checkout
content it would have produced is written directly from the same real
fixture files `test_cargo.py` / `test_facts.py` exercise in isolation, so
this file proves the *orchestration* (which crates get scanned, which
state wins) without ever touching the network — per TESTING.md.
"""

from __future__ import annotations

from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

from koinon_adoption import derive
from koinon_adoption.model import AdoptionState
from koinon_adoption.registry import TrackedRepo
from tests.conftest import FIXTURES_DIR, read_fixture

if TYPE_CHECKING:
    import pytest

_NOW = datetime(2026, 8, 10, tzinfo=UTC)
_OWN_DEFAULTS = frozenset({"telemetry", "config", "cli"})


def _write_hamma_checkout(root: Path) -> Path:
    dest = root / "hamma"
    (dest / "crates" / "dictyon" / "examples").mkdir(parents=True)
    (dest / "Cargo.toml").write_text(read_fixture("hamma_root_cargo_toml.toml"), encoding="utf-8")
    (dest / "Cargo.lock").write_text(
        read_fixture("hamma_cargo_lock_koinon_entry.lock"), encoding="utf-8"
    )
    (dest / "crates" / "dictyon" / "Cargo.toml").write_text(
        read_fixture("hamma_dictyon_cargo_toml.toml"), encoding="utf-8"
    )
    (dest / "crates" / "dictyon" / "examples" / "connect.rs").write_text(
        read_fixture("hamma_connect_excerpt.rs"), encoding="utf-8"
    )
    return dest


def _write_akroasis_checkout(root: Path) -> Path:
    dest = root / "akroasis"
    (dest / "crates" / "semaino" / "src").mkdir(parents=True)
    (dest / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/*"]\n', encoding="utf-8")
    (dest / "Cargo.lock").write_text(
        read_fixture("akroasis_cargo_lock_koinon_entry.lock"), encoding="utf-8"
    )
    (dest / "crates" / "semaino" / "Cargo.toml").write_text(
        read_fixture("akroasis_semaino_cargo_toml.toml"), encoding="utf-8"
    )
    (dest / "crates" / "semaino" / "src" / "pipeline.rs").write_text(
        read_fixture("akroasis_pipeline_excerpt.rs"), encoding="utf-8"
    )
    return dest


def test_derive_repo_resolves_hamma_with_telemetry_reference(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _write_hamma_checkout(tmp_path)
    monkeypatch.setattr(
        derive, "clone_default_branch", lambda owner, name, dest, **_kw: "d928b96ea3ab"
    )

    row = derive.derive_repo(
        TrackedRepo(name="hamma", owner="forkwright"),
        own_default_features=_OWN_DEFAULTS,
        clone_root=tmp_path,
        now=_NOW,
    )

    assert row.state is AdoptionState.RESOLVED
    assert "telemetry" in row.features
    assert row.reference is not None
    assert row.reference.module == "telemetry"
    assert row.source_sha == "d928b96ea3"  # truncated to 10 chars
    assert row.observed_at == "2026-08-10T00:00:00Z"


def test_derive_repo_reports_local_homonym_only_for_akroasis(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _write_akroasis_checkout(tmp_path)
    monkeypatch.setattr(
        derive, "clone_default_branch", lambda owner, name, dest, **_kw: "aaaaaaaaaaaa"
    )

    row = derive.derive_repo(
        TrackedRepo(name="akroasis", owner="forkwright"),
        own_default_features=_OWN_DEFAULTS,
        clone_root=tmp_path,
        now=_NOW,
    )

    assert row.state is AdoptionState.LOCAL_HOMONYM_ONLY
    assert row.features == frozenset()
    assert row.reference is None


def test_derive_repo_reports_not_adopted_for_empty_checkout(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    dest = tmp_path / "aletheia"
    dest.mkdir()
    (dest / "Cargo.toml").write_text('[package]\nname = "aletheia"\n', encoding="utf-8")
    monkeypatch.setattr(
        derive, "clone_default_branch", lambda owner, name, dest_, **_kw: "bbbbbbbbbbbb"
    )

    row = derive.derive_repo(
        TrackedRepo(name="aletheia", owner="forkwright"),
        own_default_features=_OWN_DEFAULTS,
        clone_root=tmp_path,
        now=_NOW,
    )

    assert row.state is AdoptionState.NOT_ADOPTED


def test_derive_repo_reports_unobserved_when_clone_fails(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(derive, "clone_default_branch", lambda owner, name, dest, **_kw: None)

    row = derive.derive_repo(
        TrackedRepo(name="gnomon", owner="forkwright"),
        own_default_features=_OWN_DEFAULTS,
        clone_root=tmp_path,
        now=_NOW,
    )

    assert row.state is AdoptionState.UNOBSERVED_PRIVATE
    assert row.source_sha is None
    assert row.features == frozenset()


def test_derive_repo_reports_declared_no_lockfile(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    dest = tmp_path / "logismos"
    dest.mkdir()
    (dest / "Cargo.toml").write_text(
        '[package]\nname = "logismos"\n\n[dependencies]\n'
        'koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.0" }\n',
        encoding="utf-8",
    )
    monkeypatch.setattr(
        derive, "clone_default_branch", lambda owner, name, dest_, **_kw: "cccccccccccc"
    )

    row = derive.derive_repo(
        TrackedRepo(name="logismos", owner="forkwright"),
        own_default_features=_OWN_DEFAULTS,
        clone_root=tmp_path,
        now=_NOW,
    )

    assert row.state is AdoptionState.DECLARED_NO_LOCKFILE


def test_read_own_default_features_reads_koinons_real_manifest() -> None:
    # NOTE: koinon's own Cargo.toml lives two directories above adoption/.
    manifest = Path(__file__).parents[2] / "Cargo.toml"
    features = derive.read_own_default_features(manifest)
    # WHY pinned rather than derived: the point of this test is to catch an
    # unintended change to the crate's default feature set, so it must be updated
    # deliberately whenever the set changes on purpose. `bootstrap` was added by the
    # bootstrap-boundary work; this expectation is the record that it was intended.
    assert features == frozenset({"telemetry", "config", "cli", "bootstrap"})


def test_derive_all_preserves_registry_order(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(derive, "clone_default_branch", lambda owner, name, dest, **_kw: None)
    repos = [
        TrackedRepo(name="a", owner="forkwright"),
        TrackedRepo(name="b", owner="forkwright"),
    ]
    rows = derive.derive_all(
        repos, own_default_features=_OWN_DEFAULTS, clone_root=tmp_path, now=_NOW
    )
    assert [r.repo for r in rows] == ["a", "b"]


def test_fixtures_dir_is_readable() -> None:
    assert FIXTURES_DIR.is_dir()


def test_auth_env_empty_without_token() -> None:
    assert derive._auth_env(None) == {}
    assert derive._auth_env("") == {}


def test_auth_env_never_puts_the_token_in_a_bare_argv_shaped_value() -> None:
    env = derive._auth_env("s3cr3t")
    assert env["GIT_CONFIG_COUNT"] == "1"
    assert env["GIT_CONFIG_KEY_0"] == "http.https://github.com/.extraheader"
    assert "s3cr3t" not in env["GIT_CONFIG_KEY_0"]
    assert env["GIT_CONFIG_VALUE_0"].startswith("AUTHORIZATION: basic ")

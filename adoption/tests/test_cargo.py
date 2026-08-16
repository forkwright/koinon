"""Tests for `koinon_adoption.cargo` against real captured Cargo manifests
from forkwright/hamma and forkwright/akroasis — see `tests/fixtures/*`.
"""

from __future__ import annotations

from koinon_adoption import cargo
from tests.conftest import read_fixture


def test_parse_lock_koinon_entries_finds_external_hamma_entry() -> None:
    entries = cargo.parse_lock_koinon_entries(
        read_fixture("hamma_cargo_lock_koinon_entry.lock"), source_path="hamma/Cargo.lock"
    )
    assert len(entries) == 1
    assert entries[0].version == "0.1.0"
    assert entries[0].is_external


def test_parse_lock_koinon_entries_rejects_akroasis_local_homonym() -> None:
    entries = cargo.parse_lock_koinon_entries(
        read_fixture("akroasis_cargo_lock_koinon_entry.lock"), source_path="akroasis/Cargo.lock"
    )
    assert len(entries) == 1
    assert entries[0].source is None
    assert not entries[0].is_external


def test_parse_lock_koinon_entries_empty_when_absent() -> None:
    assert cargo.parse_lock_koinon_entries("", source_path="empty") == []


def test_parse_manifest_koinon_dependency_reads_dev_dependencies() -> None:
    # dictyon's real manifest declares koinon under [dev-dependencies], not
    # [dependencies] — this is the case that broke a [dependencies]-only scan.
    dep = cargo.parse_manifest_koinon_dependency(
        read_fixture("hamma_dictyon_cargo_toml.toml"), source_path="dictyon/Cargo.toml"
    )
    assert dep is not None
    assert dep.is_workspace_ref
    assert not dep.is_path


def test_parse_manifest_koinon_dependency_flags_path_dependency() -> None:
    dep = cargo.parse_manifest_koinon_dependency(
        read_fixture("akroasis_semaino_cargo_toml.toml"), source_path="semaino/Cargo.toml"
    )
    assert dep is not None
    assert dep.is_path
    assert not dep.is_external_candidate


def test_parse_manifest_koinon_dependency_none_when_absent() -> None:
    manifest = '[package]\nname = "unrelated"\n[dependencies]\nserde = "1"\n'
    assert cargo.parse_manifest_koinon_dependency(manifest, source_path="x") is None


def test_parse_workspace_dependency_reads_hamma_root() -> None:
    dep = cargo.parse_workspace_dependency(
        read_fixture("hamma_root_cargo_toml.toml"), source_path="hamma/Cargo.toml"
    )
    assert dep is not None
    assert not dep.is_path
    assert dep.features == frozenset()
    assert dep.default_features


def test_resolve_workspace_reference_end_to_end_hamma() -> None:
    root = cargo.parse_workspace_dependency(
        read_fixture("hamma_root_cargo_toml.toml"), source_path="hamma/Cargo.toml"
    )
    member = cargo.parse_manifest_koinon_dependency(
        read_fixture("hamma_dictyon_cargo_toml.toml"), source_path="dictyon/Cargo.toml"
    )
    assert member is not None
    resolved = cargo.resolve_workspace_reference(member, root)
    assert resolved is not None
    assert not resolved.is_path
    assert resolved.default_features


def test_resolve_workspace_reference_none_when_root_is_path() -> None:
    root = cargo.ManifestDependency(
        is_path=True, is_workspace_ref=False, features=frozenset(), default_features=True
    )
    member = cargo.ManifestDependency(
        is_path=False, is_workspace_ref=True, features=frozenset(), default_features=True
    )
    assert cargo.resolve_workspace_reference(member, root) is None


def test_resolve_workspace_reference_none_when_root_missing() -> None:
    member = cargo.ManifestDependency(
        is_path=False, is_workspace_ref=True, features=frozenset(), default_features=True
    )
    assert cargo.resolve_workspace_reference(member, None) is None


_ALL_DEFAULTS = frozenset({"telemetry", "config", "cli"})


def test_resolve_workspace_reference_unions_features() -> None:
    root = cargo.ManifestDependency(
        is_path=False,
        is_workspace_ref=False,
        features=frozenset({"config"}),
        default_features=False,
    )
    member = cargo.ManifestDependency(
        is_path=False, is_workspace_ref=True, features=frozenset({"cli"}), default_features=True
    )
    resolved = cargo.resolve_workspace_reference(member, root)
    assert resolved is not None
    assert resolved.features == frozenset({"config", "cli"})
    assert resolved.default_features is False  # AND of True and False


def test_effective_features_applies_default_set_when_default_features_true() -> None:
    dep = cargo.ManifestDependency(
        is_path=False, is_workspace_ref=False, features=frozenset(), default_features=True
    )
    result = cargo.effective_features(dep, own_default_features=_ALL_DEFAULTS)
    assert result == _ALL_DEFAULTS


def test_effective_features_excludes_defaults_when_default_features_false() -> None:
    dep = cargo.ManifestDependency(
        is_path=False,
        is_workspace_ref=False,
        features=frozenset({"config"}),
        default_features=False,
    )
    result = cargo.effective_features(dep, own_default_features=_ALL_DEFAULTS)
    assert result == frozenset({"config"})


def test_parse_lock_rejects_invalid_toml() -> None:
    try:
        cargo.parse_lock_koinon_entries("not [ valid", source_path="broken.lock")
    except cargo.CargoParseError:
        return
    raise AssertionError("expected CargoParseError")

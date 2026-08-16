"""Tests for `koinon_adoption.registry.load_registry`."""

from __future__ import annotations

from pathlib import Path

import pytest

from koinon_adoption.registry import RegistryError, load_registry

_REAL_REGISTRY = Path(__file__).parents[1] / "registry.toml"


def test_loads_the_real_registry_toml_in_order() -> None:
    repos = load_registry(_REAL_REGISTRY)
    names = [r.name for r in repos]
    assert names == [
        "hamma",
        "gnomon",
        "akroasis",
        "aletheia",
        "kanon",
        "logismos",
        "harmonia",
        "thumos",
        "epistole",
        "theatron",
        "dioptron",
    ]
    assert all(r.owner == "forkwright" for r in repos)


def test_raises_on_missing_file(tmp_path: Path) -> None:
    with pytest.raises(RegistryError):
        load_registry(tmp_path / "does-not-exist.toml")


def test_raises_on_empty_registry(tmp_path: Path) -> None:
    path = tmp_path / "registry.toml"
    path.write_text("# empty\n", encoding="utf-8")
    with pytest.raises(RegistryError):
        load_registry(path)


def test_raises_on_duplicate_name(tmp_path: Path) -> None:
    path = tmp_path / "registry.toml"
    path.write_text('[[repo]]\nname = "hamma"\n\n[[repo]]\nname = "hamma"\n', encoding="utf-8")
    with pytest.raises(RegistryError):
        load_registry(path)


def test_raises_on_malformed_toml(tmp_path: Path) -> None:
    path = tmp_path / "registry.toml"
    path.write_text("not valid [ toml", encoding="utf-8")
    with pytest.raises(RegistryError):
        load_registry(path)


def test_owner_override_is_respected(tmp_path: Path) -> None:
    path = tmp_path / "registry.toml"
    path.write_text('[[repo]]\nname = "example"\nowner = "someone-else"\n', encoding="utf-8")
    repos = load_registry(path)
    assert repos[0].owner == "someone-else"

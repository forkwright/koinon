"""End-to-end test of the `check` subcommand — the exact entry point
`ci.yml` invokes on every push/PR.
"""

from __future__ import annotations

from pathlib import Path

from koinon_adoption.cli import main
from tests.conftest import FIXTURES_DIR

_REGISTRY = Path(__file__).parents[1] / "registry.toml"


def test_check_exits_zero_on_fresh_document() -> None:
    exit_code = main(
        [
            "check",
            "--registry",
            str(_REGISTRY),
            "--adoption-md",
            str(FIXTURES_DIR / "adoption_fresh.md"),
            "--now",
            "2026-08-10T00:00:00+00:00",
        ]
    )
    assert exit_code == 0


def test_check_exits_nonzero_on_stale_document() -> None:
    exit_code = main(
        [
            "check",
            "--registry",
            str(_REGISTRY),
            "--adoption-md",
            str(FIXTURES_DIR / "adoption_stale_badsha.md"),
            "--now",
            "2026-08-10T00:00:00+00:00",
        ]
    )
    assert exit_code == 1


def test_check_exits_nonzero_on_missing_adoption_md(tmp_path: Path) -> None:
    missing = tmp_path / "does-not-exist.md"
    try:
        main(["check", "--registry", str(_REGISTRY), "--adoption-md", str(missing)])
    except FileNotFoundError:
        return
    raise AssertionError("expected FileNotFoundError for a missing --adoption-md")

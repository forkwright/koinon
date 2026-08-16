"""Command-line entry points: `koinon-adoption derive` and `koinon-adoption check`.

`derive` is the live, network-touching regeneration (run by the scheduled
`adoption-refresh` workflow). `check` is the offline freshness gate (run on
every push/PR in `ci.yml`). See module docstrings on `derive.py` /
`freshness.py` for why they are split.
"""

from __future__ import annotations

import argparse
import os
import sys
import tempfile
from datetime import UTC, datetime, timedelta
from pathlib import Path

from koinon_adoption import derive as derive_mod
from koinon_adoption import freshness
from koinon_adoption.registry import RegistryError, TrackedRepo, load_registry
from koinon_adoption.render import replace_block

_REPO_ROOT = Path(__file__).resolve().parents[3]
_DEFAULT_REGISTRY = _REPO_ROOT / "adoption" / "registry.toml"
_DEFAULT_ADOPTION_MD = _REPO_ROOT / "ADOPTION.md"
_DEFAULT_KOINON_MANIFEST = _REPO_ROOT / "Cargo.toml"


def _cmd_derive(args: argparse.Namespace) -> int:
    repos: list[TrackedRepo] = load_registry(args.registry)
    if args.repo:
        wanted = set(args.repo)
        repos = [r for r in repos if r.name in wanted]
        missing = wanted - {r.name for r in repos}
        if missing:
            print(f"error: --repo names not in registry: {sorted(missing)}", file=sys.stderr)
            return 2

    own_default_features = derive_mod.read_own_default_features(args.koinon_manifest)
    now = datetime.now(UTC)
    token = os.environ.get(derive_mod.CROSS_REPO_TOKEN_ENV_VAR)

    with tempfile.TemporaryDirectory(prefix="koinon-adoption-") as tmp:
        clone_root = Path(args.clone_root) if args.clone_root else Path(tmp)
        rows = derive_mod.derive_all(
            repos,
            own_default_features=own_default_features,
            clone_root=clone_root,
            now=now,
            token=token,
        )

    document = args.adoption_md.read_text(encoding="utf-8")
    updated = replace_block(document, rows)
    changed = updated != document
    if not args.dry_run:
        args.adoption_md.write_text(updated, encoding="utf-8")

    for row in rows:
        print(f"{row.repo}: {row.state.value}")
    print("changed" if changed else "unchanged", file=sys.stderr)
    return 0


def _cmd_check(args: argparse.Namespace) -> int:
    try:
        repos = load_registry(args.registry)
    except RegistryError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    document = args.adoption_md.read_text(encoding="utf-8")
    now = datetime.now(UTC) if args.now is None else args.now
    max_age = timedelta(days=args.max_age_days)

    violations = freshness.check_freshness(
        document, [r.name for r in repos], now=now, max_age=max_age
    )
    if not violations:
        print("adoption block is fresh")
        return 0

    summary = f"adoption block failed freshness check ({len(violations)} violation(s)):"
    print(summary, file=sys.stderr)
    for v in violations:
        print(f"  {v.render()}", file=sys.stderr)
    return 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="koinon-adoption")
    sub = parser.add_subparsers(dest="command", required=True)

    derive_help = "live re-derivation; clones every registered repo"
    derive_parser = sub.add_parser("derive", help=derive_help)
    derive_parser.add_argument("--registry", type=Path, default=_DEFAULT_REGISTRY)
    derive_parser.add_argument("--adoption-md", type=Path, default=_DEFAULT_ADOPTION_MD)
    derive_parser.add_argument("--koinon-manifest", type=Path, default=_DEFAULT_KOINON_MANIFEST)
    derive_parser.add_argument(
        "--clone-root", type=str, default=None, help="reuse this dir instead of a temp dir"
    )
    derive_parser.add_argument(
        "--repo", action="append", default=[], help="limit to this repo name (repeatable)"
    )
    derive_parser.add_argument("--dry-run", action="store_true", help="do not write ADOPTION.md")
    derive_parser.set_defaults(func=_cmd_derive)

    check_parser = sub.add_parser("check", help="offline freshness check of the committed block")
    check_parser.add_argument("--registry", type=Path, default=_DEFAULT_REGISTRY)
    check_parser.add_argument("--adoption-md", type=Path, default=_DEFAULT_ADOPTION_MD)
    check_parser.add_argument("--max-age-days", type=int, default=freshness.DEFAULT_MAX_AGE.days)
    check_parser.add_argument(
        "--now", type=datetime.fromisoformat, default=None, help="override for manual reproduction"
    )
    check_parser.set_defaults(func=_cmd_check)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    sys.exit(main())

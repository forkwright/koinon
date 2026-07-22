<!--
scope: koinon repo — agent onboarding and dispatch conventions
defers_to: CLAUDE.md for full repo conventions; kanon standards for universal engineering policy
tightens: gate discipline (truthful Gate-Passed trailers, no AI indicators)
-->

# AGENTS.md

Read CLAUDE.md first for repo conventions and key patterns.

## What koinon is

Fleet-common Rust scaffolding: tracing init, typed error bases, figment config
loading, and a clap CLI prelude. Every forkwright binary and library depends
on this instead of hand-rolling these concerns.

## Commands

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Gate

Branch protection requires a `Gate-Passed:` trailer on a PR commit. Run
`kanon gate .` locally; it prints the trailer to use. Never fabricate it.

## Commit convention

`type(scope): description`. Scope is the module name (`telemetry`, `config`,
`error`, `cli`) or `crate` for workspace-level changes.

## Forbidden

- No `unwrap()`/`expect()` in library code.
- No AI attribution (no `Co-authored-by: Claude`, no emoji markers).
- Do not rewrite git history unilaterally.

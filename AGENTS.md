<!--
scope: koinon repo — agent onboarding and dispatch conventions
defers_to: CLAUDE.md for full repo conventions; kanon standards for universal engineering policy
tightens: gate discipline (truthful Gate-Passed trailers, no AI indicators)
-->

# AGENTS.md

Read CLAUDE.md first for repo conventions and key patterns.

## What koinon is

The typed application-bootstrap sequence for forkwright crates:
`bootstrap::run` integrates CLI/environment verbosity resolution, figment
config loading, and tracing init into one call. `cli`, `config`, and
`telemetry` are the leaves that sequence composes and remain directly usable
for a crate that genuinely needs only one of them. A forkwright binary that
owns a `main` should depend on this instead of hand-rolling the sequence;
koinon does not define or own a binary's top-level application error.

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

`type(scope): description`. Scope is the module name (`bootstrap`,
`telemetry`, `config`, `error`, `cli`) or `crate` for workspace-level
changes.

## Forbidden

- No `unwrap()`/`expect()` in library code.
- No AI attribution (no `Co-authored-by: Claude`, no emoji markers).
- Do not rewrite git history unilaterally.

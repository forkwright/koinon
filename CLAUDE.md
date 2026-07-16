<!--
scope: koinon repo conventions (fleet-common scaffolding crate)
defers_to: ~/.claude/CLAUDE.md for operator principles; kanon standards for universal engineering policy
tightens: narrows kanon RUST.md error/config/tracing conventions to koinon's snafu-only, figment-only, no-unwrap-in-lib specifics (see Key patterns below)
-->

# CLAUDE.md

Project orientation for AI coding agents working on koinon.

## What koinon is

Fleet-common Rust scaffolding for forkwright crates. Provides tracing init,
typed error bases, figment config loading, and a clap CLI prelude. Every
forkwright binary and library should depend on this instead of hand-rolling
these concerns.

## Standards

- `RUST.md` in kanon standards
- `TESTING.md` — test naming (`verb_condition`), error path coverage
- `WRITING.md` — doc comment style, commit message voice

## Commands

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Key patterns

- **Errors**: `snafu` throughout. Library code: domain enums. Binary `main`: `AppError`.
- **Config**: `figment` with TOML + env. No `std::env::var` calls in lib code
  (sole exception: `config::load_from_env_path`'s path-variable read, whose
  contract is that variable and which must distinguish unset from non-UTF-8 —
  figment's `Env` cannot observe either).
- **Tracing**: `tracing_subscriber::fmt` + `EnvFilter`. Never `println!` in lib.
- **No `unwrap()`/`expect()` in lib code**. Tests may use `expect("msg")`.

## Git

`type(scope): description`. Scope is the module name (`telemetry`, `config`,
`error`, `cli`) or `crate` for workspace-level changes.

## Before submitting

1. `cargo test` passes
2. `cargo clippy -- -D warnings` passes
3. `cargo fmt --check` passes
4. All public items have doc comments

<!-- kanon:auto-start -->
## Generated kanon context

- Registry name: `koinon`
- Forge repo: `forkwright/koinon`
- Kanon prefix: `ko`
- Config source: `workflow/kanon.toml [projects.koinon]`
- Standards source: `crates/basanos/standards/STANDARDS.md`
- MCP routing catalog: `workflow/AGENTS-mcp-tools.md`

Run `kanon docs sync --check --repo koinon` to verify this generated
section and `kanon docs sync --apply --repo koinon` to refresh it.

## Blast zone

- Paths explicitly named by the rendered prompt, role, or template input.

## Acceptance verifier

```bash
kanon gate
```
<!-- kanon:auto-end -->

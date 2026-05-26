# Changelog

All notable changes to koinon are documented here.

## [Unreleased]

### Added

- `telemetry` module: `init`, `init_compact`, `init_json`, `build_filter`
  — one-call tracing subscriber setup with `RUST_LOG` / `EnvFilter` fallback
- `error` module: `AppError`, `ConfigError`, `snafu` re-exports (`Snafu`,
  `ResultExt`, `Location`, `ensure`, `whatever`)
- `config` module: `load`, `load_with_defaults`, `load_from_env`,
  `load_from_env_path` — figment-backed TOML + env loader
- `cli` module: `GlobalArgs` (`--verbose` / `-v`, `--log-json`), `Verbosity`
  — clap-based global arg prelude
- Full doc coverage (`#![deny(missing_docs)]`)
- Test suite: unit tests for all public functions and types
- CI workflow: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- Gate-attestation workflow enforcing `Gate-Passed:` trailer on PRs

[Unreleased]: https://github.com/forkwright/koinon/compare/HEAD

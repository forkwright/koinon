# Changelog

All notable changes to koinon are documented here.

## 0.1.0 (2026-07-08)


### Features

* **crate:** initial fleet-common scaffolding crate ([d8b842d](https://github.com/forkwright/koinon/commit/d8b842d61ca556349916e06efd13640d8588c52d))


### Bug Fixes

* resolve all open audit findings + build the feature matrix + Tier-U CI + lint-clean ([#8](https://github.com/forkwright/koinon/issues/8)) ([b108a8c](https://github.com/forkwright/koinon/commit/b108a8c589e54f6961086ddef3b260da7a140738))

## [Unreleased]

### Changed

- **BREAKING**: `config::load` and `config::load_from_env_path` now require
  `T: Default + Serialize` and merge `Serialized::defaults(T::default())` as
  the lowest-priority layer. Before: the documented defaults layer did not
  exist and any struct with required fields failed extraction on a missing
  file. After: missing fields fall back to `T::default()`, matching the
  module doc. `load_from_env` keeps the no-defaults, no-`Default`-bound shape
  for types that should not have baked-in defaults.
- **BREAKING** (behavioral): `telemetry::build_filter` now lets a set
  `RUST_LOG` win outright over `default_directive`. Before: the default was
  layered via `add_directive` on top of the `RUST_LOG`-derived filter, so an
  equal-specificity default (e.g. bare `info` vs `RUST_LOG=warn`) silently
  clobbered the operator's env choice. After: `RUST_LOG` (parsed lossily)
  takes precedence whenever it contributes any directive; the default applies
  only when `RUST_LOG` is unset, empty, or entirely invalid.
- **BREAKING** (behavioral): `config` loaders map figment failures onto the
  full `ConfigError` taxonomy — `MissingKey` for missing fields,
  `InvalidValue` for type/value/range failures, `Parse` for TOML syntax
  errors in a file — instead of collapsing everything into `Extraction`.
  `Extraction` remains the fallback for uncategorized failures. Callers
  matching on `ConfigError::Extraction { .. }` for these cases must match the
  specific variants.
- `telemetry::build_filter` accepts multi-directive defaults
  (e.g. `"my_crate=info,hyper=warn"`). Before: the whole string failed the
  single-`Directive` parse and was silently dropped. After: segments are
  parsed individually; invalid segments are skipped with a stderr note.
- `config` loaders normalize `env_prefix`: `"APP"` and `"APP_"` both map
  `APP_PORT` → `port`. Before: a bare `"APP"` left `_PORT` as the stripped
  key and silently dropped every env override.
- `config::load_from_env_path` returns `ConfigError::InvalidValue` when the
  path variable is set but not valid UTF-8, instead of silently using
  `default_path`.
- `telemetry`, `config`, and `cli` are real Cargo features (default-on,
  per-feature optional dependencies, cfg-gated modules), making the
  documented `default-features = false` trimming work; `error` is always
  available. CI checks the feature matrix.

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

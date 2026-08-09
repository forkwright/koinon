# Changelog

All notable changes to koinon are documented here.

## [0.1.4](https://github.com/forkwright/koinon/compare/v0.1.3...v0.1.4) (2026-08-09)


### Bug Fixes

* **docs:** pin README and ADOPTION dependency examples to v0.1.3 ([#30](https://github.com/forkwright/koinon/issues/30)) ([4f59847](https://github.com/forkwright/koinon/commit/4f59847994684c666f0fe4bb1f47faf2a93ea424)), closes [#27](https://github.com/forkwright/koinon/issues/27)
* **error:** correct AppError guidance to a consumer-owned wrapper enum ([#29](https://github.com/forkwright/koinon/issues/29)) ([595aef9](https://github.com/forkwright/koinon/commit/595aef97fb8da2633e9c9be910df0aee69a4f03d)), closes [#18](https://github.com/forkwright/koinon/issues/18)
* **release:** let release-please bump the doc pins its guard requires ([#32](https://github.com/forkwright/koinon/issues/32)) ([7873d83](https://github.com/forkwright/koinon/commit/7873d837ed5d878afcdf162a845af68a90bdf8ef)), closes [#27](https://github.com/forkwright/koinon/issues/27)

## [0.1.3](https://github.com/forkwright/koinon/compare/v0.1.2...v0.1.3) (2026-08-03)


### Bug Fixes

* **config:** make the prefix-split boundary structural, not documented ([#24](https://github.com/forkwright/koinon/issues/24)) ([856987e](https://github.com/forkwright/koinon/commit/856987e34c0419ecb7e944fe8bbbc5b79a3eab78))

## [0.1.2](https://github.com/forkwright/koinon/compare/v0.1.1...v0.1.2) (2026-07-28)


### Bug Fixes

* **config:** read the exact config path instead of searching ancestors ([#23](https://github.com/forkwright/koinon/issues/23)) ([04367a6](https://github.com/forkwright/koinon/commit/04367a6b414728afd9b03e86016230198ed6ea34)), closes [#16](https://github.com/forkwright/koinon/issues/16)
* **config:** stop load_from_env_path ingesting its own path selector ([#21](https://github.com/forkwright/koinon/issues/21)) ([5fcb40b](https://github.com/forkwright/koinon/commit/5fcb40b797eb87ec9d00411b61ac6cef53f988e4)), closes [#17](https://github.com/forkwright/koinon/issues/17)

## [0.1.1](https://github.com/forkwright/koinon/compare/v0.1.0...v0.1.1) (2026-07-22)


### Documentation

* **kanon:** regenerate missing derived CLAUDE/README sections ([#12](https://github.com/forkwright/koinon/issues/12)) ([bc5f609](https://github.com/forkwright/koinon/commit/bc5f6098a58e2bef5942afa842ed1da60b290e48))
* **repo:** add thin AGENTS.md pointer to CLAUDE.md ([#13](https://github.com/forkwright/koinon/issues/13)) ([a671c7f](https://github.com/forkwright/koinon/commit/a671c7fe52fd061b152ba612eaa48ed231b415c8))

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

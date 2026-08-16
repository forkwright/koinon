# koinon Adoption Guide

Step-by-step migration from hand-rolled scaffolding to `koinon` for any
forkwright Rust crate or workspace.

## When to adopt

Adopt `koinon` when a crate has any of:
- Hand-rolled `tracing_subscriber::fmt().with_env_filter(...).init()` blocks
- A startup-error enum with no domain errors of its own (replace it with
  `koinon::error::AppError`), or one that hand-rolls config-loading errors
  instead of wrapping `koinon::error::ConfigError`
- Direct `serde` + file reads for config (replace with `koinon::config::load`)
- `clap` structs that duplicate `--verbose` / log-level args

## Step 1: Add the dependency

In `Cargo.toml`:

<!-- x-release-please-start-version -->
```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.4" }
```
<!-- x-release-please-end-version -->

Once koinon is published to crates.io:

```toml
[dependencies]
koinon = "0.1"
```

For workspaces, add to `[workspace.dependencies]` and reference via
`koinon = { workspace = true }` in each member crate.

Libraries that only need a subset can trim the dependency tree:

<!-- x-release-please-start-version -->
```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.4", default-features = false, features = ["config"] }
```
<!-- x-release-please-end-version -->

Features: `telemetry`, `config`, `cli` (implies `telemetry`); the `error`
module is always available.

## Step 2: Replace tracing init

**Before (typical hand-rolled pattern):**

```rust
let directive = "my_crate=info"
    .parse()
    .map_err(|e| MyError::TracingInit { message: e.to_string() })?;
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(directive)
    )
    .init();
```

**After:**

```rust
koinon::telemetry::init("my_crate=info");
```

If you need JSON logs in production, call `koinon::telemetry::init_json`.

Remove `tracing-subscriber` from `[dependencies]` (keep it in
`[dev-dependencies]` only if tests set up their own subscriber).

### With a CLI binary

If you have a `clap` CLI, embed `GlobalArgs` instead:

```rust
use clap::Parser;
use koinon::cli::GlobalArgs;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,
}

fn main() {
    let cli = Cli::parse();
    cli.global.init_tracing("my_crate=info");
}
```

## Step 3: Replace config loading

**Before (common figment hand-roll):**

```rust
use figment::{Figment, providers::{Format, Toml, Env}};

let config: AppConfig = Figment::new()
    .merge(Toml::file("app.toml"))
    .merge(Env::prefixed("APP_"))
    .extract()
    .map_err(|e| MyError::Config { message: e.to_string() })?;
```

**After:**

```rust
let config: AppConfig = koinon::config::load("app.toml", "APP_")?;
```

`load` requires `AppConfig: Default + Serialize` and applies `T::default()`
as the lowest-priority layer. For defaults computed at runtime instead of
baked into `Default`:

```rust
let defaults = AppConfig::for_environment(env);
let config: AppConfig = koinon::config::load_with_defaults("app.toml", "APP_", &defaults)?;
```

Remove the direct `figment` dependency from `Cargo.toml` unless the crate
uses figment APIs beyond what `koinon::config` exposes.

## Step 4: Use koinon error types (binary crates)

Binaries with **no domain-specific errors of their own** can return
`koinon::error::AppError` directly from `main`:

**Before:**

```rust
#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error("config: {0}")]
    Config(String),
    #[error("tracing init: {0}")]
    TracingInit(String),
}
```

**After:**

```rust
use koinon::error::AppError;
// AppError::Config, AppError::Startup, AppError::Argument are provided.
```

Binaries that **do** have domain-specific errors keep their own top-level
enum instead of replacing it with `AppError`, and wrap koinon's component
errors into it the same way any consumer-owned `snafu` enum wraps a
source. `AppError` is `#[non_exhaustive]`, so a downstream crate cannot
add a domain variant to it:

```rust
use koinon::error::{ConfigError, ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum MainError {
    #[snafu(display("config: {source}"))]
    Config { source: ConfigError },
    #[snafu(display("domain: {source}"))]
    Domain { source: MyDomainError },
}

fn run() -> Result<(), MainError> {
    let config: AppConfig = koinon::config::load("app.toml", "APP_")
        .context(ConfigSnafu)?;
    do_domain_work(&config).context(DomainSnafu)?;
    Ok(())
}
```

Library crates keep their own `snafu` error enums but can import the
`snafu` macros via `koinon::error::{Snafu, ResultExt}`:

```rust
use koinon::error::{Snafu, ResultExt};

#[derive(Debug, Snafu)]
enum DomainError { ... }
```

## Step 5: Verify

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

The build should succeed with no new warnings. Remove any now-unused
direct dependencies (`tracing-subscriber`, `figment`) from `Cargo.toml`.

## Fleet migration status

Derived, not hand-maintained. Every row below is computed by cloning the
tracked repo's default branch and reading its own `Cargo.toml` / `Cargo.lock`
— never inferred from a repo's name, README, or a prior run's row. See
`adoption/README.md` for the mechanism and `adoption/registry.toml` for the
tracked-repo list (adding or removing a repo there changes what gets
reported; it asserts nothing about any repo's adoption state itself).

Regenerate with `koinon-adoption derive` (run automatically by
`.github/workflows/adoption-refresh.yml`). Never hand-edit the block below —
`.github/workflows/ci.yml`'s `adoption` job runs `koinon-adoption check` on
every push/PR and fails when the block is hand-edited, drifted from
`registry.toml`, or older than 14 days.

<!-- koinon-adoption:generated:start -->
| Repo | Dependency | Features | Consumer reference | Source SHA | Observed (UTC) |
|------|-----------|----------|---------------------|-----------|------------------|
| hamma | resolved | cli, config, telemetry | `crates/dictyon/examples/connect.rs:16` (`koinon::telemetry`) | `4358b47d20` | 2026-08-16T01:54:27Z |
| gnomon | not adopted | — | — | `0e1d71258b` | 2026-08-16T01:54:27Z |
| akroasis | not adopted (local homonym only) | — | — | `3e488e0b17` | 2026-08-16T01:54:27Z |
| aletheia | not adopted | — | — | `1cc7604148` | 2026-08-16T01:54:27Z |
| kanon | not adopted | — | — | `0ea101e811` | 2026-08-16T01:54:27Z |
| logismos | not adopted | — | — | `5d0c4ddb10` | 2026-08-16T01:54:27Z |
| harmonia | not adopted | — | — | `1f7a1cfdec` | 2026-08-16T01:54:27Z |
| thumos | not adopted | — | — | `e1e66db98c` | 2026-08-16T01:54:27Z |
| epistole | not adopted | — | — | `3e7154d5fd` | 2026-08-16T01:54:27Z |
| theatron | not adopted | — | — | `9906675f3d` | 2026-08-16T01:54:27Z |
| dioptron | not adopted | — | — | `ee954441e1` | 2026-08-16T01:54:27Z |
<!-- koinon-adoption:generated:end -->

Column meanings: **Dependency** is Cargo.lock's own resolution of the
dependency edge (see `adoption/src/koinon_adoption/cargo.py` for how a
same-named *local* crate in a consumer's own workspace — a real case,
verified against `forkwright/akroasis`'s `crates/koinon` — is distinguished
from this crate). **Features** and **Consumer reference** are populated only
when Dependency is `resolved`; a `not adopted (local homonym only)` or
`unobserved` row cannot support either claim, and never shows one.

# koinon Adoption Guide

Step-by-step migration from hand-rolled scaffolding to `koinon` for any
forkwright Rust crate or workspace.

## When to adopt

Adopt `koinon` when a crate has any of:
- Hand-rolled `tracing_subscriber::fmt().with_env_filter(...).init()` blocks
- Custom `AppError` / startup error enums that duplicate `koinon::error::AppError`
- Direct `serde` + file reads for config (replace with `koinon::config::load`)
- `clap` structs that duplicate `--verbose` / log-level args

## Step 1: Add the dependency

In `Cargo.toml`:

```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.0" }
```

Once koinon is published to crates.io:

```toml
[dependencies]
koinon = "0.1"
```

For workspaces, add to `[workspace.dependencies]` and reference via
`koinon = { workspace = true }` in each member crate.

Libraries that only need a subset can trim the dependency tree:

```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.0", default-features = false, features = ["config"] }
```

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

If a binary's `main` function returns its own startup-error enum,
migrate it to `AppError`:

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
// For domain errors, keep the domain crate's own enum and add a From impl.
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

| Repo | Tracing | Config | CLI | Status | Notes |
|------|---------|--------|-----|--------|-------|
| hamma | migrated | — | — | done | proof repo #1 |
| gnomon | n/a | n/a | n/a | no Rust code | pure Python research repo; migrate when Rust crates land per BUILD-v0.8 M0 |
| akroasis | pending | pending | pending | — | active worker; migrate in dedicated pass |
| aletheia | pending | pending | pending | — | — |
| kanon | pending | pending | pending | — | — |
| logismos | pending | pending | pending | — | — |
| harmonia | pending | pending | pending | — | — |
| thumos | pending | pending | pending | — | — |
| epistole | pending | pending | pending | — | — |
| theatron | pending | pending | pending | — | — |
| dioptron | pending | pending | pending | — | — |

Update this table as migrations land.

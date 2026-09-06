# koinon Adoption Guide

Step-by-step migration from hand-rolled scaffolding to `koinon` for any
forkwright Rust crate or workspace.

## When to adopt

**A binary that owns a `main`** — parses its own CLI, loads its own config,
and initializes its own logging — adopts the integrated
[`koinon::bootstrap::run`](#recommended-full-bootstrap): one call replacing
the hand-rolled sequence of "parse args, then init tracing, then load
config, and hope nothing depends on the order."

**A library, or a binary that only has one of those three concerns**, adopts
the matching leaf module directly instead — [`koinon::telemetry`](#partial-adoption-one-leaf),
[`koinon::config`](#partial-adoption-one-leaf), or [`koinon::cli`](#partial-adoption-one-leaf).
A doctest or example binary that only needs `tracing_subscriber::fmt` set up
is not a bootstrap sequence; forcing it through `bootstrap::run` for a config
file and CLI struct it does not have would be the wrapper-for-its-own-sake
this guide does not want.

Either way, a domain error a consumer already hand-rolls with `snafu` or
`thiserror` stays exactly where it is — see
[Wrapping `ConfigError`](#wrapping-configerror) below. Koinon does not
define or claim a binary's top-level error sum.

## Step 1: Add the dependency

In `Cargo.toml`:

<!-- x-release-please-start-version -->
```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.3.1" }
```
<!-- x-release-please-end-version -->

Once koinon is published to crates.io:

```toml
[dependencies]
koinon = "0.1"
```

For workspaces, add to `[workspace.dependencies]` and reference via
`koinon = { workspace = true }` in each member crate.

Crates that only need one leaf can trim the dependency tree:

<!-- x-release-please-start-version -->
```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.3.1", default-features = false, features = ["config"] }
```
<!-- x-release-please-end-version -->

Features: `telemetry`, `config`, `cli` (implies `telemetry`), `bootstrap`
(implies `cli` + `config`); the `error` module is always available.

## Recommended: full bootstrap

Replaces a hand-rolled sequence like:

```rust
let cli = Cli::parse();
let directive = if cli.verbose > 0 { "debug" } else { "my_crate=info" };
tracing_subscriber::fmt().with_env_filter(directive).init();
let config: AppConfig = Figment::new()
    .merge(Serialized::defaults(AppConfig::default()))
    .merge(Toml::file("app.toml"))
    .merge(Env::prefixed("APP_"))
    .extract()
    .map_err(|e| MyError::Config { message: e.to_string() })?;
```

with one call that resolves CLI/environment verbosity, initializes telemetry
from that resolution, and loads the typed config through the same
defaults → TOML → env-var policy, in that order:

```rust
use clap::Parser;
use koinon::bootstrap;
use koinon::cli::GlobalArgs;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    port: u16,
}

fn main() -> Result<(), koinon::error::ConfigError> {
    let cli = Cli::parse();
    let boot = bootstrap::run(&cli.global, "app.toml", "APP", "my_crate=info")?;
    tracing::info!(port = boot.config.port, "started");
    Ok(())
}
```

`boot.config` is the loaded `AppConfig`; `boot.verbosity()` and
`boot.log_json()` are the exact values telemetry was initialized with, not
re-derived from config. `run` returns `koinon::error::ConfigError` -
§ [Wrapping `ConfigError`](#wrapping-configerror) covers wrapping it into a
binary's own top-level enum when one exists.

For a `main` with no domain errors of its own, `?` against `run`'s
`ConfigError` directly, as the example above does, is enough — there is no
separate koinon-provided top-level error type to reach for.

Remove `tracing-subscriber` and `figment` from direct `[dependencies]` (keep
`tracing-subscriber` in `[dev-dependencies]` only if tests set up their own
subscriber) once nothing in the crate calls them directly.

## Partial adoption (one leaf)

Use a single module directly when a crate has exactly one of these
boilerplate patterns and no bootstrap sequence to integrate it into — a
library example, or a binary with only one of the three concerns.

### Tracing init only

**Before:**

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

### Config loading only

**Before:**

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

### CLI verbosity flags only

Embed `GlobalArgs` without calling `bootstrap::run`, then drive telemetry
init yourself:

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

## Wrapping `ConfigError`

`koinon::error::ConfigError` is the only error type koinon exposes — it is
the one error koinon semantically owns, produced by `config::load` and by
`bootstrap::run`. A binary's top-level error sum is not koinon's to define;
it stays in the consumer, which wraps `ConfigError` into it the same way any
consumer-owned `snafu` enum wraps a source:

```rust
use koinon::error::ConfigError;
use snafu::{ResultExt, Snafu};

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

Koinon does not re-export `snafu` — import the macros directly from
`snafu`, as above. Every consumer defining its own `snafu` enum already
needs a direct `snafu` dependency to do so, so the re-export saved nothing
but a `use` line while adding a second name for the same items.

## Verify

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

### Notable adopters

- **aletheia's `xenodocheion`** (standalone stdio MCP server) adopted the
  `telemetry` leaf's writer-target seam (`init_with_writer`, v0.3.0, #61)
  to keep tracing output off the stdout stream its MCP JSON-RPC transport
  owns - merged in `forkwright/aletheia@675c27415fbc4674d712d7d2ebaeba7787fa032b`
  (PR forkwright/aletheia#7215). The generated table below picks this row
  up automatically on the next `adoption-refresh` run; this note is the
  provenance the table itself has no field for (it reports koinon's own
  resolved commit, not a pointer into a consumer repo's merge history).

<!-- koinon-adoption:generated:start -->
| Repo | Dependency | Features | Consumer reference | Source SHA | Observed (UTC) |
|------|-----------|----------|---------------------|-----------|------------------|
| hamma | resolved | bootstrap, cli, config, telemetry | `crates/dictyon/examples/connect.rs:16` (`koinon::telemetry`) | `3424f241da` | 2026-09-06T15:28:52Z |
| gnomon | unobserved (private, no cross-repo credential) | — | — | — | 2026-09-06T15:28:52Z |
| akroasis | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| aletheia | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| kanon | unobserved (private, no cross-repo credential) | — | — | — | 2026-09-06T15:28:52Z |
| logismos | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| harmonia | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| thumos | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| epistole | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| theatron | not adopted | — | — | — | 2026-09-06T15:28:52Z |
| dioptron | not adopted | — | — | — | 2026-09-06T15:28:52Z |
<!-- koinon-adoption:generated:end -->

Column meanings: **Dependency** is Cargo.lock's own resolution of the
dependency edge (see `adoption/src/koinon_adoption/cargo.py` for how a
same-named *local* crate in a consumer's own workspace — a real case,
verified against `forkwright/akroasis`'s `crates/koinon` — is distinguished
from this crate). **Features** and **Consumer reference** are populated only
when Dependency is `resolved`; a `not adopted (local homonym only)` or
`unobserved` row cannot support either claim, and never shows one. **Source
SHA** is koinon's own commit, read from the resolved lock entry's `source`
fragment — never the consumer repo's own HEAD commit (a prior generator bug
this table used to carry: a hamma commit shown on hamma's row), and blank
for every state but `resolved`, since only that state has a koinon commit
to report.

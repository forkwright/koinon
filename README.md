# koinon

κοινόν - the typed application-bootstrap sequence for forkwright crates.

Ties CLI/environment verbosity resolution, `figment`-backed config loading,
and `tracing` subscriber initialization into one call, `koinon::bootstrap::run`,
instead of leaving a binary to call three separate helpers and hope it
called them in the right order. That call is koinon's one invariant; the
individual modules below are the leaves it composes, still directly usable on
their own for a crate that genuinely only needs one of them (a library
doctest initializing telemetry with no config or CLI of its own, say).

Koinon does not define or own a binary's top-level application error - that
sum stays in the consumer, which wraps koinon's own `ConfigError` into it the
same way it wraps any other typed source.

## Modules

| Module | Purpose |
|--------|---------|
| `bootstrap` | `run` - the integrated CLI + config + telemetry sequence |
| `cli` | `GlobalArgs` (`--verbose`, `--log-json`) for `clap` CLI binaries |
| `config` | `figment` loader: TOML file → env-var override → typed struct |
| `telemetry` | `tracing` subscriber init with `RUST_LOG` / `EnvFilter` fallback |
| `error` | `ConfigError` - the one error type koinon semantically owns |

## Feature flags

`telemetry`, `config`, `cli`, and `bootstrap` are Cargo features, all on by
default; `error` is always available. Trim the dependency tree with
`default-features = false`:

<!-- x-release-please-start-version -->
```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.2.0", default-features = false, features = ["config"] }
```
<!-- x-release-please-end-version -->

| Feature | Pulls in |
|---------|----------|
| `telemetry` | `tracing`, `tracing-subscriber` |
| `config` | `figment`, `serde` |
| `cli` | `clap` (implies `telemetry`) |
| `bootstrap` | the `bootstrap` module (implies `cli` + `config`) |

## Quick start

<!-- x-release-please-start-version -->
```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.2.0" }
```
<!-- x-release-please-end-version -->

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

## Adoption guide

See [`ADOPTION.md`](ADOPTION.md) for the step-by-step migration from
hand-rolled tracing init and config loading to `koinon`.

## License

Apache-2.0 - see [`LICENSE`](LICENSE).

<!-- kanon:auto-start -->
## Repository Metadata

- Registry name: `koinon`
- Description: Kanon-managed forkwright repository `koinon`.
- Forge repo: `forkwright/koinon`
- Kanon prefix: `ko`
- Config source: `workflow/kanon.toml [projects.koinon]`
- Planning state: `projects/koinon/STATE.md`
- Last state update: `not recorded`

Run `kanon docs sync --check --repo koinon` to verify this generated
section and `kanon docs sync --apply --repo koinon` to refresh it.

## Blast zone

- Paths explicitly named by the rendered prompt, role, or template input.

## Acceptance verifier

```bash
kanon gate
```
<!-- kanon:auto-end -->

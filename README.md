# koinon

κοινόν - fleet-common Rust scaffolding for forkwright crates.

Eliminates the boilerplate that every forkwright binary and library repeats:
tracing subscriber setup, typed error bases, `figment`-backed config loading,
and a `clap` prelude for global flags.

## Modules

| Module | Purpose |
|--------|---------|
| `telemetry` | `tracing` subscriber init with `RUST_LOG` / `EnvFilter` fallback |
| `error` | `AppError`, `ConfigError`, and `snafu` re-exports |
| `config` | `figment` loader: TOML file → env-var override → typed struct |
| `cli` | `GlobalArgs` (`--verbose`, `--log-json`) for `clap` CLI binaries |

## Feature flags

`telemetry`, `config`, and `cli` are Cargo features, all on by default;
`error` is always available. Trim the dependency tree with
`default-features = false`:

```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.3", default-features = false, features = ["config"] }
```

| Feature | Pulls in |
|---------|----------|
| `telemetry` | `tracing`, `tracing-subscriber` |
| `config` | `figment`, `serde` |
| `cli` | `clap` (implies `telemetry`) |

## Quick start

```toml
[dependencies]
koinon = { git = "https://github.com/forkwright/koinon", tag = "v0.1.3" }
```

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
    tracing::info!("started");
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

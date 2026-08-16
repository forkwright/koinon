# koinon-adoption

Derives the "Fleet migration status" block in `../ADOPTION.md` - replaces
the hand-maintained matrix `forkwright/koinon#19` reported (Hamma marked
`done` by prose while only its `telemetry` column had actually moved).

## Mechanism

Two entry points, split because they have different truth sources and run
on different triggers:

| Command | Touches network | Runs |
|---|---|---|
| `koinon-adoption derive` | Yes - shallow-clones every repo in `registry.toml` | `.github/workflows/adoption-refresh.yml`, weekly + on demand |
| `koinon-adoption check` | No - reads the already-committed block | `.github/workflows/ci.yml`, every push/PR |

`derive` computes four facts per tracked repo, straight from its own
`Cargo.toml` / `Cargo.lock` at the observed commit - never from the repo's
name, README, or koinon's prior row for it:

- **Dependency** - does Cargo.lock actually resolve the *external*
  `forkwright/koinon` crate (a `git+https://github.com/forkwright/koinon`
  or, once published, `registry+` source)? A workspace-local crate that
  happens to share the name does not count - see the module docstring on
  `src/koinon_adoption/cargo.py` for why that distinction needs its own
  code path, verified against a real case in `forkwright/akroasis`.
- **Features** - the union of `koinon`'s Cargo features actually enabled
  across every crate that declares the dependency (`[dependencies]`,
  `[dev-dependencies]`, or `[build-dependencies]`; `workspace = true` is
  resolved against the workspace root).
- **Consumer reference** - the first source line matching
  `koinon::(telemetry|config|cli|error)`, restricted to files inside a
  crate that declared the external dependency. See
  `src/koinon_adoption/facts.py` for exactly what this regex does and does
  not match.
- **Source SHA + observed time** - the cloned commit and the UTC instant
  `derive` ran.

`check` never touches the network. It re-parses the already-committed block
and fails when:

- a row's repo doesn't match `registry.toml`'s order (hand edit, or
  `registry.toml` changed without a re-run);
- a Source SHA cell isn't a hex SHA or the `render.NO_VALUE` placeholder
  (the literal bug #19 reported: a prose word standing in for a derived fact);
- a row claims `resolved` but Features or Consumer reference is blank;
- any row's Observed timestamp is older than
  `koinon_adoption.freshness.DEFAULT_MAX_AGE` (14 days - `adoption-refresh`
  runs weekly, so this tolerates one missed run before paging anyone).

## Running it locally

```bash
cd adoption
uv run python -m koinon_adoption derive --dry-run   # print without writing
uv run python -m koinon_adoption derive --repo hamma --repo akroasis
uv run python -m koinon_adoption check
```

## Private tracked repos

Two registered repos (`gnomon`, `kanon`) are private. The scheduled
workflow's default `GITHUB_TOKEN` cannot read a different private repo in
the org no matter how it is passed - that restriction is a GitHub Actions
platform boundary, not a gap in this tool. Those rows report `unobserved
(private, no cross-repo credential)` unless a `KOINON_ADOPTION_TOKEN`
secret (a GitHub App installation token or a scoped PAT with read access to
those two repos) is provisioned - see `CROSS_REPO_TOKEN_ENV_VAR` in
`src/koinon_adoption/derive.py`. Provisioning that secret is an operator
action; this tool already reads it when present.

## Development

```bash
uv run pytest
uv run ruff check .
uv run ruff format --check .
uv run mypy .
```

`tests/fixtures/` holds real content captured from `forkwright/hamma` and
`forkwright/akroasis` (attributed at the top of each fixture file) - the
akroasis fixtures exist to prove `cargo.py` and `facts.py` do not mistake
that repo's unrelated local `crates/koinon` for this crate.
`tests/fixtures/adoption_stale_*.md` are negative-case fixtures for
`freshness.check_freshness`: each one names the exact violation it exists
to prove fails.

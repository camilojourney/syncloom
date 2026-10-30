# syncloom.v2

From-scratch Rust rebuild of Syncloom (formerly Holus), a solo founder's thought-to-content studio: one written thought becomes platform drafts the founder reviews and approves before an explicit publish. The old Python app lives in the separate `syncloom` repository and is not part of this one. What Syncloom is and is not: [docs/vision.md](docs/vision.md). What moved from the old app and what did not: [docs/migration.md](docs/migration.md).

## Layout

- `crates/syncloom-core/` - the rules: thought intake, drafting through a model provider, platform rules, review transitions, the contained dispatch outbox, the provenance log and the file store. The Claude adapter lives here behind the provider trait.
- `crates/syncloom-server/` - the loopback HTTP API (Axum), binary `syncloom-server`.
- `crates/syncloom-cli/` - the command line, binary `syncloom`; it calls the same core functions and returns the same errors.
- `contracts/` - fixtures the server tests against (`contracts/studio-api/`, owned by the capability `studio-api`).
- `openspec/` - specs and changes; `openspec/specs/` is the only specification authority.
- `docs/research/` and `docs/decisions/` - the studies behind the product and the decisions they served. Start at [`docs/research/INDEX.md`](docs/research/INDEX.md) and [`docs/decisions/README.md`](docs/decisions/README.md).
- `scripts/` - record checkers carried over from invoz.v2 ([docs/development.md](docs/development.md) says why they stay Python).

## Run and check

OpenSpec baseline, toolchain and how to re-check them: [docs/development.md](docs/development.md). `cargo` is reached through nix on this machine: `nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#clippy nixpkgs#rustfmt -c sh -c '<command>'`.

Server (needs `ANTHROPIC_API_KEY` in the environment to draft through Claude):

```sh
cargo run -p syncloom-server            # binds 127.0.0.1:8790, override with SYNCLOOM_ADDR
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Command line, same core, same errors:

```sh
cargo run -p syncloom-cli -- thought --text "<the thought>" --channel linkedin_text
cargo run -p syncloom-cli -- show <piece_id>
cargo run -p syncloom-cli -- approve <piece_id> --expected-revision <revision>
cargo run -p syncloom-cli -- publish <piece_id> --expected-revision <revision> --dry-run
```

Development fake Claude provider, to drive success and every provider error without a key:

```sh
cargo run -p syncloom-core --example fake_claude -- --mode ok     # ok|empty|slow|401|429|500, prints its address
SYNCLOOM_CLAUDE_BASE_URL=http://127.0.0.1:<port> ANTHROPIC_API_KEY=fake cargo run -p syncloom-server
```

Specs and records:

```sh
openspec validate --all --strict --no-interactive
python3 scripts/check-change-shape.py openspec/changes/<change> [--request-source FILE]   # config.yaml rules that validate does not enforce
python3 -I scripts/check-research-records.py
python3 -I scripts/test_check_research_records.py   # the check's own tests
```

## Environment

| Variable | Default | Meaning |
|---|---|---|
| `SYNCLOOM_ADDR` | `127.0.0.1:8790` | server bind address, must be on `127.0.0.1` |
| `SYNCLOOM_DATA_DIR` | `$HOME/.syncloom` | where pieces, outbox intents and the provenance log are stored |
| `ANTHROPIC_API_KEY` | none | Claude key, read only from the environment, never logged |
| `SYNCLOOM_CLAUDE_BASE_URL` | `https://api.anthropic.com` | provider URL; `https` or `http://127.0.0.1:<port>` only |
| `SYNCLOOM_CLAUDE_MODEL` | `claude-sonnet-5-5` | the pinned Claude Sonnet model id (unverified until the live check runs) |

# Contributing

## Quick Setup

```bash
git clone https://github.com/mnemebrain/mnemebrain-rust
cd mnemebrain-rust
cargo build
cargo test
```

## Code Style

```bash
cargo fmt                      # format
cargo clippy -- -D warnings    # lint — zero warnings required
```

All public items must have doc comments (`///`). No `unwrap()` in library code — use `?` or `expect("reason")`.

## Testing

**Unit tests** live alongside source code and use `wiremock` to mock HTTP responses.

## Test coverage

    ```bash
    cargo install cargo-tarpaulin   # one-time install
    cargo tarpaulin --out Html       # generates tarpaulin-report.html
    cargo tarpaulin --out Stdout     # print summary to terminal

**Integration tests** are in `tests/integration/` and test multi-step client workflows against a mock server.

**E2E tests** require a running server:

```bash
MNEMEBRAIN_URL=http://localhost:8000 cargo test e2e -- --ignored
MNEMEBRAIN_URL=http://localhost:8000 MNEMEBRAIN_VARIANT=lite cargo test e2e -- --ignored
```

New features must include unit tests. Bug fixes should include a regression test.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add bearer token refresh
fix: handle 429 rate-limit response
docs: add search example to README
test: cover retract with bare array response
refactor: extract request builder into separate module
chore: bump reqwest to 0.12
```

Keep the subject line under 72 characters. Add a body when the "why" is not obvious.

## Pull Requests

- One logical change per PR.
- All tests must pass (`cargo test`) and lint must be clean (`cargo clippy -- -D warnings`).
- New public APIs need doc comments and at least one unit test.
- Update `CHANGELOG.md` under `[Unreleased]`.

## Architecture

| File / Directory | Responsibility |
|---|---|
| `src/client.rs` | `MnemeBrainClient` + `MnemeBrainClientBuilder` — all HTTP dispatch |
| `src/models.rs` | Request/response types, typed enums, builder methods |
| `src/error.rs` | `MnemeBrainError` variants + `Result` alias |
| `src/subclient/` | Full-backend sub-clients (`sandbox`, `goals`, `policies`, etc.) |

The client holds a single `reqwest::Client`. All methods serialize to JSON, send, and deserialize. Sub-clients borrow an `Arc` to the inner client so they share the connection pool.

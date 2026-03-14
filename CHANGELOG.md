# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.2] - 2026-03-14

### Changed

- `MnemeBrainClient::new()` and `with_auth()` now take `Duration` instead of `u64` for timeout
- `believe()` now takes `&BelieveRequest` instead of 5 positional parameters
- `search()` now takes `&SearchRequest` instead of 4 positional parameters
- `frame_open()` now takes `&FrameOpenRequest` with builder pattern
- Sub-client methods use typed enums (`TruthState`, `BeliefType`, `AttackType`, `CommitMode`, `GoalStatus`, `PolicyStatus`) instead of `&str`
- `SandboxClient::revise()` takes `&EvidenceInput` instead of 5 separate parameters

### Added

- `MnemeBrainClientBuilder` — fluent client configuration with `Duration`-based timeout, auth, user agent
- `BelieveRequest` — typed request builder for `believe()` with defaults
- `SearchRequest` — typed request builder for `search()` with defaults
- Builder methods on `FrameOpenRequest`, `LiteFrameOpenRequest`, `BeliefFilters`
- Doc comments on `MnemeBrainError` variants explaining when each occurs
- `LICENSE` file (MIT)
- `CONTRIBUTING.md` with development guidelines

## [0.1.0-alpha.1] - 2026-03-14

### Added

- `MnemeBrainClient` — async HTTP client for MnemeBrain servers
- `MnemeBrainClient::with_auth()` — Bearer token authentication
- Core operations: `believe`, `explain`, `search`, `retract`, `revise`, `list_beliefs`
- Working memory frames: `frame_open`, `frame_open_lite`, `frame_add`, `frame_scratchpad`, `frame_context`, `frame_commit`, `frame_close`
- Lite API support via `LiteFrameOpenRequest` (query_id/goal_id/top_k)
- `retract()` handles both bare array (lite) and wrapped object (full) response formats
- Typed enums: `TruthState`, `BeliefType`, `Polarity`, `ConflictPolicy`, `AttackType`, `GoalStatus`, `PolicyStatus`, `SandboxStatus`, `CommitMode`
- Builder pattern on `EvidenceInput`: `with_polarity()`, `with_weight()`, `with_reliability()`, `with_scope()`
- Sub-clients for full backend: `SandboxClient`, `GoalClient`, `PolicyClient`, `RevisionClient`, `AttackClient`, `ReconsolidationClient`
- Utility endpoints: `reset`, `set_time_offset`, `consolidate`, `get_memory_tier`, `query_multihop`
- Benchmark endpoints: `benchmark_sandbox_fork/assume/resolve/discard`, `benchmark_attack`
- Comprehensive error types via `thiserror`: `MnemeBrainError::Http`, `Request`, `Json`, `Other`
- Unit tests with `wiremock` for all client methods
- Integration tests for multi-step workflows
- E2E test harness for testing against live servers (lite and full)
- GitHub Actions CI: lint, fmt, typecheck, test, security audit, release to crates.io

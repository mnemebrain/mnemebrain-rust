# mnemebrain

Rust SDK for [MnemeBrain](https://github.com/mnemebrain/mnemebrain) — belief-based memory for AI agents.

Provides a typed async HTTP client for both MnemeBrain Lite and the full MnemeBrain backend, with Belnap four-valued logic, append-only evidence, and working memory frames.

> **Alpha**: API may change before 0.1 stable.

## Install

```toml
[dependencies]
mnemebrain = "0.1.0-alpha.2"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use std::time::Duration;
use mnemebrain::{MnemeBrainClient, MnemeBrainClientBuilder, BelieveRequest, EvidenceInput, BeliefType};

#[tokio::main]
async fn main() -> mnemebrain::Result<()> {
    let client = MnemeBrainClientBuilder::new("http://localhost:8000")
        .timeout(Duration::from_secs(30))
        .api_key("your-api-key")
        .build();

    // Store a belief with typed request
    let ev = EvidenceInput::new("astronomy-textbook", "Confirmed by observation")
        .with_weight(0.95)
        .with_reliability(1.0);
    let request = BelieveRequest::new("Earth orbits the Sun", vec![ev])
        .with_belief_type(BeliefType::Fact)
        .with_source_agent("my-agent");
    let result = client.believe(&request).await?;
    println!("truth_state: {:?}, confidence: {}", result.truth_state, result.confidence);

    // Explain a belief
    if let Some(explanation) = client.explain("Earth orbits the Sun").await? {
        println!("supporting evidence: {}", explanation.supporting.len());
    }

    // Search beliefs with typed request
    use mnemebrain::{SearchRequest, ConflictPolicy};
    let search = SearchRequest::new("astronomy")
        .with_limit(10)
        .with_conflict_policy(ConflictPolicy::Surface);
    let results = client.search(&search).await?;
    for r in &results.results {
        println!("{}: {:?} ({})", r.claim, r.truth_state, r.rank_score);
    }

    // Retract evidence
    if let Some(ids) = &result.evidence_ids {
        let affected = client.retract(&ids[0]).await?;
        println!("affected beliefs: {}", affected.len());
    }

    Ok(())
}
```

## Client Configuration

```rust
use std::time::Duration;

// Simple (no auth)
let client = MnemeBrainClient::new("http://localhost:8000", Duration::from_secs(30));

// With auth
let client = MnemeBrainClient::with_auth("http://localhost:8000", "key", Duration::from_secs(30));

// Builder (recommended)
let client = MnemeBrainClientBuilder::new("http://localhost:8000")
    .timeout(Duration::from_secs(60))
    .api_key("your-api-key")
    .user_agent("my-agent/1.0")
    .build();
```

## Core Operations

- `believe(&BelieveRequest)` — store a belief with evidence
- `explain(claim)` — get truth state + evidence breakdown
- `search(&SearchRequest)` — semantic belief search
- `retract(evidence_id)` — retract evidence
- `revise(belief_id, &EvidenceInput)` — add evidence to existing belief
- `list_beliefs(&BeliefFilters)` — paginated belief listing

## Working Memory Frames

Frames provide an active context buffer for multi-step reasoning:

```rust
use mnemebrain::FrameOpenRequest;

// Full backend API
let req = FrameOpenRequest::new("dietary preferences")
    .with_ttl(300)
    .with_agent("agent-1");
let frame = client.frame_open(&req).await?;

// Lite API
use mnemebrain::LiteFrameOpenRequest;
let req = LiteFrameOpenRequest::new("query-uuid");
let frame = client.frame_open_lite(&req).await?;

// Add beliefs, write scratchpad, get context
client.frame_add(&frame.frame_id, "user is vegetarian").await?;
client.frame_scratchpad(&frame.frame_id, "step", serde_json::json!("analyzing")).await?;
let ctx = client.frame_context(&frame.frame_id).await?;

// Commit or close
client.frame_close(&frame.frame_id).await?;
```

## Sub-clients (Full Backend)

These methods are available against the full MnemeBrain backend only. Calling them against MnemeBrain Lite will return `MnemeBrainError::Http { status: 404, .. }`.

```rust
// Sandbox — what-if reasoning
let sb = client.sandbox().fork(None, "test-scenario", 300).await?;
client.sandbox().assume(&sb.id, "belief-id", "false").await?;
let diff = client.sandbox().diff(&sb.id).await?;

// Goals
let goal = client.goals().create("reach 100 users", "agent-1", 0.8, None, None).await?;

// Policies
let policy = client.policies().list().await?;

// Revision
client.revision().set_policy("bounded", Some(3), Some(10)).await?;

// Attacks
let edges = client.attacks().list("belief-id").await?;

// Reconsolidation
let queue = client.reconsolidation().queue().await?;
```

## Error Handling

```rust
use mnemebrain::MnemeBrainError;

match client.health().await {
    Ok(resp) => println!("status: {}", resp.status),
    Err(MnemeBrainError::Http { status, message }) => {
        // Server returned 4xx/5xx
        eprintln!("server error {}: {}", status, message);
    }
    Err(MnemeBrainError::Request(e)) => {
        // Network/transport error (DNS, connection, timeout)
        eprintln!("network error: {}", e);
    }
    Err(MnemeBrainError::Json(e)) => {
        // Response deserialization failed
        eprintln!("parse error: {}", e);
    }
    Err(MnemeBrainError::Other(msg)) => {
        eprintln!("other: {}", msg);
    }
}
```

## API Compatibility

| Feature | Lite | Full Backend |
|---------|------|-------------|
| believe, retract, explain, revise | Yes | Yes |
| search, list beliefs | Yes | Yes |
| Working memory frames | Yes | Yes |
| Consolidation, memory tiers | — | Yes |
| Sandbox, goals, policies | — | Yes |
| Revision, attacks, reconsolidation | — | Yes |
| Multihop queries | — | Yes |
| Benchmark endpoints | — | Yes |

Calling a full-backend-only method against MnemeBrain Lite will return `MnemeBrainError::Http { status: 404, .. }`.

## Data Model

**Truth States** (Belnap four-valued logic): `True`, `False`, `Both` (conflict), `Neither` (insufficient evidence)

**Belief Types**: `Fact` (365d half-life), `Preference` (90d), `Inference` (30d), `Prediction` (3d)

**Confidence**: Ranking signal via log-odds with sigmoid. `0.5` = maximum uncertainty.

## Development

```bash
cargo test                    # unit + integration tests
cargo clippy -- -D warnings   # lint
cargo fmt -- --check          # format check
cargo audit                   # security audit
```

### E2E tests

```bash
MNEMEBRAIN_URL=http://localhost:8000 cargo test e2e -- --ignored
MNEMEBRAIN_URL=http://localhost:8000 MNEMEBRAIN_VARIANT=lite cargo test e2e -- --ignored
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development guide.

## License

[MIT](LICENSE)

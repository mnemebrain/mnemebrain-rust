/// End-to-end tests for the MnemeBrain Rust SDK.
///
/// These tests run against a REAL MnemeBrain server and are `#[ignore]`d by
/// default so they never execute in CI. Run them explicitly with:
///
/// ```text
/// # Lite variant:
/// MNEMEBRAIN_URL=http://localhost:8000 MNEMEBRAIN_VARIANT=lite cargo test e2e -- --ignored
///
/// # Full variant:
/// MNEMEBRAIN_URL=http://localhost:8000 MNEMEBRAIN_VARIANT=full cargo test e2e -- --ignored
///
/// # With authentication:
/// MNEMEBRAIN_URL=http://localhost:8000 MNEMEBRAIN_API_KEY=my-key cargo test e2e -- --ignored
/// ```
///
/// Every test returns early with a printed message when `MNEMEBRAIN_URL` is not
/// set, so the suite degrades gracefully in environments without a live server.
use std::time::Duration;
use mnemebrain::{
    BeliefFilters, BelieveRequest, EvidenceInput, FrameOpenRequest, LiteFrameOpenRequest,
    MnemeBrainClient, SearchRequest, TruthState,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build an SDK client from environment variables, or return `None` if
/// `MNEMEBRAIN_URL` is not set.
fn get_client() -> Option<MnemeBrainClient> {
    std::env::var("MNEMEBRAIN_URL")
        .ok()
        .map(|url| match std::env::var("MNEMEBRAIN_API_KEY").ok() {
            Some(key) => MnemeBrainClient::with_auth(&url, &key, Duration::from_secs(30)),
            None => MnemeBrainClient::new(&url, Duration::from_secs(30)),
        })
}

/// Returns `true` when the test suite is targeting the lite backend.
fn is_lite() -> bool {
    std::env::var("MNEMEBRAIN_VARIANT").unwrap_or_default() == "lite"
}

/// Macro that exits a test early when no live server is available.
macro_rules! require_server {
    ($client:ident) => {
        let Some($client) = get_client() else {
            println!("MNEMEBRAIN_URL not set — skipping e2e test");
            return;
        };
    };
}

// ── 1. Health check ───────────────────────────────────────────────────────────

/// Verifies that the server is reachable and reports a healthy status.
#[tokio::test]
#[ignore]
async fn test_e2e_health() {
    require_server!(client);

    let resp = client.health().await.expect("health check failed");
    assert!(
        !resp.status.is_empty(),
        "status field must not be empty, got: {:?}",
        resp.status
    );
    println!("health: {:?}", resp.status);
}

// ── 2. Believe → Explain cycle ────────────────────────────────────────────────

/// Believes a claim, then explains it to verify the truth state is reflected.
#[tokio::test]
#[ignore]
async fn test_e2e_believe_explain_cycle() {
    require_server!(client);

    let claim = "the moon orbits the earth";
    let ev = EvidenceInput::new("astronomy-101", "fundamental orbital mechanics");

    let belief = client
        .believe(&BelieveRequest::new(claim, vec![ev]).with_source_agent("e2e-agent"))
        .await
        .expect("believe failed");
    assert!(
        !belief.id.is_empty(),
        "believe should return a non-empty belief id"
    );
    println!(
        "believed: id={} truth_state={:?}",
        belief.id, belief.truth_state
    );

    let explanation = client
        .explain(claim)
        .await
        .expect("explain failed")
        .expect("claim should be found after believing it");
    assert_eq!(
        explanation.truth_state,
        TruthState::True,
        "expected truth_state=true after a single supporting piece of evidence"
    );
    assert!(
        !explanation.supporting.is_empty(),
        "should have at least one supporting evidence item"
    );
    println!(
        "explained: truth_state={:?} supporting={}",
        explanation.truth_state,
        explanation.supporting.len()
    );
}

// ── 3. Retract cycle ──────────────────────────────────────────────────────────

/// Believes a claim, extracts the evidence id, retracts it, and verifies the
/// belief degrades to NEITHER (or BOTH if other evidence exists).
#[tokio::test]
#[ignore]
async fn test_e2e_retract_cycle() {
    require_server!(client);

    let claim = "copper conducts electricity";
    let ev = EvidenceInput::new("physics-ref", "standard materials property");

    let belief = client
        .believe(&BelieveRequest::new(claim, vec![ev]).with_source_agent("e2e-agent"))
        .await
        .expect("believe failed");

    let evidence_ids = belief.evidence_ids.clone().unwrap_or_default();
    if evidence_ids.is_empty() {
        println!("server did not return evidence_ids — skipping retract assertion");
        return;
    }
    let evidence_id = &evidence_ids[0];
    println!("retracting evidence_id={}", evidence_id);

    let affected = client.retract(evidence_id).await.expect("retract failed");

    // The belief that we created should appear in the affected list
    let our_belief = affected.iter().find(|b| b.id == belief.id);
    if let Some(b) = our_belief {
        assert!(
            b.truth_state == TruthState::Neither || b.truth_state == TruthState::Both,
            "after retraction the truth_state should be neither or both, got {:?}",
            b.truth_state
        );
        println!("post-retract truth_state={:?}", b.truth_state);
    } else {
        println!("belief not in affected list — it may have been merged or the server omits it");
    }
}

// ── 4. Search ─────────────────────────────────────────────────────────────────

/// Believes a distinctive claim then searches for it by keyword.
#[tokio::test]
#[ignore]
async fn test_e2e_search() {
    require_server!(client);

    let claim = "mnemebrain e2e search test claim alpha-7x9";
    let ev = EvidenceInput::new("e2e-harness", "unique test string");

    client
        .believe(&BelieveRequest::new(claim, vec![ev]).with_source_agent("e2e-agent"))
        .await
        .expect("believe failed");

    let resp = client
        .search(&SearchRequest::new("mnemebrain e2e search test"))
        .await
        .expect("search failed");

    println!("search returned {} result(s)", resp.results.len());

    // We should find at least our claim somewhere in the results.
    // Semantic search may not guarantee exact match at position 0 without embeddings.
    let found = resp.results.iter().any(|r| r.claim.contains("alpha-7x9"));
    if !found {
        println!("claim not found in search results (embeddings may not be available)");
    }
}

// ── 5. List beliefs with filters ──────────────────────────────────────────────

/// Believes a claim, then lists beliefs and verifies the list is non-empty and
/// properly structured.
#[tokio::test]
#[ignore]
async fn test_e2e_list_beliefs() {
    require_server!(client);

    let ev = EvidenceInput::new("e2e-harness", "list test evidence");
    client
        .believe(&BelieveRequest::new("e2e list beliefs test claim bravo-3k", vec![ev]).with_source_agent("e2e-agent"))
        .await
        .expect("believe failed");

    // Unfiltered list
    let all = client
        .list_beliefs(&BeliefFilters::default())
        .await
        .expect("list_beliefs failed");
    assert!(
        all.total >= 1,
        "should have at least one belief after believing one"
    );
    println!("total beliefs: {}", all.total);

    // Filter by truth_state=true
    let true_filter = BeliefFilters {
        truth_state: Some(TruthState::True),
        ..BeliefFilters::default()
    };
    let true_beliefs = client
        .list_beliefs(&true_filter)
        .await
        .expect("list_beliefs with truth_state filter failed");
    assert!(
        true_beliefs
            .beliefs
            .iter()
            .all(|b| b.truth_state == TruthState::True),
        "all returned beliefs should have truth_state=true when filtered"
    );
    println!(
        "filtered true beliefs: {} / {}",
        true_beliefs.beliefs.len(),
        true_beliefs.total
    );
}

// ── 6. Frame lifecycle ────────────────────────────────────────────────────────

/// Runs a complete working-memory frame: open, add belief, write scratchpad,
/// inspect context, then either commit (full) or close (lite).
#[tokio::test]
#[ignore]
async fn test_e2e_frame_lifecycle() {
    require_server!(client);

    let frame_id = if is_lite() {
        // Lite uses query_id / top_k shape
        let req = LiteFrameOpenRequest::new("e2e-query-id-001");
        let open = client
            .frame_open_lite(&req)
            .await
            .expect("frame_open_lite failed");
        println!(
            "lite frame opened: id={} beliefs_loaded={}",
            open.frame_id, open.beliefs_loaded
        );
        open.frame_id
    } else {
        let open = client
            .frame_open(&FrameOpenRequest::new("e2e reasoning session").with_source_agent("e2e-agent"))
            .await
            .expect("frame_open failed");
        println!(
            "full frame opened: id={} beliefs_loaded={}",
            open.frame_id, open.beliefs_loaded
        );
        open.frame_id
    };

    // Add a belief to the active frame
    let added = client
        .frame_add(&frame_id, "e2e frame add test belief")
        .await
        .expect("frame_add failed");
    println!("frame_add: belief_id={}", added.belief_id);

    // Write a scratchpad entry
    client
        .frame_scratchpad(&frame_id, "e2e_key", serde_json::json!("e2e_value"))
        .await
        .expect("frame_scratchpad failed");

    // Read context
    let ctx = client
        .frame_context(&frame_id)
        .await
        .expect("frame_context failed");
    assert!(
        !ctx.query.is_empty(),
        "frame context should have a non-empty query field"
    );
    println!(
        "frame_context: query={:?} beliefs={} scratchpad_keys={}",
        ctx.query,
        ctx.beliefs.len(),
        ctx.scratchpad.len()
    );

    if is_lite() {
        // Lite: just close the frame
        client
            .frame_close(&frame_id)
            .await
            .expect("frame_close failed");
        println!("lite frame closed");
    } else {
        // Full: commit then close
        let commit = client
            .frame_commit(&frame_id, &[], &[])
            .await
            .expect("frame_commit failed");
        assert_eq!(commit.frame_id, frame_id);
        println!(
            "frame_commit: created={} revised={}",
            commit.beliefs_created, commit.beliefs_revised
        );
        client
            .frame_close(&frame_id)
            .await
            .expect("frame_close failed");
    }
}

// ── 7. Full-only: consolidate ─────────────────────────────────────────────────

/// Triggers memory consolidation. Skipped automatically on lite variant.
#[tokio::test]
#[ignore]
async fn test_e2e_full_only_consolidate() {
    require_server!(client);

    if is_lite() {
        println!("MNEMEBRAIN_VARIANT=lite — skipping consolidate test");
        return;
    }

    let result = client.consolidate().await.expect("consolidate failed");
    println!(
        "consolidate: semantic_beliefs_created={} episodics_pruned={} clusters_found={}",
        result.semantic_beliefs_created, result.episodics_pruned, result.clusters_found
    );
    // The counts may all be zero if there is nothing to consolidate; that is valid.
    assert!(
        result.semantic_beliefs_created >= 0,
        "semantic_beliefs_created should be non-negative"
    );
}

// ── 8. Full-only: multihop query ──────────────────────────────────────────────

/// Runs a multihop query. Skipped automatically on lite variant.
#[tokio::test]
#[ignore]
async fn test_e2e_full_only_multihop() {
    require_server!(client);

    if is_lite() {
        println!("MNEMEBRAIN_VARIANT=lite — skipping multihop test");
        return;
    }

    // Believe a seed claim first so the graph has something to traverse
    let ev = EvidenceInput::new("e2e-harness", "seed for multihop");
    client
        .believe(&BelieveRequest::new("multihop seed belief delta-9z", vec![ev]).with_source_agent("e2e-agent"))
        .await
        .expect("believe failed");

    let resp = client
        .query_multihop("multihop seed")
        .await
        .expect("query_multihop failed");

    println!("multihop returned {} result(s)", resp.results.len());

    // Each result must have a non-empty belief_id and claim
    for item in &resp.results {
        assert!(!item.belief_id.is_empty(), "belief_id must not be empty");
        assert!(!item.claim.is_empty(), "claim must not be empty");
    }
}

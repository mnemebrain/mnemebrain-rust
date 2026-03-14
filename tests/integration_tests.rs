use mnemebrain::{
    BeliefFilters, BeliefType, BelieveRequest, CommitMode, EvidenceInput, FrameOpenRequest,
    GoalStatus, LiteFrameOpenRequest, MnemeBrainClient, Polarity, PolicyStatus, SearchRequest,
    TruthState,
};
use serde_json::json;
/// Integration tests for the MnemeBrain Rust SDK.
///
/// These tests use `wiremock` to simulate realistic multi-step workflows against
/// mocked servers. Each test exercises a sequence of SDK calls rather than a
/// single endpoint in isolation.
use std::time::Duration;
use wiremock::matchers::{header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn belief_json(id: &str, truth_state: &str, confidence: f64, conflict: bool) -> serde_json::Value {
    json!({
        "id": id,
        "truth_state": truth_state,
        "confidence": confidence,
        "conflict": conflict,
        "was_separated": false,
        "memory_tier": "episodic",
        "evidence_ids": []
    })
}

fn evidence_detail_json(
    id: &str,
    source_ref: &str,
    content: &str,
    polarity: &str,
) -> serde_json::Value {
    json!({
        "id": id,
        "source_ref": source_ref,
        "content": content,
        "polarity": polarity,
        "weight": 0.7,
        "reliability": 0.8
    })
}

fn belief_snapshot_json(id: &str, claim: &str, truth_state: &str) -> serde_json::Value {
    json!({
        "belief_id": id,
        "claim": claim,
        "truth_state": truth_state,
        "confidence": 0.85,
        "belief_type": "fact",
        "evidence_count": 1,
        "conflict": false
    })
}

// ── 1. Believe → Explain → Retract workflow ───────────────────────────────────

#[tokio::test]
async fn test_believe_explain_retract_workflow() {
    let mock_server = MockServer::start().await;

    // Step 1: believe the claim — returns evidence id "e-sky-1"
    Mock::given(method("POST"))
        .and(path("/believe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "b-sky-1",
            "truth_state": "true",
            "confidence": 0.88,
            "conflict": false,
            "was_separated": false,
            "memory_tier": "episodic",
            "evidence_ids": ["e-sky-1"]
        })))
        .mount(&mock_server)
        .await;

    // Step 2: explain — shows the supporting evidence
    Mock::given(method("GET"))
        .and(path("/explain"))
        .and(query_param("claim", "the sky is blue"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "claim": "the sky is blue",
            "truth_state": "true",
            "confidence": 0.88,
            "supporting": [
                evidence_detail_json("e-sky-1", "observation", "looked up and saw blue sky", "supports")
            ],
            "attacking": [],
            "expired": []
        })))
        .mount(&mock_server)
        .await;

    // Step 3: retract evidence — belief collapses to NEITHER
    Mock::given(method("POST"))
        .and(path("/retract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "affected_beliefs": [
                belief_json("b-sky-1", "neither", 0.0, false)
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ev = EvidenceInput::new("observation", "looked up and saw blue sky");

    // believe
    let belief = client
        .believe(&BelieveRequest::new("the sky is blue", vec![ev]).with_source_agent("agent-1"))
        .await
        .unwrap();
    assert_eq!(belief.id, "b-sky-1");
    assert_eq!(belief.truth_state, TruthState::True);
    let evidence_id = belief.evidence_ids.unwrap().into_iter().next().unwrap();
    assert_eq!(evidence_id, "e-sky-1");

    // explain — supporting evidence present
    let explanation = client.explain("the sky is blue").await.unwrap().unwrap();
    assert_eq!(explanation.truth_state, TruthState::True);
    assert_eq!(explanation.supporting.len(), 1);
    assert_eq!(explanation.attacking.len(), 0);

    // retract evidence — truth state should be NEITHER
    let affected = client.retract(&evidence_id).await.unwrap();
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].id, "b-sky-1");
    assert_eq!(affected[0].truth_state, TruthState::Neither);
    assert!((affected[0].confidence - 0.0).abs() < 0.001);
}

// ── 2. Believe multiple claims → search → list with filter ───────────────────

#[tokio::test]
async fn test_search_and_filter_workflow() {
    let mock_server = MockServer::start().await;

    // Believe claim A
    Mock::given(method("POST"))
        .and(path("/believe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(belief_json(
            "b-water-1",
            "true",
            0.95,
            false,
        )))
        .expect(2) // called twice (one per claim)
        .mount(&mock_server)
        .await;

    // Search for "water"
    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("query", "water"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "belief_id": "b-water-1",
                    "claim": "water is wet",
                    "truth_state": "true",
                    "confidence": 0.95,
                    "similarity": 0.98,
                    "rank_score": 0.96
                },
                {
                    "belief_id": "b-water-2",
                    "claim": "water freezes at 0 degrees Celsius",
                    "truth_state": "true",
                    "confidence": 0.92,
                    "similarity": 0.87,
                    "rank_score": 0.89
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // list_beliefs filtered to "true" truth_state
    Mock::given(method("GET"))
        .and(path("/beliefs"))
        .and(query_param("truth_state", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "beliefs": [
                {
                    "id": "b-water-1",
                    "claim": "water is wet",
                    "belief_type": "fact",
                    "truth_state": "true",
                    "confidence": 0.95,
                    "tag_count": 0,
                    "evidence_count": 1,
                    "created_at": "2026-01-01T00:00:00Z",
                    "last_revised": "2026-01-01T00:00:00Z"
                },
                {
                    "id": "b-water-2",
                    "claim": "water freezes at 0 degrees Celsius",
                    "belief_type": "fact",
                    "truth_state": "true",
                    "confidence": 0.92,
                    "tag_count": 0,
                    "evidence_count": 1,
                    "created_at": "2026-01-01T00:00:00Z",
                    "last_revised": "2026-01-01T00:00:00Z"
                }
            ],
            "total": 2,
            "offset": 0,
            "limit": 50
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ev = EvidenceInput::new("chemistry-textbook", "well established");

    // Believe two claims
    client
        .believe(
            &BelieveRequest::new("water is wet", vec![ev.clone()]).with_source_agent("agent-1"),
        )
        .await
        .unwrap();
    client
        .believe(
            &BelieveRequest::new("water freezes at 0 degrees Celsius", vec![ev])
                .with_source_agent("agent-1"),
        )
        .await
        .unwrap();

    // Search
    let search_resp = client.search(&SearchRequest::new("water")).await.unwrap();
    assert_eq!(search_resp.results.len(), 2);
    assert!(search_resp.results[0].similarity > search_resp.results[1].similarity);

    // Filter list to only "true" beliefs
    let filters = BeliefFilters {
        truth_state: Some(TruthState::True),
        ..BeliefFilters::default()
    };
    let list_resp = client.list_beliefs(&filters).await.unwrap();
    assert_eq!(list_resp.total, 2);
    assert!(list_resp
        .beliefs
        .iter()
        .all(|b| b.truth_state == TruthState::True));
}

// ── 3. Full frame lifecycle (full-backend shape) ──────────────────────────────

#[tokio::test]
async fn test_frame_full_lifecycle() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/frame/open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "frame_id": "f-full-1",
            "beliefs_loaded": 2,
            "conflicts": 0,
            "snapshots": [
                belief_snapshot_json("b-1", "memory is persistent", "true"),
                belief_snapshot_json("b-2", "inference is fallible", "true")
            ]
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/frame/f-full-1/add"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(belief_snapshot_json(
                "b-3",
                "reasoning requires evidence",
                "neither",
            )),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/frame/f-full-1/scratchpad"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/frame/f-full-1/context"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query": "what do we know about reasoning?",
            "beliefs": [
                belief_snapshot_json("b-1", "memory is persistent", "true"),
                belief_snapshot_json("b-2", "inference is fallible", "true"),
                belief_snapshot_json("b-3", "reasoning requires evidence", "neither")
            ],
            "scratchpad": {"step": "analysis"},
            "conflicts": [],
            "step_count": 1
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/frame/f-full-1/commit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "frame_id": "f-full-1",
            "beliefs_created": 1,
            "beliefs_revised": 0
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/frame/f-full-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    // Open with full-backend shape
    let open = client
        .frame_open(
            &FrameOpenRequest::new("what do we know about reasoning?")
                .with_preload_claims(vec![
                    "memory is persistent".into(),
                    "inference is fallible".into(),
                ])
                .with_source_agent("agent-reasoner"),
        )
        .await
        .unwrap();
    assert_eq!(open.frame_id, "f-full-1");
    assert_eq!(open.beliefs_loaded, 2);
    assert_eq!(open.snapshots.len(), 2);

    // Add a belief to the frame
    let added = client
        .frame_add("f-full-1", "reasoning requires evidence")
        .await
        .unwrap();
    assert_eq!(added.belief_id, "b-3");

    // Write to scratchpad
    client
        .frame_scratchpad("f-full-1", "step", json!("analysis"))
        .await
        .unwrap();

    // Inspect context
    let ctx = client.frame_context("f-full-1").await.unwrap();
    assert_eq!(ctx.query, "what do we know about reasoning?");
    assert_eq!(ctx.beliefs.len(), 3);
    assert_eq!(ctx.step_count, 1);
    assert!(ctx.scratchpad.contains_key("step"));

    // Commit new belief
    let new_belief = json!({
        "claim": "reasoning requires evidence",
        "belief_type": "inference",
        "evidence": [{"source_ref": "frame", "content": "derived in session"}]
    });
    let commit = client
        .frame_commit("f-full-1", &[new_belief], &[])
        .await
        .unwrap();
    assert_eq!(commit.frame_id, "f-full-1");
    assert_eq!(commit.beliefs_created, 1);
    assert_eq!(commit.beliefs_revised, 0);

    // Close the frame
    client.frame_close("f-full-1").await.unwrap();
}

// ── 4. Lite frame lifecycle ───────────────────────────────────────────────────

#[tokio::test]
async fn test_lite_frame_lifecycle() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/frame/open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "frame_id": "f-lite-7",
            "beliefs_loaded": 3,
            "conflicts": 0,
            "snapshots": []
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/frame/f-lite-7/add"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(belief_snapshot_json(
                "b-lite-1",
                "the user prefers brevity",
                "true",
            )),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/frame/f-lite-7/scratchpad"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/frame/f-lite-7/context"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "active_query": "q-uuid-999",
            "beliefs": [belief_snapshot_json("b-lite-1", "the user prefers brevity", "true")],
            "scratchpad": {"note": "keep it short"},
            "conflicts": [],
            "step_count": 2
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/frame/f-lite-7"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    // Open via lite shape
    let req = LiteFrameOpenRequest {
        query_id: "q-uuid-999".into(),
        goal_id: Some("g-uuid-001".into()),
        top_k: 10,
        ttl_seconds: 120,
    };
    let open = client.frame_open_lite(&req).await.unwrap();
    assert_eq!(open.frame_id, "f-lite-7");
    assert_eq!(open.beliefs_loaded, 3);

    // Add a belief
    let added = client
        .frame_add("f-lite-7", "the user prefers brevity")
        .await
        .unwrap();
    assert_eq!(added.claim, "the user prefers brevity");

    // Write a scratchpad note
    client
        .frame_scratchpad("f-lite-7", "note", json!("keep it short"))
        .await
        .unwrap();

    // Read context — lite server returns "active_query" alias
    let ctx = client.frame_context("f-lite-7").await.unwrap();
    assert_eq!(ctx.query, "q-uuid-999");
    assert_eq!(ctx.beliefs.len(), 1);
    assert_eq!(ctx.step_count, 2);

    // Close without committing
    client.frame_close("f-lite-7").await.unwrap();
}

// ── 5. Conflict detection workflow ───────────────────────────────────────────

#[tokio::test]
async fn test_conflict_detection_workflow() {
    let mock_server = MockServer::start().await;

    // Believe a claim with strong supporting evidence
    Mock::given(method("POST"))
        .and(path("/believe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "b-claim-1",
            "truth_state": "true",
            "confidence": 0.9,
            "conflict": false,
            "was_separated": false,
            "memory_tier": "episodic",
            "evidence_ids": ["e-support-1"]
        })))
        .mount(&mock_server)
        .await;

    // Revise with attacking evidence — belief becomes BOTH (conflict)
    Mock::given(method("POST"))
        .and(path("/revise"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "b-claim-1",
            "truth_state": "both",
            "confidence": 0.5,
            "conflict": true,
            "was_separated": false,
            "memory_tier": "episodic",
            "evidence_ids": ["e-support-1", "e-attack-1"]
        })))
        .mount(&mock_server)
        .await;

    // Explain shows both supporting and attacking evidence
    Mock::given(method("GET"))
        .and(path("/explain"))
        .and(query_param("claim", "protein causes cancer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "claim": "protein causes cancer",
            "truth_state": "both",
            "confidence": 0.5,
            "supporting": [
                evidence_detail_json("e-support-1", "study-A", "correlation found", "supports")
            ],
            "attacking": [
                evidence_detail_json("e-attack-1", "study-B", "no causal link", "attacks")
            ],
            "expired": []
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    // Believe with supporting evidence
    let ev_support = EvidenceInput::new("study-A", "correlation found");
    let belief = client
        .believe(
            &BelieveRequest::new("protein causes cancer", vec![ev_support])
                .with_belief_type(BeliefType::Inference)
                .with_source_agent("researcher"),
        )
        .await
        .unwrap();
    assert_eq!(belief.truth_state, TruthState::True);
    assert!(!belief.conflict);

    // Revise with an attacking piece of evidence
    let ev_attack = EvidenceInput::new("study-B", "no causal link")
        .with_polarity(Polarity::Attacks)
        .with_weight(0.85);
    let revised = client.revise("b-claim-1", &ev_attack).await.unwrap();
    assert_eq!(revised.truth_state, TruthState::Both);
    assert!(revised.conflict);
    assert!((revised.confidence - 0.5).abs() < 0.01);

    // Explain reveals both sides
    let explanation = client
        .explain("protein causes cancer")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(explanation.truth_state, TruthState::Both);
    assert_eq!(explanation.supporting.len(), 1);
    assert_eq!(explanation.attacking.len(), 1);
}

// ── 6. Sandbox: fork → assume → diff → commit ────────────────────────────────

#[tokio::test]
async fn test_sandbox_fork_assume_diff_commit() {
    let mock_server = MockServer::start().await;

    // Fork sandbox
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/fork"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sb-1",
            "frame_id": null,
            "scenario_label": "hypothesis-test",
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z",
            "expires_at": "2026-01-01T01:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    // Assume a truth state for a belief in the sandbox
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/assume"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    // Get diff — one belief changed
    Mock::given(method("GET"))
        .and(path("/api/mneme/sandbox/sb-1/diff"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "belief_changes": [
                {
                    "belief_id": "b-target",
                    "field": "truth_state",
                    "old_value": "neither",
                    "new_value": "true"
                }
            ],
            "evidence_invalidations": [],
            "new_beliefs": [],
            "temporary_attacks": [],
            "goal_changes": [],
            "summary": "1 belief forced to true"
        })))
        .mount(&mock_server)
        .await;

    // Commit sandbox
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/commit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandbox_id": "sb-1",
            "committed_belief_ids": ["b-target"],
            "conflicts": []
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    // Fork
    let sandbox = client
        .sandbox()
        .fork(None, "hypothesis-test", 3600)
        .await
        .unwrap();
    assert_eq!(sandbox.id, "sb-1");
    assert_eq!(sandbox.scenario_label, "hypothesis-test");

    // Assume belief truth state
    client
        .sandbox()
        .assume("sb-1", "b-target", TruthState::True)
        .await
        .unwrap();

    // Inspect diff
    let diff = client.sandbox().diff("sb-1").await.unwrap();
    assert_eq!(diff.belief_changes.len(), 1);
    assert_eq!(diff.belief_changes[0].belief_id, "b-target");
    assert_eq!(diff.belief_changes[0].field, "truth_state");
    assert_eq!(diff.belief_changes[0].new_value, "true");
    assert!(!diff.summary.is_empty());

    // Commit
    let commit = client
        .sandbox()
        .commit("sb-1", CommitMode::All, None)
        .await
        .unwrap();
    assert_eq!(commit.sandbox_id, "sb-1");
    assert_eq!(commit.committed_belief_ids, vec!["b-target"]);
    assert!(commit.conflicts.is_empty());
}

// ── 7. Goal lifecycle ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_goal_lifecycle() {
    let mock_server = MockServer::start().await;

    // Create goal
    Mock::given(method("POST"))
        .and(path("/api/mneme/goals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "g-mission-1",
            "goal": "ship the Rust SDK",
            "owner": "team-backend",
            "priority": 0.9,
            "status": "active",
            "created_at": "2026-03-14T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    // Evaluate goal
    Mock::given(method("GET"))
        .and(path("/api/mneme/goals/g-mission-1/evaluate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "goal_id": "g-mission-1",
            "status": "active",
            "completion_fraction": 0.75,
            "blocking_belief_ids": [],
            "supporting_belief_ids": ["b-tests-written"]
        })))
        .mount(&mock_server)
        .await;

    // Update status to completed
    Mock::given(method("PATCH"))
        .and(path("/api/mneme/goals/g-mission-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "g-mission-1",
            "goal": "ship the Rust SDK",
            "owner": "team-backend",
            "priority": 0.9,
            "status": "completed",
            "created_at": "2026-03-14T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    // Create
    let goal = client
        .goals()
        .create("ship the Rust SDK", "team-backend", 0.9, None, None)
        .await
        .unwrap();
    assert_eq!(goal.id, "g-mission-1");
    assert_eq!(goal.goal, "ship the Rust SDK");
    assert_eq!(goal.status, mnemebrain::GoalStatus::Active);

    // Evaluate
    let eval = client.goals().evaluate("g-mission-1").await.unwrap();
    assert_eq!(eval.goal_id, "g-mission-1");
    assert!((eval.completion_fraction - 0.75).abs() < 0.001);
    assert!(eval.blocking_belief_ids.is_empty());
    assert_eq!(eval.supporting_belief_ids.len(), 1);

    // Complete
    let updated = client
        .goals()
        .update_status("g-mission-1", GoalStatus::Completed)
        .await
        .unwrap();
    assert_eq!(updated.status, mnemebrain::GoalStatus::Completed);
}

// ── 8. Policy lifecycle ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_policy_lifecycle() {
    let mock_server = MockServer::start().await;

    let policy_body = json!({
        "id": "p-safe-1",
        "name": "safe-inference",
        "description": "Only assert with confidence > 0.7",
        "version": 1,
        "reliability": 0.95,
        "status": "active",
        "created_at": "2026-03-14T00:00:00Z",
        "last_updated": "2026-03-14T00:00:00Z",
        "steps": [
            {
                "step_id": 1,
                "action": "check_confidence",
                "tool": null,
                "conditions": ["confidence > 0.7"],
                "fallback": "reject"
            }
        ]
    });

    // Create policy
    Mock::given(method("POST"))
        .and(path("/api/mneme/policies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(policy_body.clone()))
        .mount(&mock_server)
        .await;

    // List policies
    Mock::given(method("GET"))
        .and(path("/api/mneme/policies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([policy_body.clone()])))
        .mount(&mock_server)
        .await;

    // Get history
    Mock::given(method("GET"))
        .and(path("/api/mneme/policies/p-safe-1/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([policy_body.clone()])))
        .mount(&mock_server)
        .await;

    // Update status to retired
    Mock::given(method("PATCH"))
        .and(path("/api/mneme/policies/p-safe-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "p-safe-1",
            "name": "safe-inference",
            "description": "Only assert with confidence > 0.7",
            "version": 1,
            "reliability": 0.95,
            "status": "retired",
            "created_at": "2026-03-14T00:00:00Z",
            "last_updated": "2026-03-14T12:00:00Z",
            "steps": []
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let step =
        json!({"step_id": 1, "action": "check_confidence", "conditions": ["confidence > 0.7"]});

    // Create
    let policy = client
        .policies()
        .create(
            "safe-inference",
            &[step],
            "Only assert with confidence > 0.7",
            None,
        )
        .await
        .unwrap();
    assert_eq!(policy.id, "p-safe-1");
    assert_eq!(policy.status, mnemebrain::PolicyStatus::Active);
    assert_eq!(policy.steps.len(), 1);

    // List
    let all = client.policies().list().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "safe-inference");

    // History
    let history = client.policies().get_history("p-safe-1").await.unwrap();
    assert_eq!(history.len(), 1);

    // Retire
    let retired = client
        .policies()
        .update_status("p-safe-1", PolicyStatus::Retired)
        .await
        .unwrap();
    assert_eq!(retired.status, mnemebrain::PolicyStatus::Retired);
}

// ── 9. Revision with policy ───────────────────────────────────────────────────

#[tokio::test]
async fn test_revision_with_policy() {
    let mock_server = MockServer::start().await;

    // Set a conservative revision policy
    Mock::given(method("POST"))
        .and(path("/api/mneme/revision/policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "policy_name": "conservative",
            "max_retraction_depth": 2,
            "max_retractions": 5
        })))
        .mount(&mock_server)
        .await;

    // Perform revision
    Mock::given(method("POST"))
        .and(path("/api/mneme/revision/revise"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "superseded_evidence_ids": ["e-old-1"],
            "retracted_belief_ids": [],
            "revision_depth": 1,
            "policy_name": "conservative",
            "bounded": true
        })))
        .mount(&mock_server)
        .await;

    // Check audit — one entry recorded
    Mock::given(method("GET"))
        .and(path("/api/mneme/revision/audit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "audit-1",
                "timestamp": "2026-03-14T00:00:00Z",
                "incoming_belief_id": "b-revised-1",
                "policy_name": "conservative",
                "revision_depth": 1,
                "bounded": true,
                "agent_id": "agent-tester"
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    // Configure policy
    let policy = client
        .revision()
        .set_policy("conservative", Some(2), Some(5))
        .await
        .unwrap();
    assert_eq!(policy.policy_name, "conservative");
    assert_eq!(policy.max_retraction_depth, 2);

    // Perform revision
    let conflicting = vec![mnemebrain::RevisionEvidenceItem {
        source_ref: "old-study".into(),
        content: "old conflicting claim".into(),
        polarity: Polarity::Supports,
        weight: 0.6,
        reliability: 0.7,
        id: Some("e-old-1".into()),
    }];
    let incoming = vec![mnemebrain::RevisionEvidenceItem {
        source_ref: "new-study".into(),
        content: "revised understanding".into(),
        polarity: Polarity::Supports,
        weight: 0.9,
        reliability: 0.85,
        id: None,
    }];
    let result = client
        .revision()
        .revise("b-revised-1", &conflicting, &incoming, "agent-tester")
        .await
        .unwrap();
    assert_eq!(result.superseded_evidence_ids, vec!["e-old-1"]);
    assert!(result.bounded);
    assert_eq!(result.revision_depth, 1);
    assert_eq!(result.policy_name, "conservative");

    // Verify audit trail
    let audit = client.revision().list_audit().await.unwrap();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].incoming_belief_id, "b-revised-1");
    assert_eq!(audit[0].policy_name, "conservative");
    assert!(audit[0].bounded);
}

// ── 10. Auth header propagation ───────────────────────────────────────────────

#[tokio::test]
async fn test_auth_header_propagation() {
    let mock_server = MockServer::start().await;

    // Health — require Authorization header with the expected value
    Mock::given(method("GET"))
        .and(path("/health"))
        .and(header("Authorization", "Bearer super-secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    // believe — also requires the auth header
    Mock::given(method("POST"))
        .and(path("/believe"))
        .and(header_exists("Authorization"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(belief_json("b-auth-1", "true", 0.9, false)),
        )
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::with_auth(
        &mock_server.uri(),
        "super-secret-key",
        Duration::from_secs(5),
    );

    // Health check should carry the auth header
    let health = client.health().await.unwrap();
    assert_eq!(health.status, "ok");

    // Subsequent calls should also carry the header
    let ev = EvidenceInput::new("src", "content");
    let belief = client
        .believe(&BelieveRequest::new("auth test claim", vec![ev]).with_source_agent("agent"))
        .await
        .unwrap();
    assert_eq!(belief.id, "b-auth-1");
}

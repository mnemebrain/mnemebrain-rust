use mnemebrain::{
    BelieveRequest, EvidenceInput, FrameOpenRequest, LiteFrameOpenRequest, MnemeBrainClient,
    SearchRequest, TruthState,
};
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_health() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let resp = client.health().await.unwrap();
    assert_eq!(resp.status, "ok");
}

#[tokio::test]
async fn test_believe() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/believe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "b-123",
            "truth_state": "true",
            "confidence": 0.85,
            "conflict": false,
            "was_separated": false,
            "memory_tier": "episodic",
            "evidence_ids": ["e-1"]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ev = EvidenceInput::new("test", "the sky is blue");
    let result = client
        .believe(&BelieveRequest::new("the sky is blue", vec![ev]))
        .await
        .unwrap();
    assert_eq!(result.id, "b-123");
    assert_eq!(result.truth_state, TruthState::True);
    assert!((result.confidence - 0.85).abs() < 0.001);
    assert!(!result.conflict);
}

#[tokio::test]
async fn test_explain_found() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/explain"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "claim": "sky is blue",
            "truth_state": "true",
            "confidence": 0.9,
            "supporting": [{
                "id": "e-1", "source_ref": "obs", "content": "looked up",
                "polarity": "supports", "weight": 0.7, "reliability": 0.8
            }],
            "attacking": [],
            "expired": []
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.explain("sky is blue").await.unwrap().unwrap();
    assert_eq!(result.claim, "sky is blue");
    assert_eq!(result.truth_state, TruthState::True);
    assert_eq!(result.supporting.len(), 1);
}

#[tokio::test]
async fn test_explain_not_found() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/explain"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.explain("unknown claim").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_search() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("query", "sky"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{
                "belief_id": "b-1",
                "claim": "sky is blue",
                "truth_state": "true",
                "confidence": 0.9,
                "similarity": 0.95,
                "rank_score": 0.92
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let resp = client.search(&SearchRequest::new("sky")).await.unwrap();
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].claim, "sky is blue");
    assert_eq!(resp.results[0].truth_state, TruthState::True);
}

#[tokio::test]
async fn test_retract_full_api() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/retract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "affected_beliefs": [{
                "id": "b-1",
                "truth_state": "neither",
                "confidence": 0.0,
                "conflict": false
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let results = client.retract("e-1").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].truth_state, TruthState::Neither);
}

#[tokio::test]
async fn test_retract_lite_api() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/retract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "b-1",
                "truth_state": "neither",
                "confidence": 0.5,
                "conflict": false
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let results = client.retract("e-1").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].truth_state, TruthState::Neither);
}

#[tokio::test]
async fn test_list_beliefs() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/beliefs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "beliefs": [{
                "id": "b-1",
                "claim": "test",
                "belief_type": "fact",
                "truth_state": "true",
                "confidence": 0.9,
                "tag_count": 0,
                "evidence_count": 1,
                "created_at": "2026-01-01T00:00:00Z",
                "last_revised": "2026-01-01T00:00:00Z"
            }],
            "total": 1,
            "offset": 0,
            "limit": 50
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let filters = mnemebrain::BeliefFilters::default();
    let resp = client.list_beliefs(&filters).await.unwrap();
    assert_eq!(resp.beliefs.len(), 1);
    assert_eq!(resp.total, 1);
}

#[tokio::test]
async fn test_frame_lifecycle() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/frame/open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "frame_id": "f-1",
            "beliefs_loaded": 0,
            "conflicts": 0,
            "snapshots": []
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/frame/f-1/context"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query": "test",
            "beliefs": [],
            "scratchpad": {},
            "conflicts": [],
            "step_count": 0
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/frame/f-1/commit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "frame_id": "f-1",
            "beliefs_created": 0,
            "beliefs_revised": 0
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/frame/f-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));

    let open = client
        .frame_open(&FrameOpenRequest::new("test"))
        .await
        .unwrap();
    assert_eq!(open.frame_id, "f-1");

    let ctx = client.frame_context("f-1").await.unwrap();
    assert_eq!(ctx.query, "test");

    let commit = client.frame_commit("f-1", &[], &[]).await.unwrap();
    assert_eq!(commit.frame_id, "f-1");

    client.frame_close("f-1").await.unwrap();
}

#[tokio::test]
async fn test_frame_open_lite() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/frame/open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "frame_id": "f-lite-1",
            "beliefs_loaded": 5,
            "conflicts": 1,
            "snapshots": []
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let req = LiteFrameOpenRequest::new("query-uuid-123");
    let open = client.frame_open_lite(&req).await.unwrap();
    assert_eq!(open.frame_id, "f-lite-1");
    assert_eq!(open.beliefs_loaded, 5);
}

#[tokio::test]
async fn test_http_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.health().await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, message } => {
            assert_eq!(status, 500);
            assert_eq!(message, "internal error");
        }
        _ => panic!("expected Http error"),
    }
}

#[tokio::test]
async fn test_with_auth() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client =
        MnemeBrainClient::with_auth(&mock_server.uri(), "test-api-key", Duration::from_secs(5));
    let resp = client.health().await.unwrap();
    assert_eq!(resp.status, "ok");
}

// ── revise ──

#[tokio::test]
async fn test_revise() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/revise"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "b-999",
            "truth_state": "false",
            "confidence": 0.3,
            "conflict": true,
            "was_separated": false,
            "memory_tier": "episodic"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ev = EvidenceInput::new("src-ref", "counter-evidence");
    let result = client.revise("b-999", &ev).await.unwrap();
    assert_eq!(result.id, "b-999");
    assert_eq!(result.truth_state, mnemebrain::TruthState::False);
    assert!(result.conflict);
}

// ── frame_add ──

#[tokio::test]
async fn test_frame_add() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/frame/f-1/add"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "belief_id": "b-2",
            "claim": "water is wet",
            "truth_state": "true",
            "confidence": 0.95,
            "belief_type": "fact",
            "evidence_count": 1,
            "conflict": false
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let snapshot = client.frame_add("f-1", "water is wet").await.unwrap();
    assert_eq!(snapshot.belief_id, "b-2");
    assert_eq!(snapshot.claim, "water is wet");
    assert_eq!(snapshot.truth_state, TruthState::True);
    assert!(!snapshot.conflict);
}

// ── frame_scratchpad ──

#[tokio::test]
async fn test_frame_scratchpad() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/frame/f-1/scratchpad"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client
        .frame_scratchpad("f-1", "draft", json!("some note"))
        .await
        .unwrap();
}

// ── frame_close error path ──

#[tokio::test]
async fn test_frame_close_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/frame/bad-id"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.frame_close("bad-id").await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, message } => {
            assert_eq!(status, 404);
            assert_eq!(message, "not found");
        }
        _ => panic!("expected Http error"),
    }
}

// ── reset ──

#[tokio::test]
async fn test_reset() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/reset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.reset().await.unwrap();
}

// ── set_time_offset ──

#[tokio::test]
async fn test_set_time_offset() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/time-offset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.set_time_offset(-7).await.unwrap();
}

// ── consolidate ──

#[tokio::test]
async fn test_consolidate() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/consolidate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "semantic_beliefs_created": 3,
            "episodics_pruned": 10,
            "clusters_found": 2
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.consolidate().await.unwrap();
    assert_eq!(result.semantic_beliefs_created, 3);
    assert_eq!(result.episodics_pruned, 10);
    assert_eq!(result.clusters_found, 2);
}

// ── get_memory_tier ──

#[tokio::test]
async fn test_get_memory_tier() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/memory-tier/b-42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "belief_id": "b-42",
            "memory_tier": "semantic",
            "consolidated_from_count": 5
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.get_memory_tier("b-42").await.unwrap();
    assert_eq!(result.belief_id, "b-42");
    assert_eq!(result.memory_tier, "semantic");
    assert_eq!(result.consolidated_from_count, 5);
}

// ── query_multihop ──

#[tokio::test]
async fn test_query_multihop() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/query_multihop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "belief_id": "b-1",
                    "claim": "A causes X",
                    "confidence": 0.8,
                    "truth_state": "true"
                },
                {
                    "belief_id": "b-2",
                    "claim": "B causes X",
                    "confidence": 0.6,
                    "truth_state": "true"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let resp = client.query_multihop("causes of X").await.unwrap();
    assert_eq!(resp.results.len(), 2);
    assert_eq!(resp.results[0].belief_id, "b-1");
    assert_eq!(resp.results[1].claim, "B causes X");
}

// ── benchmark_sandbox_fork ──

#[tokio::test]
async fn test_benchmark_sandbox_fork() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/benchmark/sandbox/fork"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandbox_id": "sb-1",
            "resolved_truth_state": "true",
            "canonical_unchanged": true
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.benchmark_sandbox_fork("scenario-A").await.unwrap();
    assert_eq!(result.sandbox_id, "sb-1");
    assert_eq!(result.resolved_truth_state, TruthState::True);
    assert!(result.canonical_unchanged);
}

// ── benchmark_sandbox_assume ──

#[tokio::test]
async fn test_benchmark_sandbox_assume() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/benchmark/sandbox/sb-2/assume"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandbox_id": "sb-2",
            "resolved_truth_state": "false",
            "canonical_unchanged": false
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .benchmark_sandbox_assume("sb-2", "b-5", "false")
        .await
        .unwrap();
    assert_eq!(result.sandbox_id, "sb-2");
    assert_eq!(result.resolved_truth_state, TruthState::False);
    assert!(!result.canonical_unchanged);
}

// ── benchmark_sandbox_resolve ──

#[tokio::test]
async fn test_benchmark_sandbox_resolve() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/benchmark/sandbox/sb-3/resolve/b-7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandbox_id": "sb-3",
            "resolved_truth_state": "neither",
            "canonical_unchanged": true
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .benchmark_sandbox_resolve("sb-3", "b-7")
        .await
        .unwrap();
    assert_eq!(result.sandbox_id, "sb-3");
    assert_eq!(result.resolved_truth_state, TruthState::Neither);
}

// ── benchmark_sandbox_discard ──

#[tokio::test]
async fn test_benchmark_sandbox_discard() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/benchmark/sandbox/sb-4/discard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.benchmark_sandbox_discard("sb-4").await.unwrap();
}

// ── benchmark_attack ──

#[tokio::test]
async fn test_benchmark_attack() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/benchmark/attack"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "edge_id": "edge-1",
            "attacker_id": "b-attacker",
            "target_id": "b-target"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .benchmark_attack("b-attacker", "b-target", "contradicts", 0.8)
        .await
        .unwrap();
    assert_eq!(result.edge_id, "edge-1");
    assert_eq!(result.attacker_id, "b-attacker");
    assert_eq!(result.target_id, "b-target");
}

// ── sub-client accessor methods ──

#[tokio::test]
async fn test_sandbox_accessor_returns_consistent_client() {
    let mock_server = MockServer::start().await;
    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    // Call twice — OnceLock must return the same instance
    let sc1 = client.sandbox() as *const _;
    let sc2 = client.sandbox() as *const _;
    assert_eq!(sc1, sc2);
}

#[tokio::test]
async fn test_goals_accessor_returns_consistent_client() {
    let mock_server = MockServer::start().await;
    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let gc1 = client.goals() as *const _;
    let gc2 = client.goals() as *const _;
    assert_eq!(gc1, gc2);
}

#[tokio::test]
async fn test_policies_accessor_returns_consistent_client() {
    let mock_server = MockServer::start().await;
    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let pc1 = client.policies() as *const _;
    let pc2 = client.policies() as *const _;
    assert_eq!(pc1, pc2);
}

#[tokio::test]
async fn test_revision_accessor_returns_consistent_client() {
    let mock_server = MockServer::start().await;
    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let rc1 = client.revision() as *const _;
    let rc2 = client.revision() as *const _;
    assert_eq!(rc1, rc2);
}

#[tokio::test]
async fn test_attacks_accessor_returns_consistent_client() {
    let mock_server = MockServer::start().await;
    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ac1 = client.attacks() as *const _;
    let ac2 = client.attacks() as *const _;
    assert_eq!(ac1, ac2);
}

#[tokio::test]
async fn test_reconsolidation_accessor_returns_consistent_client() {
    let mock_server = MockServer::start().await;
    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let rc1 = client.reconsolidation() as *const _;
    let rc2 = client.reconsolidation() as *const _;
    assert_eq!(rc1, rc2);
}

// ── MnemeBrainClient::default() ──

#[test]
fn test_client_default_constructor() {
    // Verify the convenience constructor builds without panicking.
    let _client = MnemeBrainClient::default();
}

// ── Builder user_agent ──

#[test]
fn test_client_builder_with_user_agent() {
    let _client = mnemebrain::MnemeBrainClientBuilder::new("http://localhost:8000")
        .user_agent("my-agent/1.0")
        .build();
    // Verify it builds without panicking; user_agent is internal to reqwest.
}

// ── list_beliefs with belief_type and tag filters ──

#[tokio::test]
async fn test_list_beliefs_with_belief_type_filter() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/beliefs"))
        .and(query_param("belief_type", "inference"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "beliefs": [],
            "total": 0,
            "offset": 0,
            "limit": 50
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let filters = mnemebrain::BeliefFilters::default()
        .with_belief_type(mnemebrain::BeliefType::Inference);
    let resp = client.list_beliefs(&filters).await.unwrap();
    assert_eq!(resp.total, 0);
}

#[tokio::test]
async fn test_list_beliefs_with_tag_filter() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/beliefs"))
        .and(query_param("tag", "science"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "beliefs": [],
            "total": 0,
            "offset": 0,
            "limit": 50
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let filters = mnemebrain::BeliefFilters::default()
        .with_tag("science");
    let resp = client.list_beliefs(&filters).await.unwrap();
    assert_eq!(resp.total, 0);
}

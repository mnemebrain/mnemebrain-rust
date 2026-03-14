use mnemebrain::{
    AttackType, BeliefType, CommitMode, EvidenceInput, GoalStatus, MnemeBrainClient, PolicyStatus,
    TruthState,
};
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ══════════════════════════════════════════════
// SandboxClient
// ══════════════════════════════════════════════

// ── fork (without frame_id) ──

#[tokio::test]
async fn test_sandbox_fork_no_frame() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/fork"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sb-1",
            "frame_id": null,
            "scenario_label": "what-if",
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z",
            "expires_at": "2026-01-02T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.sandbox().fork(None, "what-if", 86400).await.unwrap();
    assert_eq!(result.id, "sb-1");
    assert_eq!(result.scenario_label, "what-if");
    assert_eq!(result.status, "active");
}

// ── fork (with frame_id) ──

#[tokio::test]
async fn test_sandbox_fork_with_frame() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/fork"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sb-framed",
            "frame_id": "f-1",
            "scenario_label": "framed-scenario",
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z",
            "expires_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .sandbox()
        .fork(Some("f-1"), "framed-scenario", 3600)
        .await
        .unwrap();
    assert_eq!(result.id, "sb-framed");
    assert_eq!(result.frame_id, Some("f-1".into()));
    assert!(result.expires_at.is_none());
}

// ── quick (without frame_id) ──

#[tokio::test]
async fn test_sandbox_quick_no_frame() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/quick"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sb-quick",
            "frame_id": null,
            "scenario_label": "quick",
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z",
            "expires_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.sandbox().quick(None).await.unwrap();
    assert_eq!(result.id, "sb-quick");
}

// ── quick (with frame_id) ──

#[tokio::test]
async fn test_sandbox_quick_with_frame() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/quick"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sb-qf",
            "frame_id": "f-2",
            "scenario_label": "quick",
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z",
            "expires_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.sandbox().quick(Some("f-2")).await.unwrap();
    assert_eq!(result.frame_id, Some("f-2".into()));
}

// ── get_context ──

#[tokio::test]
async fn test_sandbox_get_context() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/sandbox/sb-1/context"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sb-1",
            "frame_id": null,
            "scenario_label": "ctx-test",
            "status": "active",
            "belief_overrides": {},
            "added_belief_ids": ["b-1"],
            "invalidated_evidence": ["e-3"],
            "created_at": "2026-01-01T00:00:00Z",
            "expires_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ctx = client.sandbox().get_context("sb-1").await.unwrap();
    assert_eq!(ctx.id, "sb-1");
    assert_eq!(ctx.added_belief_ids[0], "b-1");
    assert_eq!(ctx.invalidated_evidence[0], "e-3");
}

// ── assume ──

#[tokio::test]
async fn test_sandbox_assume() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/assume"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client
        .sandbox()
        .assume("sb-1", "b-5", TruthState::False)
        .await
        .unwrap();
}

// ── retract ──

#[tokio::test]
async fn test_sandbox_retract() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/retract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.sandbox().retract("sb-1", "e-7").await.unwrap();
}

// ── believe ──

#[tokio::test]
async fn test_sandbox_believe() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/believe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "b-new",
            "truth_state": "true",
            "confidence": 0.85,
            "conflict": false
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .sandbox()
        .believe("sb-1", "new belief claim", BeliefType::Fact)
        .await
        .unwrap();
    assert_eq!(result["id"], "b-new");
    assert_eq!(result["truth_state"], "true");
}

// ── revise ──

#[tokio::test]
async fn test_sandbox_revise() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/revise"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client
        .sandbox()
        .revise("sb-1", "b-3", &EvidenceInput::new("source", "new content"))
        .await
        .unwrap();
}

// ── attack ──

#[tokio::test]
async fn test_sandbox_attack() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/attack"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "edge_id": "e-atk-1",
            "type": "contradicts"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .sandbox()
        .attack(
            "sb-1",
            "b-attacker",
            "b-target",
            AttackType::Contradicts,
            0.9,
        )
        .await
        .unwrap();
    assert_eq!(result["edge_id"], "e-atk-1");
}

// ── diff ──

#[tokio::test]
async fn test_sandbox_diff() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/sandbox/sb-1/diff"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "belief_changes": [{
                "belief_id": "b-1",
                "field": "truth_state",
                "old_value": "true",
                "new_value": "false"
            }],
            "evidence_invalidations": [],
            "new_beliefs": [],
            "temporary_attacks": [],
            "goal_changes": [],
            "summary": "one override"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let diff = client.sandbox().diff("sb-1").await.unwrap();
    assert_eq!(diff.belief_changes.len(), 1);
    assert_eq!(diff.belief_changes[0].field, "truth_state");
    assert_eq!(diff.summary, "one override");
}

// ── commit (selective, with ids) ──

#[tokio::test]
async fn test_sandbox_commit_selective() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/commit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandbox_id": "sb-1",
            "committed_belief_ids": ["b-1"],
            "conflicts": []
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let ids = vec!["b-1".to_string()];
    let result = client
        .sandbox()
        .commit("sb-1", CommitMode::Selective, Some(&ids))
        .await
        .unwrap();
    assert_eq!(result.sandbox_id, "sb-1");
    assert_eq!(result.committed_belief_ids[0], "b-1");
}

// ── commit (all, without ids) ──

#[tokio::test]
async fn test_sandbox_commit_all() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-2/commit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandbox_id": "sb-2",
            "committed_belief_ids": ["b-1", "b-2"],
            "conflicts": ["b-3"]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .sandbox()
        .commit("sb-2", CommitMode::All, None)
        .await
        .unwrap();
    assert_eq!(result.committed_belief_ids.len(), 2);
    assert_eq!(result.conflicts[0], "b-3");
}

// ── discard ──

#[tokio::test]
async fn test_sandbox_discard() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/sb-1/discard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.sandbox().discard("sb-1").await.unwrap();
}

// ── explain ──

#[tokio::test]
async fn test_sandbox_explain() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/sandbox/sb-1/explain/b-5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "belief_id": "b-5",
            "sandbox_id": "sb-1",
            "resolved_truth_state": "true",
            "has_override": false,
            "override_fields": [],
            "invalidated_evidence_ids": [],
            "source": "canonical"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.sandbox().explain("sb-1", "b-5").await.unwrap();
    assert_eq!(result.belief_id, "b-5");
    assert_eq!(result.resolved_truth_state, TruthState::True);
    assert!(!result.has_override);
    assert_eq!(result.source, "canonical");
}

// ── evaluate_goal ──

#[tokio::test]
async fn test_sandbox_evaluate_goal() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/sandbox/sb-1/goals/g-1/evaluate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "goal_id": "g-1",
            "status": "active",
            "completion_fraction": 0.5,
            "blocking_belief_ids": ["b-bad"],
            "supporting_belief_ids": ["b-good"]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.sandbox().evaluate_goal("sb-1", "g-1").await.unwrap();
    assert_eq!(result.goal_id, "g-1");
    assert!((result.completion_fraction - 0.5).abs() < 0.001);
    assert_eq!(result.blocking_belief_ids[0], "b-bad");
}

// ── sandbox HTTP error propagation ──

#[tokio::test]
async fn test_sandbox_http_error_propagates() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/sandbox/fork"))
        .respond_with(ResponseTemplate::new(422).set_body_string("unprocessable"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.sandbox().fork(None, "bad", 100).await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, .. } => assert_eq!(status, 422),
        _ => panic!("expected Http error"),
    }
}

// ══════════════════════════════════════════════
// GoalClient
// ══════════════════════════════════════════════

// ── create (no optional fields) ──

#[tokio::test]
async fn test_goal_create_minimal() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/goals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "g-1",
            "goal": "achieve X",
            "owner": "agent-1",
            "priority": 0.8,
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .goals()
        .create("achieve X", "agent-1", 0.8, None, None)
        .await
        .unwrap();
    assert_eq!(result.id, "g-1");
    assert_eq!(result.goal, "achieve X");
    assert!(result.deadline.is_none());
}

// ── create (with success_criteria and deadline) ──

#[tokio::test]
async fn test_goal_create_with_criteria_and_deadline() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/goals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "g-2",
            "goal": "achieve Y",
            "owner": "agent-2",
            "priority": 0.5,
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z",
            "deadline": "2026-12-31T00:00:00Z",
            "success_criteria": {"metric": "accuracy"}
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let mut criteria = serde_json::Map::new();
    criteria.insert("metric".into(), json!("accuracy"));
    let result = client
        .goals()
        .create(
            "achieve Y",
            "agent-2",
            0.5,
            Some(&criteria),
            Some("2026-12-31T00:00:00Z"),
        )
        .await
        .unwrap();
    assert_eq!(result.id, "g-2");
    assert_eq!(result.deadline, Some("2026-12-31T00:00:00Z".into()));
}

// ── list ──

#[tokio::test]
async fn test_goal_list() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/goals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "g-1",
                "goal": "goal 1",
                "owner": "agent",
                "priority": 0.9,
                "status": "active",
                "created_at": "2026-01-01T00:00:00Z"
            },
            {
                "id": "g-2",
                "goal": "goal 2",
                "owner": "agent",
                "priority": 0.3,
                "status": "paused",
                "created_at": "2026-01-02T00:00:00Z"
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let goals = client.goals().list().await.unwrap();
    assert_eq!(goals.len(), 2);
    assert_eq!(goals[0].id, "g-1");
    assert_eq!(goals[1].id, "g-2");
}

// ── get ──

#[tokio::test]
async fn test_goal_get() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/goals/g-99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "g-99",
            "goal": "fetch me",
            "owner": "owner",
            "priority": 1.0,
            "status": "completed",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let goal = client.goals().get("g-99").await.unwrap();
    assert_eq!(goal.id, "g-99");
    assert_eq!(goal.status, mnemebrain::GoalStatus::Completed);
}

// ── evaluate ──

#[tokio::test]
async fn test_goal_evaluate() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/goals/g-1/evaluate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "goal_id": "g-1",
            "status": "active",
            "completion_fraction": 0.75,
            "blocking_belief_ids": [],
            "supporting_belief_ids": ["b-1"]
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let eval = client.goals().evaluate("g-1").await.unwrap();
    assert_eq!(eval.goal_id, "g-1");
    assert!((eval.completion_fraction - 0.75).abs() < 0.001);
}

// ── update_status ──

#[tokio::test]
async fn test_goal_update_status() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/mneme/goals/g-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "g-1",
            "goal": "test",
            "owner": "agent",
            "priority": 0.5,
            "status": "paused",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let goal = client
        .goals()
        .update_status("g-1", GoalStatus::Paused)
        .await
        .unwrap();
    assert_eq!(goal.status, mnemebrain::GoalStatus::Paused);
}

// ── abandon ──

#[tokio::test]
async fn test_goal_abandon() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/goals/g-1/abandon"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.goals().abandon("g-1").await.unwrap();
}

// ── goal HTTP error propagation ──

#[tokio::test]
async fn test_goal_http_error_propagates() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/goals/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.goals().get("missing").await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, .. } => assert_eq!(status, 404),
        _ => panic!("expected Http error"),
    }
}

// ══════════════════════════════════════════════
// PolicyClient
// ══════════════════════════════════════════════

fn policy_json(id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": "test-policy",
        "description": "a policy",
        "version": 1,
        "reliability": 0.9,
        "status": "active",
        "created_at": "2026-01-01T00:00:00Z",
        "last_updated": "2026-01-01T00:00:00Z",
        "steps": []
    })
}

// ── create (no applicability) ──

#[tokio::test]
async fn test_policy_create_minimal() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/policies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(policy_json("p-1")))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let steps = vec![json!({"step_id": 1, "action": "check"})];
    let result = client
        .policies()
        .create("test-policy", &steps, "a policy", None)
        .await
        .unwrap();
    assert_eq!(result.id, "p-1");
    assert_eq!(result.status, mnemebrain::PolicyStatus::Active);
}

// ── create (with applicability) ──

#[tokio::test]
async fn test_policy_create_with_applicability() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/policies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(policy_json("p-app")))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let mut app = serde_json::Map::new();
    app.insert("domain".into(), json!("science"));
    let result = client
        .policies()
        .create("test-policy", &[], "desc", Some(&app))
        .await
        .unwrap();
    assert_eq!(result.id, "p-app");
}

// ── list ──

#[tokio::test]
async fn test_policy_list() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/policies"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([policy_json("p-1"), policy_json("p-2")])),
        )
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let policies = client.policies().list().await.unwrap();
    assert_eq!(policies.len(), 2);
}

// ── get ──

#[tokio::test]
async fn test_policy_get() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/policies/p-42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(policy_json("p-42")))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let policy = client.policies().get("p-42").await.unwrap();
    assert_eq!(policy.id, "p-42");
}

// ── get_history ──

#[tokio::test]
async fn test_policy_get_history() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/policies/p-1/history"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([policy_json("p-1"), policy_json("p-1-old")])),
        )
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let history = client.policies().get_history("p-1").await.unwrap();
    assert_eq!(history.len(), 2);
}

// ── update_status ──

#[tokio::test]
async fn test_policy_update_status() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/mneme/policies/p-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "p-1",
            "name": "policy",
            "description": "d",
            "version": 2,
            "reliability": 0.8,
            "status": "retired",
            "created_at": "2026-01-01T00:00:00Z",
            "last_updated": "2026-02-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .policies()
        .update_status("p-1", PolicyStatus::Retired)
        .await
        .unwrap();
    assert_eq!(result.status, mnemebrain::PolicyStatus::Retired);
}

// ── policy HTTP error propagation ──

#[tokio::test]
async fn test_policy_http_error_propagates() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/policies"))
        .respond_with(ResponseTemplate::new(500).set_body_string("server error"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.policies().list().await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, .. } => assert_eq!(status, 500),
        _ => panic!("expected Http error"),
    }
}

// ══════════════════════════════════════════════
// RevisionClient
// ══════════════════════════════════════════════

// ── set_policy (all optional fields present) ──

#[tokio::test]
async fn test_revision_set_policy_full() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/revision/policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "policy_name": "conservative",
            "max_retraction_depth": 3,
            "max_retractions": 5
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .revision()
        .set_policy("conservative", Some(3), Some(5))
        .await
        .unwrap();
    assert_eq!(result.policy_name, "conservative");
    assert_eq!(result.max_retraction_depth, 3);
    assert_eq!(result.max_retractions, 5);
}

// ── set_policy (no optional fields) ──

#[tokio::test]
async fn test_revision_set_policy_minimal() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/revision/policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "policy_name": "permissive",
            "max_retraction_depth": 10,
            "max_retractions": 100
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .revision()
        .set_policy("permissive", None, None)
        .await
        .unwrap();
    assert_eq!(result.policy_name, "permissive");
}

// ── get_policy ──

#[tokio::test]
async fn test_revision_get_policy() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/revision/policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "policy_name": "standard",
            "max_retraction_depth": 5,
            "max_retractions": 50
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.revision().get_policy().await.unwrap();
    assert_eq!(result.policy_name, "standard");
    assert_eq!(result.max_retraction_depth, 5);
}

// ── list_audit ──

#[tokio::test]
async fn test_revision_list_audit() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/revision/audit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "audit-1",
                "timestamp": "2026-01-01T00:00:00Z",
                "incoming_belief_id": "b-1",
                "policy_name": "standard",
                "revision_depth": 1,
                "bounded": false,
                "agent_id": "agent-x"
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let audit = client.revision().list_audit().await.unwrap();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].id, "audit-1");
    assert_eq!(audit[0].agent_id, "agent-x");
}

// ── revise ──

#[tokio::test]
async fn test_revision_revise() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/revision/revise"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "superseded_evidence_ids": ["e-old"],
            "retracted_belief_ids": [],
            "revision_depth": 1,
            "policy_name": "standard",
            "bounded": false
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let conflicting = vec![mnemebrain::RevisionEvidenceItem {
        source_ref: "doc-old".into(),
        content: "old claim".into(),
        polarity: mnemebrain::Polarity::Attacks,
        weight: 0.6,
        reliability: 0.7,
        id: Some("e-old".into()),
    }];
    let incoming = vec![mnemebrain::RevisionEvidenceItem {
        source_ref: "doc-new".into(),
        content: "new claim".into(),
        polarity: mnemebrain::Polarity::Supports,
        weight: 0.9,
        reliability: 0.95,
        id: None,
    }];
    let result = client
        .revision()
        .revise("b-incoming", &conflicting, &incoming, "agent-1")
        .await
        .unwrap();
    assert_eq!(result.superseded_evidence_ids[0], "e-old");
    assert_eq!(result.revision_depth, 1);
    assert!(!result.bounded);
}

// ── revision HTTP error propagation ──

#[tokio::test]
async fn test_revision_http_error_propagates() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/revision/policy"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.revision().get_policy().await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, message } => {
            assert_eq!(status, 403);
            assert_eq!(message, "forbidden");
        }
        _ => panic!("expected Http error"),
    }
}

// ══════════════════════════════════════════════
// AttackClient
// ══════════════════════════════════════════════

fn attack_edge_json(id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "source_belief_id": "b-1",
        "target_belief_id": "b-2",
        "attack_type": "undermines",
        "weight": 0.7,
        "active": true,
        "created_at": "2026-01-01T00:00:00Z"
    })
}

// ── create ──

#[tokio::test]
async fn test_attack_create() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/attacks/b-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(attack_edge_json("atk-1")))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client
        .attacks()
        .create("b-1", "b-2", AttackType::Undermines, 0.7)
        .await
        .unwrap();
    assert_eq!(result.id, "atk-1");
    assert_eq!(result.attack_type, mnemebrain::AttackType::Undermines);
    assert!(result.active);
}

// ── list ──

#[tokio::test]
async fn test_attack_list() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/attacks/b-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            attack_edge_json("atk-1"),
            attack_edge_json("atk-2")
        ])))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let attacks = client.attacks().list("b-1").await.unwrap();
    assert_eq!(attacks.len(), 2);
}

// ── get_chain ──

#[tokio::test]
async fn test_attack_get_chain() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/attacks/b-1/chain"))
        .and(query_param("max_depth", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            [attack_edge_json("atk-1")],
            [attack_edge_json("atk-2"), attack_edge_json("atk-3")]
        ])))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let chain = client.attacks().get_chain("b-1", 3).await.unwrap();
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].len(), 1);
    assert_eq!(chain[1].len(), 2);
    assert_eq!(chain[0][0].id, "atk-1");
}

// ── deactivate ──

#[tokio::test]
async fn test_attack_deactivate() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/attacks/atk-1/deactivate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    client.attacks().deactivate("atk-1").await.unwrap();
}

// ── attack HTTP error propagation ──

#[tokio::test]
async fn test_attack_http_error_propagates() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/attacks/b-missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.attacks().list("b-missing").await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, .. } => assert_eq!(status, 404),
        _ => panic!("expected Http error"),
    }
}

// ══════════════════════════════════════════════
// ReconsolidationClient
// ══════════════════════════════════════════════

// ── queue ──

#[tokio::test]
async fn test_recon_queue() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/mneme/reconsolidation/queue"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queue_size": 17
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.reconsolidation().queue().await.unwrap();
    assert_eq!(result.queue_size, 17);
}

// ── run ──

#[tokio::test]
async fn test_recon_run() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/reconsolidation/run"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "processed": 5,
            "timestamp": "2026-01-01T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let result = client.reconsolidation().run().await.unwrap();
    assert_eq!(result.processed, 5);
    assert_eq!(result.timestamp, "2026-01-01T12:00:00Z");
}

// ── recon HTTP error propagation ──

#[tokio::test]
async fn test_recon_http_error_propagates() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mneme/reconsolidation/run"))
        .respond_with(ResponseTemplate::new(503).set_body_string("service unavailable"))
        .mount(&mock_server)
        .await;

    let client = MnemeBrainClient::new(&mock_server.uri(), Duration::from_secs(5));
    let err = client.reconsolidation().run().await.unwrap_err();
    match err {
        mnemebrain::MnemeBrainError::Http { status, message } => {
            assert_eq!(status, 503);
            assert_eq!(message, "service unavailable");
        }
        _ => panic!("expected Http error"),
    }
}

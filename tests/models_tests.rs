use mnemebrain::*;
use serde_json::json;

#[test]
fn test_truth_state_serde() {
    let ts = TruthState::True;
    let s = serde_json::to_string(&ts).unwrap();
    assert_eq!(s, "\"true\"");

    let parsed: TruthState = serde_json::from_str("\"both\"").unwrap();
    assert_eq!(parsed, TruthState::Both);
}

#[test]
fn test_belief_type_serde() {
    let bt = BeliefType::Inference;
    let s = serde_json::to_string(&bt).unwrap();
    assert_eq!(s, "\"inference\"");
}

#[test]
fn test_polarity_serde() {
    let p = Polarity::Supports;
    let s = serde_json::to_string(&p).unwrap();
    assert_eq!(s, "\"supports\"");

    let parsed: Polarity = serde_json::from_str("\"attacks\"").unwrap();
    assert_eq!(parsed, Polarity::Attacks);
}

#[test]
fn test_polarity_default() {
    let p = Polarity::default();
    assert_eq!(p, Polarity::Supports);
}

#[test]
fn test_commit_mode_serde() {
    let cm = CommitMode::DiscardConflicts;
    let s = serde_json::to_string(&cm).unwrap();
    assert_eq!(s, "\"discard_conflicts\"");

    let parsed: CommitMode = serde_json::from_str("\"all\"").unwrap();
    assert_eq!(parsed, CommitMode::All);
}

#[test]
fn test_policy_status_serde() {
    let ps = PolicyStatus::FlaggedForRevision;
    let s = serde_json::to_string(&ps).unwrap();
    assert_eq!(s, "\"flagged_for_revision\"");
}

#[test]
fn test_conflict_policy_serde() {
    let cp = ConflictPolicy::Conservative;
    let s = serde_json::to_string(&cp).unwrap();
    assert_eq!(s, "\"conservative\"");

    let parsed: ConflictPolicy = serde_json::from_str("\"surface\"").unwrap();
    assert_eq!(parsed, ConflictPolicy::Surface);
}

#[test]
fn test_evidence_input_new() {
    let ev = EvidenceInput::new("src", "content");
    assert_eq!(ev.source_ref, "src");
    assert_eq!(ev.polarity, Polarity::Supports);
    assert!((ev.weight - 0.7).abs() < 0.001);
    assert!((ev.reliability - 0.8).abs() < 0.001);
    assert!(ev.scope.is_none());
}

#[test]
fn test_evidence_input_builder() {
    let ev = EvidenceInput::new("src", "content")
        .with_polarity(Polarity::Attacks)
        .with_weight(0.9)
        .with_reliability(0.95)
        .with_scope("local");
    assert_eq!(ev.polarity, Polarity::Attacks);
    assert!((ev.weight - 0.9).abs() < 0.001);
    assert!((ev.reliability - 0.95).abs() < 0.001);
    assert_eq!(ev.scope, Some("local".into()));
}

#[test]
fn test_evidence_input_serde() {
    let ev = EvidenceInput::new("test", "hello");
    let json = serde_json::to_value(&ev).unwrap();
    assert_eq!(json["source_ref"], "test");
    assert_eq!(json["polarity"], "supports");
    assert!(json.get("scope").is_none()); // skip_serializing_if None

    let back: EvidenceInput = serde_json::from_value(json).unwrap();
    assert_eq!(back.source_ref, "test");
    assert_eq!(back.polarity, Polarity::Supports);
}

#[test]
fn test_belief_result_deserialize() {
    let v = json!({
        "id": "b-1",
        "truth_state": "true",
        "confidence": 0.9,
        "conflict": false
    });
    let br: BeliefResult = serde_json::from_value(v).unwrap();
    assert_eq!(br.id, "b-1");
    assert_eq!(br.truth_state, TruthState::True);
    assert!(!br.was_separated); // default
    assert_eq!(br.memory_tier, "episodic"); // default
}

#[test]
fn test_search_result_deserialize() {
    let v = json!({
        "belief_id": "b-1",
        "claim": "sky is blue",
        "truth_state": "true",
        "confidence": 0.9,
        "similarity": 0.95,
        "rank_score": 0.92
    });
    let sr: SearchResult = serde_json::from_value(v).unwrap();
    assert_eq!(sr.belief_id, "b-1");
    assert_eq!(sr.truth_state, TruthState::True);
    assert!((sr.rank_score - 0.92).abs() < 0.001);
}

#[test]
fn test_belief_filters_default() {
    let f = BeliefFilters::default();
    assert!(f.truth_state.is_none());
    assert_eq!(f.limit, 50);
    assert!((f.max_confidence - 1.0).abs() < 0.001);
    assert!((f.min_confidence - 0.0).abs() < 0.001);
    assert_eq!(f.offset, 0);
}

#[test]
fn test_goal_result_deserialize() {
    let v = json!({
        "id": "g-1",
        "goal": "test goal",
        "owner": "agent-1",
        "priority": 0.5,
        "status": "active",
        "created_at": "2026-01-01T00:00:00Z"
    });
    let gr: GoalResult = serde_json::from_value(v).unwrap();
    assert_eq!(gr.id, "g-1");
    assert_eq!(gr.status, GoalStatus::Active);
    assert!(gr.deadline.is_none());
}

#[test]
fn test_policy_result_deserialize() {
    let v = json!({
        "id": "p-1",
        "name": "default",
        "description": "test",
        "version": 1,
        "reliability": 0.9,
        "status": "active",
        "created_at": "2026-01-01T00:00:00Z",
        "last_updated": "2026-01-01T00:00:00Z",
        "steps": [{
            "step_id": 1,
            "action": "check",
            "tool": null,
            "conditions": [],
            "fallback": null
        }]
    });
    let pr: PolicyResult = serde_json::from_value(v).unwrap();
    assert_eq!(pr.steps.len(), 1);
    assert_eq!(pr.steps[0].action, "check");
    assert_eq!(pr.status, PolicyStatus::Active);
}

#[test]
fn test_attack_edge_deserialize() {
    let v = json!({
        "id": "a-1",
        "source_belief_id": "b-1",
        "target_belief_id": "b-2",
        "attack_type": "contradicts",
        "weight": 0.5,
        "active": true,
        "created_at": "2026-01-01T00:00:00Z"
    });
    let ae: AttackEdgeResult = serde_json::from_value(v).unwrap();
    assert_eq!(ae.id, "a-1");
    assert_eq!(ae.attack_type, AttackType::Contradicts);
    assert!(ae.active);
}

#[test]
fn test_lite_frame_open_request() {
    let req = LiteFrameOpenRequest::new("abc-123");
    assert_eq!(req.query_id, "abc-123");
    assert!(req.goal_id.is_none());
    assert_eq!(req.top_k, 20);
    assert_eq!(req.ttl_seconds, 300);

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["query_id"], "abc-123");
    assert!(json.get("goal_id").is_none()); // skip None
}

#[test]
fn test_belief_list_item_typed() {
    let v = json!({
        "id": "b-1",
        "claim": "test",
        "belief_type": "fact",
        "truth_state": "both",
        "confidence": 0.55,
        "tag_count": 1,
        "evidence_count": 2,
        "created_at": "2026-01-01T00:00:00Z",
        "last_revised": "2026-01-01T00:00:00Z"
    });
    let item: BeliefListItem = serde_json::from_value(v).unwrap();
    assert_eq!(item.belief_type, BeliefType::Fact);
    assert_eq!(item.truth_state, TruthState::Both);
}

// ── Additional enum serde coverage ──

#[test]
fn test_sandbox_status_serde() {
    let cases: &[(SandboxStatus, &str)] = &[
        (SandboxStatus::Active, "\"active\""),
        (SandboxStatus::Committed, "\"committed\""),
        (SandboxStatus::Discarded, "\"discarded\""),
        (SandboxStatus::Expired, "\"expired\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(variant).unwrap();
        assert_eq!(&s, expected);
        let back: SandboxStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, variant);
    }
}

#[test]
fn test_attack_type_serde() {
    let cases: &[(AttackType, &str)] = &[
        (AttackType::Contradicts, "\"contradicts\""),
        (AttackType::Undermines, "\"undermines\""),
        (AttackType::Rebuts, "\"rebuts\""),
        (AttackType::Undercuts, "\"undercuts\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(variant).unwrap();
        assert_eq!(&s, expected);
        let back: AttackType = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, variant);
    }
}

#[test]
fn test_goal_status_serde() {
    let cases: &[(GoalStatus, &str)] = &[
        (GoalStatus::Active, "\"active\""),
        (GoalStatus::Paused, "\"paused\""),
        (GoalStatus::Completed, "\"completed\""),
        (GoalStatus::Failed, "\"failed\""),
        (GoalStatus::Abandoned, "\"abandoned\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(variant).unwrap();
        assert_eq!(&s, expected);
        let back: GoalStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, variant);
    }
}

#[test]
fn test_conflict_policy_optimistic() {
    let cp = ConflictPolicy::Optimistic;
    let s = serde_json::to_string(&cp).unwrap();
    assert_eq!(s, "\"optimistic\"");
    let back: ConflictPolicy = serde_json::from_str("\"optimistic\"").unwrap();
    assert_eq!(back, ConflictPolicy::Optimistic);
}

#[test]
fn test_truth_state_all_variants() {
    let cases: &[(TruthState, &str)] = &[
        (TruthState::True, "\"true\""),
        (TruthState::False, "\"false\""),
        (TruthState::Both, "\"both\""),
        (TruthState::Neither, "\"neither\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(variant).unwrap();
        assert_eq!(&s, expected);
        let back: TruthState = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, variant);
    }
}

#[test]
fn test_belief_type_all_variants() {
    let cases: &[(BeliefType, &str)] = &[
        (BeliefType::Fact, "\"fact\""),
        (BeliefType::Preference, "\"preference\""),
        (BeliefType::Inference, "\"inference\""),
        (BeliefType::Prediction, "\"prediction\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(variant).unwrap();
        assert_eq!(&s, expected);
        let back: BeliefType = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, variant);
    }
}

#[test]
fn test_commit_mode_selective() {
    let cm = CommitMode::Selective;
    let s = serde_json::to_string(&cm).unwrap();
    assert_eq!(s, "\"selective\"");
    let back: CommitMode = serde_json::from_str("\"selective\"").unwrap();
    assert_eq!(back, CommitMode::Selective);
}

#[test]
fn test_policy_status_all_variants() {
    let cases: &[(PolicyStatus, &str)] = &[
        (PolicyStatus::Active, "\"active\""),
        (PolicyStatus::FlaggedForRevision, "\"flagged_for_revision\""),
        (PolicyStatus::Superseded, "\"superseded\""),
        (PolicyStatus::Retired, "\"retired\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(variant).unwrap();
        assert_eq!(&s, expected);
        let back: PolicyStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, variant);
    }
}

// ── FrameOpenRequest serialization ──

#[test]
fn test_frame_open_request_serialization() {
    let req = FrameOpenRequest {
        query: "test query".into(),
        preload_claims: vec!["claim-1".into()],
        ttl_seconds: 600,
        source_agent: "agent-x".into(),
    };
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["query"], "test query");
    assert_eq!(v["ttl_seconds"], 600);
    assert_eq!(v["preload_claims"][0], "claim-1");
    assert_eq!(v["source_agent"], "agent-x");
}

#[test]
fn test_frame_open_request_skip_empty_vec() {
    let req = FrameOpenRequest {
        query: "q".into(),
        preload_claims: vec![],
        ttl_seconds: 60,
        source_agent: "".into(),
    };
    let v = serde_json::to_value(&req).unwrap();
    // preload_claims is empty — skipped by skip_serializing_if = Vec::is_empty
    assert!(v.get("preload_claims").is_none());
    // source_agent is "" — skipped by skip_serializing_if = str::is_empty
    assert!(v.get("source_agent").is_none());
}

// ── Sandbox response types ──

#[test]
fn test_sandbox_result_deserialize() {
    let v = json!({
        "id": "sb-1",
        "frame_id": null,
        "scenario_label": "what-if",
        "status": "active",
        "created_at": "2026-01-01T00:00:00Z",
        "expires_at": "2026-01-02T00:00:00Z"
    });
    let sr: SandboxResult = serde_json::from_value(v).unwrap();
    assert_eq!(sr.id, "sb-1");
    assert_eq!(sr.scenario_label, "what-if");
    assert_eq!(sr.status, "active");
    assert!(sr.frame_id.is_none());
    assert!(sr.expires_at.is_some());
}

#[test]
fn test_sandbox_context_result_deserialize() {
    let v = json!({
        "id": "sb-1",
        "frame_id": "f-1",
        "scenario_label": "scenario",
        "status": "active",
        "belief_overrides": {"b-1": "true"},
        "added_belief_ids": ["b-2"],
        "invalidated_evidence": ["e-9"],
        "created_at": "2026-01-01T00:00:00Z",
        "expires_at": null
    });
    let scr: SandboxContextResult = serde_json::from_value(v).unwrap();
    assert_eq!(scr.id, "sb-1");
    assert_eq!(scr.frame_id, Some("f-1".into()));
    assert_eq!(scr.added_belief_ids.len(), 1);
    assert_eq!(scr.invalidated_evidence[0], "e-9");
}

#[test]
fn test_sandbox_diff_result_deserialize() {
    let v = json!({
        "belief_changes": [{
            "belief_id": "b-1",
            "field": "truth_state",
            "old_value": "true",
            "new_value": "false"
        }],
        "evidence_invalidations": ["e-1"],
        "new_beliefs": ["b-new"],
        "temporary_attacks": [],
        "goal_changes": [],
        "summary": "one change"
    });
    let diff: SandboxDiffResult = serde_json::from_value(v).unwrap();
    assert_eq!(diff.belief_changes.len(), 1);
    assert_eq!(diff.belief_changes[0].belief_id, "b-1");
    assert_eq!(diff.belief_changes[0].field, "truth_state");
    assert_eq!(diff.evidence_invalidations[0], "e-1");
    assert_eq!(diff.new_beliefs[0], "b-new");
    assert_eq!(diff.summary, "one change");
}

#[test]
fn test_sandbox_commit_result_deserialize() {
    let v = json!({
        "sandbox_id": "sb-1",
        "committed_belief_ids": ["b-1", "b-2"],
        "conflicts": []
    });
    let scr: SandboxCommitResult = serde_json::from_value(v).unwrap();
    assert_eq!(scr.sandbox_id, "sb-1");
    assert_eq!(scr.committed_belief_ids.len(), 2);
    assert!(scr.conflicts.is_empty());
}

#[test]
fn test_sandbox_explain_result_deserialize() {
    let v = json!({
        "belief_id": "b-1",
        "sandbox_id": "sb-1",
        "resolved_truth_state": "both",
        "has_override": true,
        "override_fields": ["truth_state"],
        "invalidated_evidence_ids": ["e-3"],
        "source": "sandbox"
    });
    let ser: SandboxExplainResult = serde_json::from_value(v).unwrap();
    assert_eq!(ser.belief_id, "b-1");
    assert_eq!(ser.resolved_truth_state, TruthState::Both);
    assert!(ser.has_override);
    assert_eq!(ser.override_fields[0], "truth_state");
    assert_eq!(ser.source, "sandbox");
}

// ── Revision types ──

#[test]
fn test_revision_policy_result_deserialize() {
    let v = json!({
        "policy_name": "conservative",
        "max_retraction_depth": 3,
        "max_retractions": 10
    });
    let rpr: RevisionPolicyResult = serde_json::from_value(v).unwrap();
    assert_eq!(rpr.policy_name, "conservative");
    assert_eq!(rpr.max_retraction_depth, 3);
    assert_eq!(rpr.max_retractions, 10);
}

#[test]
fn test_revision_audit_entry_deserialize() {
    let v = json!({
        "id": "audit-1",
        "timestamp": "2026-01-01T00:00:00Z",
        "incoming_belief_id": "b-1",
        "policy_name": "aggressive",
        "revision_depth": 2,
        "bounded": false,
        "agent_id": "agent-1"
    });
    let rae: RevisionAuditEntry = serde_json::from_value(v).unwrap();
    assert_eq!(rae.id, "audit-1");
    assert_eq!(rae.policy_name, "aggressive");
    assert_eq!(rae.revision_depth, 2);
    assert!(!rae.bounded);
}

#[test]
fn test_revision_evidence_item_defaults() {
    // Deserialize with no fields set — all use defaults
    let v = json!({});
    let rei: RevisionEvidenceItem = serde_json::from_value(v).unwrap();
    assert_eq!(rei.source_ref, "");
    assert_eq!(rei.content, "");
    assert_eq!(rei.polarity, Polarity::Supports);
    assert!((rei.weight - 0.8).abs() < 0.001);
    assert!((rei.reliability - 0.7).abs() < 0.001);
    assert!(rei.id.is_none());
}

#[test]
fn test_revision_evidence_item_with_id() {
    let v = json!({
        "source_ref": "doc-1",
        "content": "some evidence",
        "polarity": "attacks",
        "weight": 0.5,
        "reliability": 0.6,
        "id": "ev-99"
    });
    let rei: RevisionEvidenceItem = serde_json::from_value(v).unwrap();
    assert_eq!(rei.source_ref, "doc-1");
    assert_eq!(rei.polarity, Polarity::Attacks);
    assert_eq!(rei.id, Some("ev-99".into()));
}

#[test]
fn test_revision_evidence_item_skip_none_id_on_serialize() {
    let rei = RevisionEvidenceItem {
        source_ref: "s".into(),
        content: "c".into(),
        polarity: Polarity::Supports,
        weight: 0.8,
        reliability: 0.7,
        id: None,
    };
    let v = serde_json::to_value(&rei).unwrap();
    assert!(v.get("id").is_none());
}

#[test]
fn test_revision_result_deserialize() {
    let v = json!({
        "superseded_evidence_ids": ["e-1"],
        "retracted_belief_ids": ["b-old"],
        "revision_depth": 1,
        "policy_name": "standard",
        "bounded": true
    });
    let rr: RevisionResult = serde_json::from_value(v).unwrap();
    assert_eq!(rr.superseded_evidence_ids[0], "e-1");
    assert_eq!(rr.retracted_belief_ids[0], "b-old");
    assert_eq!(rr.revision_depth, 1);
    assert_eq!(rr.policy_name, "standard");
    assert!(rr.bounded);
}

#[test]
fn test_revision_result_defaults() {
    let v = json!({});
    let rr: RevisionResult = serde_json::from_value(v).unwrap();
    assert!(rr.superseded_evidence_ids.is_empty());
    assert!(rr.retracted_belief_ids.is_empty());
    assert_eq!(rr.revision_depth, 0);
    assert_eq!(rr.policy_name, "");
    assert!(!rr.bounded);
}

// ── Reconsolidation types ──

#[test]
fn test_reconsolidation_queue_result_deserialize() {
    let v = json!({ "queue_size": 42 });
    let rqr: ReconsolidationQueueResult = serde_json::from_value(v).unwrap();
    assert_eq!(rqr.queue_size, 42);
}

#[test]
fn test_reconsolidation_run_result_deserialize() {
    let v = json!({
        "processed": 7,
        "timestamp": "2026-01-01T12:00:00Z"
    });
    let rrr: ReconsolidationRunResult = serde_json::from_value(v).unwrap();
    assert_eq!(rrr.processed, 7);
    assert_eq!(rrr.timestamp, "2026-01-01T12:00:00Z");
}

// ── Consolidation & MemoryTier types ──

#[test]
fn test_consolidate_result_deserialize() {
    let v = json!({
        "semantic_beliefs_created": 5,
        "episodics_pruned": 20,
        "clusters_found": 3
    });
    let cr: ConsolidateResult = serde_json::from_value(v).unwrap();
    assert_eq!(cr.semantic_beliefs_created, 5);
    assert_eq!(cr.episodics_pruned, 20);
    assert_eq!(cr.clusters_found, 3);
}

#[test]
fn test_memory_tier_result_deserialize() {
    let v = json!({
        "belief_id": "b-7",
        "memory_tier": "semantic",
        "consolidated_from_count": 4
    });
    let mtr: MemoryTierResult = serde_json::from_value(v).unwrap();
    assert_eq!(mtr.belief_id, "b-7");
    assert_eq!(mtr.memory_tier, "semantic");
    assert_eq!(mtr.consolidated_from_count, 4);
}

// ── Multihop types ──

#[test]
fn test_multihop_result_item_deserialize() {
    let v = json!({
        "belief_id": "b-hop",
        "claim": "transitive relation",
        "confidence": 0.7,
        "truth_state": "true"
    });
    let item: MultihopResultItem = serde_json::from_value(v).unwrap();
    assert_eq!(item.belief_id, "b-hop");
    assert_eq!(item.truth_state, TruthState::True);
    assert!((item.confidence - 0.7).abs() < 0.001);
}

#[test]
fn test_multihop_response_empty_default() {
    let v = json!({});
    let resp: MultihopResponse = serde_json::from_value(v).unwrap();
    assert!(resp.results.is_empty());
}

#[test]
fn test_multihop_response_with_results() {
    let v = json!({
        "results": [
            {"belief_id": "b-1", "claim": "A", "confidence": 0.9, "truth_state": "true"},
            {"belief_id": "b-2", "claim": "B", "confidence": 0.5, "truth_state": "neither"}
        ]
    });
    let resp: MultihopResponse = serde_json::from_value(v).unwrap();
    assert_eq!(resp.results.len(), 2);
    assert_eq!(resp.results[1].truth_state, TruthState::Neither);
}

// ── Benchmark types ──

#[test]
fn test_benchmark_sandbox_result_deserialize() {
    let v = json!({
        "sandbox_id": "sb-bench",
        "resolved_truth_state": "both",
        "canonical_unchanged": false
    });
    let bsr: BenchmarkSandboxResult = serde_json::from_value(v).unwrap();
    assert_eq!(bsr.sandbox_id, "sb-bench");
    assert_eq!(bsr.resolved_truth_state, TruthState::Both);
    assert!(!bsr.canonical_unchanged);
}

#[test]
fn test_benchmark_attack_result_deserialize() {
    let v = json!({
        "edge_id": "edge-99",
        "attacker_id": "b-atk",
        "target_id": "b-tgt"
    });
    let bar: BenchmarkAttackResult = serde_json::from_value(v).unwrap();
    assert_eq!(bar.edge_id, "edge-99");
    assert_eq!(bar.attacker_id, "b-atk");
    assert_eq!(bar.target_id, "b-tgt");
}

// ── FrameContextResult alias field ──

#[test]
fn test_frame_context_result_active_query_alias() {
    // Backend may return "active_query" instead of "query"
    let v = json!({
        "active_query": "aliased query",
        "beliefs": [],
        "scratchpad": {},
        "conflicts": [],
        "step_count": 3
    });
    let fcr: FrameContextResult = serde_json::from_value(v).unwrap();
    assert_eq!(fcr.query, "aliased query");
    assert_eq!(fcr.step_count, 3);
}

#[test]
fn test_frame_context_result_query_field() {
    // Standard "query" field works normally
    let v = json!({
        "query": "normal query",
        "beliefs": [],
        "scratchpad": {},
        "conflicts": [],
        "step_count": 0
    });
    let fcr: FrameContextResult = serde_json::from_value(v).unwrap();
    assert_eq!(fcr.query, "normal query");
}

// ── Additional struct coverage ──

#[test]
fn test_belief_snapshot_deserialize() {
    let v = json!({
        "belief_id": "b-snap",
        "claim": "snapshot claim",
        "truth_state": "true",
        "confidence": 0.88,
        "belief_type": "inference",
        "evidence_count": 2,
        "conflict": true
    });
    let bs: BeliefSnapshot = serde_json::from_value(v).unwrap();
    assert_eq!(bs.belief_id, "b-snap");
    assert_eq!(bs.belief_type, BeliefType::Inference);
    assert!(bs.conflict);
}

#[test]
fn test_frame_open_result_with_snapshots() {
    let v = json!({
        "frame_id": "f-2",
        "beliefs_loaded": 2,
        "conflicts": 1,
        "snapshots": [{
            "belief_id": "b-1",
            "claim": "c1",
            "truth_state": "true",
            "confidence": 0.8,
            "belief_type": "fact",
            "evidence_count": 1,
            "conflict": false
        }]
    });
    let for_: FrameOpenResult = serde_json::from_value(v).unwrap();
    assert_eq!(for_.frame_id, "f-2");
    assert_eq!(for_.snapshots.len(), 1);
    assert_eq!(for_.snapshots[0].claim, "c1");
}

#[test]
fn test_frame_commit_result_deserialize() {
    let v = json!({
        "frame_id": "f-1",
        "beliefs_created": 2,
        "beliefs_revised": 1
    });
    let fcr: FrameCommitResult = serde_json::from_value(v).unwrap();
    assert_eq!(fcr.frame_id, "f-1");
    assert_eq!(fcr.beliefs_created, 2);
    assert_eq!(fcr.beliefs_revised, 1);
}

#[test]
fn test_health_response_extra_fields() {
    let v = json!({
        "status": "ok",
        "version": "0.5.0",
        "uptime_seconds": 3600
    });
    let hr: HealthResponse = serde_json::from_value(v).unwrap();
    assert_eq!(hr.status, "ok");
    assert_eq!(hr.extra["version"], "0.5.0");
    assert_eq!(hr.extra["uptime_seconds"], 3600);
}

#[test]
fn test_retrieved_belief_deserialize() {
    let v = json!({
        "claim": "Earth orbits the Sun",
        "confidence": 0.99,
        "similarity": 0.87
    });
    let rb: RetrievedBelief = serde_json::from_value(v).unwrap();
    assert_eq!(rb.claim, "Earth orbits the Sun");
    assert!((rb.similarity - 0.87).abs() < 0.001);
}

#[test]
fn test_ask_result_deserialize() {
    let v = json!({
        "query_id": "q-abc",
        "retrieved_beliefs": [{
            "claim": "test",
            "confidence": 0.5,
            "similarity": 0.6
        }]
    });
    let ar: AskResult = serde_json::from_value(v).unwrap();
    assert_eq!(ar.query_id, "q-abc");
    assert_eq!(ar.retrieved_beliefs.len(), 1);
}

#[test]
fn test_evidence_detail_deserialize() {
    let v = json!({
        "id": "e-1",
        "source_ref": "paper",
        "content": "detailed evidence",
        "polarity": "attacks",
        "weight": 0.6,
        "reliability": 0.75,
        "scope": "domain-A"
    });
    let ed: EvidenceDetail = serde_json::from_value(v).unwrap();
    assert_eq!(ed.id, "e-1");
    assert_eq!(ed.polarity, Polarity::Attacks);
    assert_eq!(ed.scope, Some("domain-A".into()));
}

#[test]
fn test_explanation_result_full() {
    let v = json!({
        "claim": "fire is hot",
        "truth_state": "true",
        "confidence": 0.98,
        "supporting": [{
            "id": "e-s1",
            "source_ref": "observation",
            "content": "touching fire hurts",
            "polarity": "supports",
            "weight": 0.9,
            "reliability": 0.95
        }],
        "attacking": [{
            "id": "e-a1",
            "source_ref": "cold-fire claim",
            "content": "some fires are cold",
            "polarity": "attacks",
            "weight": 0.1,
            "reliability": 0.2
        }],
        "expired": []
    });
    let er: ExplanationResult = serde_json::from_value(v).unwrap();
    assert_eq!(er.claim, "fire is hot");
    assert_eq!(er.supporting.len(), 1);
    assert_eq!(er.attacking.len(), 1);
    assert!(er.expired.is_empty());
}

#[test]
fn test_goal_evaluation_result_deserialize() {
    let v = json!({
        "goal_id": "g-1",
        "status": "completed",
        "completion_fraction": 1.0,
        "blocking_belief_ids": [],
        "supporting_belief_ids": ["b-1", "b-2"]
    });
    let ger: GoalEvaluationResult = serde_json::from_value(v).unwrap();
    assert_eq!(ger.goal_id, "g-1");
    assert_eq!(ger.status, GoalStatus::Completed);
    assert!((ger.completion_fraction - 1.0).abs() < 0.001);
    assert_eq!(ger.supporting_belief_ids.len(), 2);
}

#[test]
fn test_belief_list_response_defaults() {
    // limit defaults to 50 when absent
    let v = json!({
        "beliefs": [],
        "total": 0,
        "offset": 0
    });
    let blr: BeliefListResponse = serde_json::from_value(v).unwrap();
    assert_eq!(blr.limit, 50);
    assert_eq!(blr.total, 0);
}

#[test]
fn test_belief_result_with_evidence_ids() {
    let v = json!({
        "id": "b-ev",
        "truth_state": "true",
        "confidence": 0.9,
        "conflict": false,
        "was_separated": true,
        "memory_tier": "semantic",
        "evidence_ids": ["e-1", "e-2"]
    });
    let br: BeliefResult = serde_json::from_value(v).unwrap();
    assert!(br.was_separated);
    assert_eq!(br.memory_tier, "semantic");
    let ids = br.evidence_ids.unwrap();
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_policy_step_result_with_tool_and_fallback() {
    let v = json!({
        "step_id": 2,
        "action": "execute",
        "tool": "search_tool",
        "conditions": ["if_fact"],
        "fallback": "skip"
    });
    let ps: PolicyStepResult = serde_json::from_value(v).unwrap();
    assert_eq!(ps.step_id, 2);
    assert_eq!(ps.tool, Some("search_tool".into()));
    assert_eq!(ps.conditions[0], "if_fact");
    assert_eq!(ps.fallback, Some("skip".into()));
}

// ── Builder method coverage ──

#[test]
fn test_believe_request_with_tags() {
    let ev = EvidenceInput::new("src", "content");
    let req = BelieveRequest::new("claim", vec![ev]).with_tags(vec!["tag1".into(), "tag2".into()]);
    assert_eq!(req.tags.len(), 2);
    assert_eq!(req.tags[0], "tag1");
}

#[test]
fn test_search_request_builders() {
    let req = SearchRequest::new("query")
        .with_limit(20)
        .with_alpha(0.5)
        .with_conflict_policy(ConflictPolicy::Conservative);
    assert_eq!(req.limit, 20);
    assert!((req.alpha - 0.5).abs() < 0.001);
    assert_eq!(req.conflict_policy, ConflictPolicy::Conservative);
}

#[test]
fn test_frame_open_request_with_ttl() {
    let req = FrameOpenRequest::new("q").with_ttl(600);
    assert_eq!(req.ttl_seconds, 600);
}

#[test]
fn test_lite_frame_open_request_builders() {
    let req = LiteFrameOpenRequest::new("q-1")
        .with_goal_id("g-1")
        .with_top_k(50)
        .with_ttl(120);
    assert_eq!(req.goal_id, Some("g-1".into()));
    assert_eq!(req.top_k, 50);
    assert_eq!(req.ttl_seconds, 120);
}

#[test]
fn test_belief_filters_builders() {
    let f = BeliefFilters::default()
        .with_truth_state(TruthState::Both)
        .with_belief_type(BeliefType::Inference)
        .with_tag("science")
        .with_confidence_range(0.3, 0.9)
        .with_limit(25)
        .with_offset(10);
    assert_eq!(f.truth_state, Some(TruthState::Both));
    assert_eq!(f.belief_type, Some(BeliefType::Inference));
    assert_eq!(f.tag, Some("science".into()));
    assert!((f.min_confidence - 0.3).abs() < 0.001);
    assert!((f.max_confidence - 0.9).abs() < 0.001);
    assert_eq!(f.limit, 25);
    assert_eq!(f.offset, 10);
}

#[test]
fn test_evidence_input_default_weight_and_reliability_via_serde() {
    // Exercises the default_weight() and default_reliability() functions
    let v = serde_json::json!({
        "source_ref": "x",
        "content": "y",
        "polarity": "supports"
    });
    let ev: EvidenceInput = serde_json::from_value(v).unwrap();
    assert!((ev.weight - 0.7).abs() < 0.001);
    assert!((ev.reliability - 0.8).abs() < 0.001);
}

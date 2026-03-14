use serde::{Deserialize, Serialize};

// ── Enums ──

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TruthState {
    True,
    False,
    Both,
    Neither,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BeliefType {
    #[default]
    Fact,
    Preference,
    Inference,
    Prediction,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Polarity {
    #[default]
    Supports,
    Attacks,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxStatus {
    Active,
    Committed,
    Discarded,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitMode {
    Selective,
    All,
    DiscardConflicts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttackType {
    Contradicts,
    Undermines,
    Rebuts,
    Undercuts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    Active,
    Paused,
    Completed,
    Failed,
    Abandoned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Active,
    FlaggedForRevision,
    Superseded,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    #[default]
    Surface,
    Conservative,
    Optimistic,
}

// ── Request types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceInput {
    pub source_ref: String,
    pub content: String,
    #[serde(default)]
    pub polarity: Polarity,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default = "default_reliability")]
    pub reliability: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

fn default_weight() -> f64 {
    0.7
}
fn default_reliability() -> f64 {
    0.8
}

impl EvidenceInput {
    pub fn new(source_ref: &str, content: &str) -> Self {
        Self {
            source_ref: source_ref.into(),
            content: content.into(),
            polarity: Polarity::Supports,
            weight: 0.7,
            reliability: 0.8,
            scope: None,
        }
    }

    pub fn with_polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = polarity;
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_reliability(mut self, reliability: f64) -> Self {
        self.reliability = reliability;
        self
    }

    pub fn with_scope(mut self, scope: &str) -> Self {
        self.scope = Some(scope.into());
        self
    }
}

/// Request struct for the `/believe` endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct BelieveRequest {
    pub claim: String,
    pub evidence: Vec<EvidenceInput>,
    pub belief_type: BeliefType,
    pub tags: Vec<String>,
    pub source_agent: String,
}

impl BelieveRequest {
    /// Create a new request with the given claim and evidence.
    ///
    /// Defaults: `BeliefType::Fact`, no tags, empty source_agent.
    pub fn new(claim: impl Into<String>, evidence: Vec<EvidenceInput>) -> Self {
        Self {
            claim: claim.into(),
            evidence,
            belief_type: BeliefType::Fact,
            tags: Vec::new(),
            source_agent: String::new(),
        }
    }

    pub fn with_belief_type(mut self, belief_type: BeliefType) -> Self {
        self.belief_type = belief_type;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_source_agent(mut self, source_agent: impl Into<String>) -> Self {
        self.source_agent = source_agent.into();
        self
    }
}

/// Request struct for the `/search` endpoint.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub limit: u32,
    pub alpha: f64,
    pub conflict_policy: ConflictPolicy,
}

impl SearchRequest {
    /// Create a new request with the given query.
    ///
    /// Defaults: limit 10, alpha 0.7, `ConflictPolicy::Surface`.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: 10,
            alpha: 0.7,
            conflict_policy: ConflictPolicy::Surface,
        }
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha;
        self
    }

    pub fn with_conflict_policy(mut self, conflict_policy: ConflictPolicy) -> Self {
        self.conflict_policy = conflict_policy;
        self
    }
}

/// Request struct for the `/frame/open` endpoint (full backend API shape).
#[derive(Debug, Clone, Serialize)]
pub struct FrameOpenRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub preload_claims: Vec<String>,
    pub ttl_seconds: u32,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub source_agent: String,
}

impl FrameOpenRequest {
    /// Create a new request with the given query.
    ///
    /// Defaults: no preload_claims, ttl_seconds 300, empty source_agent.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            preload_claims: Vec::new(),
            ttl_seconds: 300,
            source_agent: String::new(),
        }
    }

    pub fn with_preload_claims(mut self, preload_claims: Vec<String>) -> Self {
        self.preload_claims = preload_claims;
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u32) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }

    pub fn with_source_agent(mut self, source_agent: impl Into<String>) -> Self {
        self.source_agent = source_agent.into();
        self
    }
}

/// Request struct for the `/frame/open` endpoint (lite API shape).
#[derive(Debug, Clone, Serialize)]
pub struct LiteFrameOpenRequest {
    pub query_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_id: Option<String>,
    pub top_k: u32,
    pub ttl_seconds: u32,
}

impl LiteFrameOpenRequest {
    pub fn new(query_id: &str) -> Self {
        Self {
            query_id: query_id.into(),
            goal_id: None,
            top_k: 20,
            ttl_seconds: 300,
        }
    }

    pub fn with_goal_id(mut self, goal_id: impl Into<String>) -> Self {
        self.goal_id = Some(goal_id.into());
        self
    }

    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = top_k;
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u32) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truth_state: Option<TruthState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belief_type: Option<BeliefType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub min_confidence: f64,
    pub max_confidence: f64,
    pub limit: u32,
    pub offset: u32,
}

impl Default for BeliefFilters {
    fn default() -> Self {
        Self {
            truth_state: None,
            belief_type: None,
            tag: None,
            min_confidence: 0.0,
            max_confidence: 1.0,
            limit: 50,
            offset: 0,
        }
    }
}

impl BeliefFilters {
    pub fn with_truth_state(mut self, truth_state: TruthState) -> Self {
        self.truth_state = Some(truth_state);
        self
    }

    pub fn with_belief_type(mut self, belief_type: BeliefType) -> Self {
        self.belief_type = Some(belief_type);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn with_confidence_range(mut self, min: f64, max: f64) -> Self {
        self.min_confidence = min;
        self.max_confidence = max;
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }
}

// ── Response types ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeliefResult {
    pub id: String,
    pub truth_state: TruthState,
    pub confidence: f64,
    pub conflict: bool,
    #[serde(default)]
    pub was_separated: bool,
    #[serde(default = "default_memory_tier")]
    pub memory_tier: String,
    #[serde(default)]
    pub evidence_ids: Option<Vec<String>>,
}

fn default_memory_tier() -> String {
    "episodic".into()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvidenceDetail {
    pub id: String,
    pub source_ref: String,
    pub content: String,
    pub polarity: Polarity,
    pub weight: f64,
    pub reliability: f64,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExplanationResult {
    pub claim: String,
    pub truth_state: TruthState,
    pub confidence: f64,
    #[serde(default)]
    pub supporting: Vec<EvidenceDetail>,
    #[serde(default)]
    pub attacking: Vec<EvidenceDetail>,
    #[serde(default)]
    pub expired: Vec<EvidenceDetail>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub belief_id: String,
    pub claim: String,
    pub truth_state: TruthState,
    pub confidence: f64,
    pub similarity: f64,
    pub rank_score: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeliefListItem {
    pub id: String,
    pub claim: String,
    pub belief_type: BeliefType,
    pub truth_state: TruthState,
    pub confidence: f64,
    pub tag_count: i64,
    pub evidence_count: i64,
    pub created_at: String,
    pub last_revised: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeliefListResponse {
    #[serde(default)]
    pub beliefs: Vec<BeliefListItem>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default = "default_list_limit")]
    pub limit: i64,
}

fn default_list_limit() -> i64 {
    50
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeliefSnapshot {
    pub belief_id: String,
    pub claim: String,
    pub truth_state: TruthState,
    pub confidence: f64,
    pub belief_type: BeliefType,
    pub evidence_count: i64,
    pub conflict: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameOpenResult {
    pub frame_id: String,
    pub beliefs_loaded: i64,
    pub conflicts: i64,
    #[serde(default)]
    pub snapshots: Vec<BeliefSnapshot>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameContextResult {
    #[serde(alias = "active_query")]
    pub query: String,
    #[serde(default)]
    pub beliefs: Vec<BeliefSnapshot>,
    #[serde(default)]
    pub scratchpad: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub conflicts: Vec<BeliefSnapshot>,
    #[serde(default)]
    pub step_count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameCommitResult {
    pub frame_id: String,
    pub beliefs_created: i64,
    pub beliefs_revised: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetrievedBelief {
    pub claim: String,
    pub confidence: f64,
    pub similarity: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AskResult {
    pub query_id: String,
    #[serde(default)]
    pub retrieved_beliefs: Vec<RetrievedBelief>,
}

// ── Sandbox ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SandboxResult {
    pub id: String,
    pub frame_id: Option<String>,
    pub scenario_label: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SandboxContextResult {
    pub id: String,
    pub frame_id: Option<String>,
    pub scenario_label: String,
    pub status: String,
    #[serde(default)]
    pub belief_overrides: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub added_belief_ids: Vec<String>,
    #[serde(default)]
    pub invalidated_evidence: Vec<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeliefChangeDetail {
    pub belief_id: String,
    pub field: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SandboxDiffResult {
    #[serde(default)]
    pub belief_changes: Vec<BeliefChangeDetail>,
    #[serde(default)]
    pub evidence_invalidations: Vec<String>,
    #[serde(default)]
    pub new_beliefs: Vec<String>,
    #[serde(default)]
    pub temporary_attacks: Vec<serde_json::Value>,
    #[serde(default)]
    pub goal_changes: Vec<serde_json::Value>,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SandboxCommitResult {
    pub sandbox_id: String,
    #[serde(default)]
    pub committed_belief_ids: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SandboxExplainResult {
    pub belief_id: String,
    pub sandbox_id: String,
    pub resolved_truth_state: TruthState,
    pub has_override: bool,
    #[serde(default)]
    pub override_fields: Vec<String>,
    #[serde(default)]
    pub invalidated_evidence_ids: Vec<String>,
    #[serde(default)]
    pub source: String,
}

// ── Revision ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevisionPolicyResult {
    pub policy_name: String,
    pub max_retraction_depth: i64,
    pub max_retractions: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevisionAuditEntry {
    pub id: String,
    pub timestamp: String,
    pub incoming_belief_id: String,
    pub policy_name: String,
    pub revision_depth: i64,
    pub bounded: bool,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionEvidenceItem {
    #[serde(default)]
    pub source_ref: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub polarity: Polarity,
    #[serde(default = "default_revision_weight")]
    pub weight: f64,
    #[serde(default = "default_revision_reliability")]
    pub reliability: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

fn default_revision_weight() -> f64 {
    0.8
}
fn default_revision_reliability() -> f64 {
    0.7
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevisionResult {
    #[serde(default)]
    pub superseded_evidence_ids: Vec<String>,
    #[serde(default)]
    pub retracted_belief_ids: Vec<String>,
    #[serde(default)]
    pub revision_depth: i64,
    #[serde(default)]
    pub policy_name: String,
    #[serde(default)]
    pub bounded: bool,
}

// ── Attacks ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttackEdgeResult {
    pub id: String,
    pub source_belief_id: String,
    pub target_belief_id: String,
    pub attack_type: AttackType,
    pub weight: f64,
    pub active: bool,
    pub created_at: String,
}

// ── Reconsolidation ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconsolidationQueueResult {
    pub queue_size: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconsolidationRunResult {
    pub processed: i64,
    pub timestamp: String,
}

// ── Goals ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoalResult {
    pub id: String,
    pub goal: String,
    pub owner: String,
    pub priority: f64,
    pub status: GoalStatus,
    pub created_at: String,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub success_criteria: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoalEvaluationResult {
    pub goal_id: String,
    pub status: GoalStatus,
    pub completion_fraction: f64,
    #[serde(default)]
    pub blocking_belief_ids: Vec<String>,
    #[serde(default)]
    pub supporting_belief_ids: Vec<String>,
}

// ── Policies ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyStepResult {
    pub step_id: i64,
    pub action: String,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub fallback: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: i64,
    pub reliability: f64,
    pub status: PolicyStatus,
    pub created_at: String,
    pub last_updated: String,
    #[serde(default)]
    pub superseded_by: Option<String>,
    #[serde(default)]
    pub steps: Vec<PolicyStepResult>,
    #[serde(default)]
    pub applicability: serde_json::Map<String, serde_json::Value>,
}

// ── Consolidation & Memory Tiers ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsolidateResult {
    pub semantic_beliefs_created: i64,
    pub episodics_pruned: i64,
    pub clusters_found: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryTierResult {
    pub belief_id: String,
    pub memory_tier: String,
    pub consolidated_from_count: i64,
}

// ── Multihop ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultihopResultItem {
    pub belief_id: String,
    pub claim: String,
    pub confidence: f64,
    pub truth_state: TruthState,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultihopResponse {
    #[serde(default)]
    pub results: Vec<MultihopResultItem>,
}

// ── Benchmark ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkSandboxResult {
    pub sandbox_id: String,
    pub resolved_truth_state: TruthState,
    pub canonical_unchanged: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkAttackResult {
    pub edge_id: String,
    pub attacker_id: String,
    pub target_id: String,
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed FR-240/FR-286 result construction and immutable document storage.

use quire_observation::authority::{closure as observation_closure, completeness, progress};
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};
use serde::Serialize;
use serde_json::{json, Value};

use crate::assessment::{AxisInputs, GlobalConformanceInput, GlobalPremise};
use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::closure::{AssessmentExecution, GlobalConformanceClosure, ScopeState};
use crate::digest::jcs::{canonicalize, ResultDigest};
use crate::repro::ReproductionRecord;
use crate::ByteDigest;

use super::{global, lineage, ErrorCode, Refusal, Result};

pub(crate) const ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/item/retain"),
    "result::record::produce",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const PREMISE_ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/premise/retain"),
    "result::record::produce",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const SUPPORT_ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/support/retain"),
    "result::record::produce",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const LIMITATION_ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/limitation/retain"),
    "result::record::produce",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const OUTPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/output/allocate"),
    "result::record::produce",
    BudgetDimension::OutputByte,
    SiteKind::Allocation,
);
pub(crate) const STRING_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/output/string/retain"),
    "result::record::produce",
    BudgetDimension::OutputByte,
    SiteKind::Retention,
);
pub(crate) const DEPTH_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/output/container/depth"),
    "result::record::produce",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/output/field/visit"),
    "result::record::produce",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/document/allocate"),
    "result::record::produce",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Canonical result contract label.
pub const CONTRACT: &str = "quire.protocol.result/v1-draft.1";
/// Immutable schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/protocol-result-v1-draft-1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "8825d5b05edf7aef9e53dd117b4f344b824e1c6fb2757071f6dd14a268079dd2";
const IDENTITY_VERSION: &str = "quire.protocol.result-identity/v1-draft.1";

/// Immutable owner maxima. Caller limits may lower, never raise, these values.
pub const OWNER_MAX: Limits = Limits {
    max_input_bytes: 8 * 1024 * 1024,
    max_output_bytes: 8 * 1024 * 1024,
    max_depth: 64,
    max_string_bytes: 1024 * 1024,
    max_claims: 64,
    max_premises: 10_000,
    max_support_records: 10_000,
    max_limitations: 10_000,
    max_lineage_nodes: 10_000,
    max_visited_fields: 1_000_000,
};

/// Independent resource ceilings for result production, reading, and mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum input bytes accepted by a reader.
    pub max_input_bytes: usize,
    /// Maximum canonical bytes emitted by a producer.
    pub max_output_bytes: usize,
    /// Maximum JSON container depth.
    pub max_depth: usize,
    /// Maximum encoded bytes in one string.
    pub max_string_bytes: usize,
    /// Maximum claim records retained in one result.
    pub max_claims: usize,
    /// Maximum global-closure premise records.
    pub max_premises: usize,
    /// Maximum decision-support records.
    pub max_support_records: usize,
    /// Maximum limitation records.
    pub max_limitations: usize,
    /// Maximum correction/support lineage nodes visited.
    pub max_lineage_nodes: usize,
    /// Maximum object members and array values visited.
    pub max_visited_fields: usize,
}

impl Limits {
    /// Immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamp every caller limit to the owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_input_bytes: min(self.max_input_bytes, OWNER_MAX.max_input_bytes),
            max_output_bytes: min(self.max_output_bytes, OWNER_MAX.max_output_bytes),
            max_depth: min(self.max_depth, OWNER_MAX.max_depth),
            max_string_bytes: min(self.max_string_bytes, OWNER_MAX.max_string_bytes),
            max_claims: min(self.max_claims, OWNER_MAX.max_claims),
            max_premises: min(self.max_premises, OWNER_MAX.max_premises),
            max_support_records: min(self.max_support_records, OWNER_MAX.max_support_records),
            max_limitations: min(self.max_limitations, OWNER_MAX.max_limitations),
            max_lineage_nodes: min(self.max_lineage_nodes, OWNER_MAX.max_lineage_nodes),
            max_visited_fields: min(self.max_visited_fields, OWNER_MAX.max_visited_fields),
        }
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

impl Default for Limits {
    fn default() -> Self {
        OWNER_MAX
    }
}

/// Bounded work observed by a result operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Input or output wire bytes observed.
    pub wire_bytes: usize,
    /// Deepest container observed.
    pub depth: usize,
    /// Largest encoded string observed.
    pub string_bytes: usize,
    /// Claim records visited.
    pub claims: usize,
    /// Global premise records visited.
    pub premises: usize,
    /// Decision-support records visited.
    pub support_records: usize,
    /// Limitation records visited.
    pub limitations: usize,
    /// Lineage nodes visited.
    pub lineage_nodes: usize,
    /// Object members and array values visited.
    pub visited_fields: usize,
}

/// FR-242 truth disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Truth {
    Satisfied,
    Violated,
    Pending,
    Unavailable,
}
impl Truth {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Violated => "violated",
            Self::Pending => "pending",
            Self::Unavailable => "unavailable",
        }
    }
}

/// FR-243 settlement basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SettlementBasis {
    ClosedScope,
    DecisiveWitness,
    DecisiveCounterexample,
    Unsettled,
    Unavailable,
}
impl SettlementBasis {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ClosedScope => "closed-scope",
            Self::DecisiveWitness => "decisive-witness",
            Self::DecisiveCounterexample => "decisive-counterexample",
            Self::Unsettled => "unsettled",
            Self::Unavailable => "unavailable",
        }
    }
}

/// FR-244 activation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Activation {
    Triggered,
    Inactive,
    ActivationUnknown,
}
impl Activation {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Triggered => "triggered",
            Self::Inactive => "inactive",
            Self::ActivationUnknown => "activation-unknown",
        }
    }
}

/// FR-245 observation adequacy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Adequacy {
    Adequate,
    Inadequate,
    Unknown,
    NotAssessed,
}
impl Adequacy {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Adequate => "adequate",
            Self::Inadequate => "inadequate",
            Self::Unknown => "unknown",
            Self::NotAssessed => "not-assessed",
        }
    }
}

/// FR-246 protocol adequacy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProtocolAdequacy {
    Demonstrated,
    NotDemonstrated,
    Inapplicable,
    Unknown,
}
impl ProtocolAdequacy {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Demonstrated => "demonstrated",
            Self::NotDemonstrated => "not-demonstrated",
            Self::Inapplicable => "inapplicable",
            Self::Unknown => "unknown",
        }
    }
}

/// FR-280 assumption state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AssumptionState {
    Met,
    NotMet,
    Unknown,
    NotAssessed,
}
impl AssumptionState {
    const fn label(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::NotMet => "not-met",
            Self::Unknown => "unknown",
            Self::NotAssessed => "not-assessed",
        }
    }
}

/// One independently identified assumption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assumption {
    pub identity: String,
    pub state: AssumptionState,
}

/// FR-290 protocol claim kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ClaimKind {
    GlobalConformance,
    Monitorability,
    LocalProjection,
    Refinement,
    Realizability,
    Composition,
}
impl ClaimKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::GlobalConformance => "global-conformance",
            Self::Monitorability => "monitorability",
            Self::LocalProjection => "local-projection",
            Self::Refinement => "refinement",
            Self::Realizability => "realizability",
            Self::Composition => "composition",
        }
    }
}

/// FR-281 claim strength.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ClaimStrength {
    Proven,
    BoundedChecked,
    Tested,
    Observed,
}
impl ClaimStrength {
    const fn label(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::BoundedChecked => "bounded-checked",
            Self::Tested => "tested",
            Self::Observed => "observed",
        }
    }
}

/// Exact claim, admitted-fragment, and backend selections retained by a result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub kind: ClaimKind,
    pub strength: Option<ClaimStrength>,
    pub admitted_fragment_identity: Option<String>,
    pub backend_identity: Option<String>,
}

/// FR-282 dependence relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DependenceRelation {
    Independent,
    Shared,
    Derived,
    Unknown,
}
impl DependenceRelation {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::Shared => "shared",
            Self::Derived => "derived",
            Self::Unknown => "unknown",
        }
    }
}

/// Exact locator for one decision-support record.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SupportRef {
    pub oracle_identity: String,
    pub observation_identity: String,
}

/// FR-248 decision-support record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionSupport {
    pub oracle_identity: String,
    pub observation_identity: String,
    pub relation: DependenceRelation,
    pub predecessor: Option<SupportRef>,
    pub dependence_group: Option<String>,
}

/// Settlement plus the exact fact/progress identities authorizing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    pub basis: SettlementBasis,
    pub supporting_fact_identities: Vec<String>,
    pub supporting_progress_identities: Vec<String>,
}

/// One typed retained limitation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Limitation {
    pub code: String,
    pub affected_identity: String,
}

/// FR-285 closed participation value object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Participation {
    Known {
        required_role_instance_refs: Vec<String>,
        participating_role_instance_refs: Vec<String>,
    },
    Unknown {
        participating_role_instance_refs: Vec<String>,
        missing_dependency_refs: Vec<String>,
        reasons: Vec<String>,
    },
}

/// Complete provenance identity set retained independently of display text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    pub source_identity: String,
    pub profile_identity: String,
    pub model_identity: String,
    pub protocol_identity: String,
    pub request_identity: String,
    pub binding_identity: String,
    pub input_identity: String,
    pub backend_identity: String,
    pub producer_identity: String,
    /// Exact local-run inputs needed to reproduce this result.
    pub reproduction: ReproductionRecord,
}

/// Explicit correction relation to the direct predecessor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectionRelation {
    Supersedes,
    Invalidates,
}
impl CorrectionRelation {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Invalidates => "invalidates",
        }
    }
}

/// Immutable direct correction input. A predecessor must already be validated.
#[derive(Clone, Copy, Debug)]
pub struct CorrectionInput<'a> {
    pub relation: CorrectionRelation,
    pub predecessor: &'a ValidatedResult,
    pub contradicted_premise_identity: &'a str,
    pub corrected_input_identity: &'a str,
}

/// Complete typed input to canonical result production.
#[derive(Clone, Debug)]
pub struct Input<'a> {
    pub checked_predicate: &'a checked_predicate::ValidatedCheckedPredicate,
    pub temporal_subject: &'a temporal_subject::ValidatedTemporalSubject,
    pub axes: AxisInputs<'a>,
    pub completeness: &'a completeness::View,
    pub execution: AssessmentExecution,
    pub truth: Truth,
    pub settlement: Settlement,
    pub activation: Activation,
    pub participation: Participation,
    pub assumptions: Vec<Assumption>,
    pub observation_adequacy: Adequacy,
    pub protocol_adequacy: ProtocolAdequacy,
    pub claim: Claim,
    pub decision_support: Vec<DecisionSupport>,
    pub global_conformance: GlobalConformanceInput,
    pub limitations: Vec<Limitation>,
    pub provenance: ProvenanceRecord,
    pub correction: Option<CorrectionInput<'a>>,
}

/// Immutable canonical result bytes plus identities outside those bytes.
#[derive(Clone, Debug)]
pub struct Document {
    bytes: Vec<u8>,
    result_id: String,
    digest: String,
    pub(crate) usage: Usage,
    pub(crate) snapshot: Snapshot,
}

impl Document {
    /// Complete canonical result bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// FR-286 result identity derived with `resultId` omitted.
    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }
    /// Enclosing digest of the complete canonical result.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Work retained while producing the document.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

/// Strict-reader output. Its fields cannot be constructed outside this module.
#[derive(Clone, Debug)]
pub struct ValidatedResult {
    pub(crate) document: Document,
}

impl ValidatedResult {
    /// Complete canonical bytes accepted by the strict reader.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.document.bytes()
    }
    /// Re-derived FR-286 result identity.
    #[must_use]
    pub fn result_id(&self) -> &str {
        self.document.result_id()
    }
    /// Re-derived enclosing result digest.
    #[must_use]
    pub fn digest(&self) -> &str {
        self.document.digest()
    }
    /// Work observed while reading the result.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.document.usage()
    }
    /// Whether the selected assessment executed.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        self.document.snapshot.execution
    }
    /// Native truth disposition, never Boolean-normalized.
    #[must_use]
    pub const fn truth(&self) -> Truth {
        self.document.snapshot.truth
    }
    /// Exact settlement basis.
    #[must_use]
    pub const fn settlement_basis(&self) -> SettlementBasis {
        self.document.snapshot.settlement.basis
    }
    /// Complete settlement record and its authorizing identities.
    #[must_use]
    pub const fn settlement(&self) -> &Settlement {
        &self.document.snapshot.settlement
    }
    /// Activation state retained independently from truth.
    #[must_use]
    pub const fn activation(&self) -> Activation {
        self.document.snapshot.activation
    }
    /// Observation adequacy retained independently from execution and truth.
    #[must_use]
    pub const fn observation_adequacy(&self) -> Adequacy {
        self.document.snapshot.observation_adequacy
    }
    /// Protocol adequacy retained independently from observation adequacy.
    #[must_use]
    pub const fn protocol_adequacy(&self) -> ProtocolAdequacy {
        self.document.snapshot.protocol_adequacy
    }
    /// Exact claim selection assessed by this result.
    #[must_use]
    pub const fn claim(&self) -> &Claim {
        &self.document.snapshot.claim
    }
    /// Assumptions retained in their declared order.
    #[must_use]
    pub fn assumptions(&self) -> &[Assumption] {
        &self.document.snapshot.assumptions
    }
    /// Participation state and exact role/dependency identities.
    #[must_use]
    pub const fn participation(&self) -> &Participation {
        &self.document.snapshot.participation
    }
    /// Decision support retained in declared causal order.
    #[must_use]
    pub fn decision_support(&self) -> &[DecisionSupport] {
        &self.document.snapshot.decision_support
    }
    /// Global-conformance closure state.
    #[must_use]
    pub const fn global_conformance_state(&self) -> GlobalConformanceClosure {
        self.document.snapshot.global_conformance_state
    }
    /// Complete global-conformance premise population.
    #[must_use]
    pub fn global_premises(&self) -> &[GlobalPremise] {
        &self.document.snapshot.global_premises
    }
    /// Retained limitations in canonical order.
    #[must_use]
    pub fn limitations(&self) -> &[Limitation] {
        &self.document.snapshot.limitations
    }
    /// Complete provenance selections, including reproduction inputs.
    #[must_use]
    pub const fn provenance(&self) -> &ProvenanceRecord {
        &self.document.snapshot.provenance
    }
    /// Decision-scope progress assertion view.
    #[must_use]
    pub const fn decision_scope_progress(&self) -> &AxisView {
        &self.document.snapshot.axes[0]
    }
    /// Decision-scope closure assertion view.
    #[must_use]
    pub const fn decision_scope_closure(&self) -> &AxisView {
        &self.document.snapshot.axes[1]
    }
    /// Surrounding-execution progress assertion view.
    #[must_use]
    pub const fn surrounding_execution_progress(&self) -> &AxisView {
        &self.document.snapshot.axes[2]
    }
    /// Surrounding-execution closure assertion view.
    #[must_use]
    pub const fn surrounding_execution_closure(&self) -> &AxisView {
        &self.document.snapshot.axes[3]
    }
    /// Complete owner-issued completeness assertion view.
    #[must_use]
    pub const fn completeness(&self) -> &CompletenessView {
        &self.document.snapshot.completeness
    }
    /// Direct correction predecessor, when this result is a correction.
    #[must_use]
    pub const fn correction(&self) -> Option<&CorrectionView> {
        self.document.snapshot.correction.as_ref()
    }
    /// Completeness state retained for source-compatible callers.
    #[must_use]
    pub const fn completeness_state(&self) -> completeness::State {
        self.document.snapshot.completeness.state
    }
    /// Native temporal-subject identity.
    #[must_use]
    pub fn native_subject_identity(&self) -> &str {
        &self.document.snapshot.native_subject_identity
    }
    /// Source-bound checked-predicate/correspondence identity.
    #[must_use]
    pub fn native_correspondence_identity(&self) -> &str {
        &self.document.snapshot.native_correspondence_identity
    }
}

/// Constructor-private owner assertion retained by a validated result.
#[derive(Clone, Debug)]
pub struct AxisView {
    pub(crate) contract: &'static str,
    pub(crate) schema_sha256: &'static str,
    pub(crate) document_identity: String,
    pub(crate) document_digest: String,
    pub(crate) authority_identity: String,
    pub(crate) authority_revision: String,
    pub(crate) authority_digest: String,
    pub(crate) scope_identity: String,
    pub(crate) population_identity: String,
    pub(crate) state: ScopeState,
}

impl AxisView {
    /// Selected owner contract.
    #[must_use]
    pub const fn contract(&self) -> &str {
        self.contract
    }
    /// Selected owner schema digest.
    #[must_use]
    pub const fn schema_sha256(&self) -> &str {
        self.schema_sha256
    }
    /// Canonical assertion-document identity.
    #[must_use]
    pub fn document_identity(&self) -> &str {
        &self.document_identity
    }
    /// Digest of the canonical assertion bytes.
    #[must_use]
    pub fn document_digest(&self) -> &str {
        &self.document_digest
    }
    /// Definition authority identity.
    #[must_use]
    pub fn authority_identity(&self) -> &str {
        &self.authority_identity
    }
    /// Definition authority revision.
    #[must_use]
    pub fn authority_revision(&self) -> &str {
        &self.authority_revision
    }
    /// Definition authority digest bytes rendered as lowercase hex.
    #[must_use]
    pub fn authority_digest(&self) -> &str {
        &self.authority_digest
    }
    /// Exact scope identity.
    #[must_use]
    pub fn scope_identity(&self) -> &str {
        &self.scope_identity
    }
    /// Exact population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }
    /// Independent open/closed state.
    #[must_use]
    pub const fn state(&self) -> ScopeState {
        self.state
    }
}

/// Constructor-private completeness assertion retained by a validated result.
#[derive(Clone, Debug)]
pub struct CompletenessView {
    pub(crate) document_identity: String,
    pub(crate) document_digest: String,
    pub(crate) population_identity: String,
    pub(crate) boundary_identity: String,
    pub(crate) fact_identities: Vec<String>,
    pub(crate) state: completeness::State,
}

impl CompletenessView {
    /// Canonical completeness-document identity.
    #[must_use]
    pub fn document_identity(&self) -> &str {
        &self.document_identity
    }
    /// Digest of the canonical completeness bytes.
    #[must_use]
    pub fn document_digest(&self) -> &str {
        &self.document_digest
    }
    /// Exact population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }
    /// Exact temporal boundary identity.
    #[must_use]
    pub fn boundary_identity(&self) -> &str {
        &self.boundary_identity
    }
    /// Facts establishing or contradicting completeness.
    #[must_use]
    pub fn fact_identities(&self) -> &[String] {
        &self.fact_identities
    }
    /// Complete, incomplete, or contradicted state.
    #[must_use]
    pub const fn state(&self) -> completeness::State {
        self.state
    }
}

/// Constructor-private direct correction edge retained by a validated result.
#[derive(Clone, Debug)]
pub struct CorrectionView {
    pub(crate) relation: CorrectionRelation,
    pub(crate) predecessor_result_id: String,
    pub(crate) predecessor_digest: String,
    pub(crate) contradicted_premise_identity: String,
    pub(crate) corrected_input_identity: String,
}

impl CorrectionView {
    /// Correction relation.
    #[must_use]
    pub const fn relation(&self) -> CorrectionRelation {
        self.relation
    }
    /// Direct predecessor result identity.
    #[must_use]
    pub fn predecessor_result_id(&self) -> &str {
        &self.predecessor_result_id
    }
    /// Direct predecessor digest.
    #[must_use]
    pub fn predecessor_digest(&self) -> &str {
        &self.predecessor_digest
    }
    /// Premise contradicted by the correction.
    #[must_use]
    pub fn contradicted_premise_identity(&self) -> &str {
        &self.contradicted_premise_identity
    }
    /// Replacement input identity.
    #[must_use]
    pub fn corrected_input_identity(&self) -> &str {
        &self.corrected_input_identity
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub native_subject_identity: String,
    pub native_correspondence_identity: String,
    pub execution: AssessmentExecution,
    pub truth: Truth,
    pub activation: Activation,
    pub observation_adequacy: Adequacy,
    pub protocol_adequacy: ProtocolAdequacy,
    pub claim: Claim,
    pub assumptions: Vec<Assumption>,
    pub participation: Participation,
    pub settlement: Settlement,
    pub completeness: CompletenessView,
    pub axes: [AxisView; 4],
    pub decision_support: Vec<DecisionSupport>,
    pub global_conformance_state: GlobalConformanceClosure,
    pub global_premises: Vec<GlobalPremise>,
    pub limitations: Vec<Limitation>,
    pub provenance: ProvenanceRecord,
    pub correction: Option<CorrectionView>,
}

struct Prepared {
    snapshot: Snapshot,
    value: Value,
    result_id: String,
    semantic_usage: Usage,
}

/// Produce one canonical FR-286 result or one typed refusal with no partial output.
pub(crate) fn produce(input: Input<'_>, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    finish(prepare(input, effective)?, effective)
}

/// Produce one canonical result under the shared QProtocol caller budget.
///
/// Canonical bytes are produced once, then their exact structural usage is
/// charged against both owner and caller ceilings before the document is
/// retained.
pub(crate) fn produce_bounded(
    input: Input<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Document, Refusal> {
    resources.run_unit(
        UnitLabel::new("result::record::produce_bounded"),
        |resources| produce_bounded_unit(input, limits, resources),
    )
}

fn produce_bounded_unit(
    input: Input<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Document, Refusal> {
    let effective = limits.effective();
    let semantic = semantic_usage(&input);
    let reproduction_string = largest_reproduction_string(&input).unwrap_or(usize::MAX);
    let charges = [
        (ALLOCATION_SOURCE.id(), 1, None),
        (
            ITEM_SOURCE.id(),
            semantic.claims,
            Some(effective.max_claims),
        ),
        (
            PREMISE_ITEM_SOURCE.id(),
            semantic.premises,
            Some(effective.max_premises),
        ),
        (
            SUPPORT_ITEM_SOURCE.id(),
            semantic.support_records,
            Some(effective.max_support_records),
        ),
        (
            LIMITATION_ITEM_SOURCE.id(),
            semantic.limitations,
            Some(effective.max_limitations),
        ),
        (
            super::lineage::LINEAGE_SOURCE.id(),
            semantic.lineage_nodes,
            Some(effective.max_lineage_nodes),
        ),
        (
            STRING_SOURCE.id(),
            reproduction_string,
            Some(effective.max_string_bytes),
        ),
    ];
    if let Err(error) = resources.charge_all_mixed(&charges) {
        return OperationOutcome::from_charge(error);
    }
    let prepared = match prepare(input, effective) {
        Ok(prepared) => prepared,
        Err(error) => return OperationOutcome::DomainError(error),
    };
    let canonical = match canonicalize(&prepared.value) {
        Ok(canonical) => canonical,
        Err(_) => {
            return OperationOutcome::DomainError(refusal(
                ErrorCode::InvalidSelection,
                "result cannot produce its canonical output",
                Usage::default(),
                Some(&prepared.result_id),
            ));
        }
    };
    if let Err(error) = resources.charge_with_limit(
        OUTPUT_SOURCE.id(),
        canonical.len(),
        effective.max_output_bytes,
    ) {
        return OperationOutcome::from_charge(error);
    }
    let structural = super::reader::structural_usage(canonical.as_bytes());
    let additional_string = structural.string_bytes.saturating_sub(reproduction_string);
    let structural_charges = [
        (
            STRING_SOURCE.id(),
            additional_string,
            Some(effective.max_string_bytes),
        ),
        (
            DEPTH_SOURCE.id(),
            structural.depth,
            Some(effective.max_depth),
        ),
        (
            OPERATION_SOURCE.id(),
            structural.fields,
            Some(effective.max_visited_fields),
        ),
    ];
    if let Err(error) = resources.charge_all_mixed(&structural_charges) {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(finish_canonical(prepared, canonical, effective))
}

fn prepare(input: Input<'_>, effective: Limits) -> Result<Prepared> {
    validate_input(&input, effective)?;
    let semantic_usage = semantic_usage(&input);
    let snapshot = snapshot(&input);
    let mut value = result_value(&input, &snapshot, None)?;
    let preimage = json!({"identityVersion": IDENTITY_VERSION, "result": value});
    let result_id = ResultDigest::of_result(&preimage)
        .map_err(|_| {
            refusal(
                ErrorCode::InvalidSelection,
                "result identity preimage is outside the supported JCS domain",
                Usage::default(),
                None,
            )
        })?
        .text();
    value = result_value(&input, &snapshot, Some(&result_id))?;
    Ok(Prepared {
        snapshot,
        value,
        result_id,
        semantic_usage,
    })
}

fn finish(prepared: Prepared, effective: Limits) -> Result<Document> {
    let canonical = canonicalize(&prepared.value).map_err(|_| {
        refusal(
            ErrorCode::InvalidSelection,
            "result contains an unsupported JCS number",
            Usage::default(),
            Some(&prepared.result_id),
        )
    })?;
    finish_canonical(prepared, canonical, effective)
}

fn finish_canonical(prepared: Prepared, canonical: String, effective: Limits) -> Result<Document> {
    let Prepared {
        snapshot,
        value,
        result_id,
        semantic_usage,
    } = prepared;
    let bytes = canonical.into_bytes();
    let usage = super::reader::preflight_output(&bytes, effective, semantic_usage)?;
    let digest = ResultDigest::of_result(&value)
        .map_err(|_| {
            refusal(
                ErrorCode::InvalidSelection,
                "result is outside the supported JCS domain",
                usage,
                Some(&result_id),
            )
        })?
        .text();
    Ok(Document {
        bytes,
        result_id,
        digest,
        usage,
        snapshot,
    })
}

fn largest_reproduction_string(input: &Input<'_>) -> Option<usize> {
    let provenance = &input.provenance;
    Some(
        provenance
            .reproduction
            .toolchain()
            .bytes()
            .len()
            .checked_mul(2)?
            .max(
                provenance
                    .reproduction
                    .corpus()
                    .bytes()
                    .len()
                    .checked_mul(2)?,
            )
            .max(provenance.reproduction.seed().to_string().len()),
    )
}

fn validate_input(input: &Input<'_>, limits: Limits) -> Result<()> {
    if limits.max_claims < 1 {
        return Err(refusal(
            ErrorCode::ResourceIncomplete,
            "claim ceiling does not admit the required claim",
            Usage {
                claims: 1,
                ..Usage::default()
            },
            None,
        ));
    }
    if input.global_conformance.premises.len() > limits.max_premises
        || input.global_conformance.required_premises.len() > limits.max_premises
        || input.decision_support.len() > limits.max_support_records
        || input.limitations.len() > limits.max_limitations
    {
        return Err(refusal(
            ErrorCode::ResourceIncomplete,
            "a result collection exceeds its effective ceiling",
            semantic_usage(input),
            None,
        ));
    }
    validate_nonempty(input, limits)?;
    global::validate(&input.claim, &input.global_conformance, limits)?;
    lineage::validate_support(&input.decision_support, limits)?;
    lineage::validate_correction(input.correction, limits)?;
    validate_axes(input)?;
    validate_combination(input)?;
    validate_unique_strings(input)?;
    Ok(())
}

fn validate_nonempty(input: &Input<'_>, limits: Limits) -> Result<()> {
    let p = &input.provenance;
    let required = [
        p.source_identity.as_str(),
        p.profile_identity.as_str(),
        p.model_identity.as_str(),
        p.protocol_identity.as_str(),
        p.request_identity.as_str(),
        p.binding_identity.as_str(),
        p.input_identity.as_str(),
        p.backend_identity.as_str(),
        p.producer_identity.as_str(),
        input.checked_predicate.document().identity(),
        input.temporal_subject.document().identity(),
    ];
    if required.iter().any(|value| value.is_empty()) {
        return Err(refusal(
            ErrorCode::InvalidSelection,
            "a required exact identity is empty",
            Usage::default(),
            None,
        ));
    }
    let Some(largest_reproduction_string) = largest_reproduction_string(input) else {
        return Err(refusal(
            ErrorCode::ResourceIncomplete,
            "a reproduction identity cannot be represented within the result domain",
            Usage {
                string_bytes: usize::MAX,
                ..Usage::default()
            },
            None,
        ));
    };
    if largest_reproduction_string > limits.max_string_bytes {
        return Err(refusal(
            ErrorCode::ResourceIncomplete,
            "a reproduction identity exceeds the effective string ceiling",
            Usage {
                string_bytes: largest_reproduction_string,
                ..Usage::default()
            },
            None,
        ));
    }
    let advanced = input.claim.strength.is_some()
        || input.claim.admitted_fragment_identity.is_some()
        || input.claim.backend_identity.is_some();
    if advanced
        && (input.claim.strength.is_none()
            || input
                .claim
                .admitted_fragment_identity
                .as_deref()
                .is_none_or(str::is_empty)
            || input
                .claim
                .backend_identity
                .as_deref()
                .is_none_or(str::is_empty))
    {
        return Err(refusal(
            ErrorCode::InvalidSelection,
            "an advanced claim requires strength, fragment, and backend together",
            Usage::default(),
            None,
        ));
    }
    Ok(())
}

fn validate_axes(input: &Input<'_>) -> Result<()> {
    let a = input.axes;
    let decision_subject = a.decision_scope_progress.subject();
    let decision_closure_subject = a.decision_scope_closure.subject();
    let surrounding_subject = a.surrounding_execution_progress.subject();
    let surrounding_closure_subject = a.surrounding_execution_closure.subject();
    let paired = decision_subject == decision_closure_subject
        && surrounding_subject == surrounding_closure_subject;
    let distinct = decision_subject.scope_identity != surrounding_subject.scope_identity;
    let completeness_population = input.completeness.payload().population_identity();
    if !paired
        || !distinct
        || completeness_population != decision_subject.population_identity.as_str()
    {
        return Err(refusal(
            ErrorCode::AxisMismatch,
            "scope owner assertions are cross-wired or completeness names another population",
            Usage::default(),
            None,
        ));
    }
    Ok(())
}

fn validate_combination(input: &Input<'_>) -> Result<()> {
    use AssessmentExecution as E;
    use SettlementBasis as S;
    use Truth as T;
    let progress = input.axes.decision_scope_progress.payload().state();
    let closure = input.axes.decision_scope_closure.payload().state();
    let completeness = input.completeness.payload().state();
    let valid = match (
        input.execution,
        input.truth,
        input.settlement.basis,
        input.activation,
    ) {
        (E::Completed, T::Satisfied | T::Violated, S::ClosedScope, Activation::Triggered) => {
            progress == quire_observation::authority::OpenClosed::Closed
                && closure == quire_observation::authority::OpenClosed::Closed
                && completeness == completeness::State::Complete
                && !input.decision_support.is_empty()
        }
        (E::Completed, T::Satisfied, S::DecisiveWitness, Activation::Triggered)
        | (E::Completed, T::Violated, S::DecisiveCounterexample, Activation::Triggered) => {
            closure == quire_observation::authority::OpenClosed::Open
                && !input.decision_support.is_empty()
        }
        (E::Completed, T::Pending, S::Unsettled, Activation::Triggered) => {
            progress == quire_observation::authority::OpenClosed::Open
        }
        (
            E::Completed,
            T::Unavailable,
            S::Unavailable,
            Activation::Inactive | Activation::ActivationUnknown,
        ) => true,
        (E::Completed, T::Unavailable, S::Unavailable, Activation::Triggered) => {
            input.decision_support.is_empty()
        }
        (
            E::Refused | E::Unsupported | E::ResourceIncomplete | E::Failed,
            T::Unavailable,
            S::Unavailable,
            _,
        ) => true,
        _ => false,
    };
    if !valid {
        return Err(refusal(
            ErrorCode::InvalidCombination,
            "execution, truth, settlement, activation, and support do not form an FR-061 row",
            Usage::default(),
            None,
        ));
    }
    if matches!(input.truth, T::Satisfied | T::Violated)
        && input.decision_support.iter().any(|support| {
            !completeness_has_available(input.completeness, &support.observation_identity)
        })
    {
        return Err(refusal(
            ErrorCode::InvalidCombination,
            "settled truth has incomplete decision support",
            Usage::default(),
            None,
        ));
    }
    validate_settlement_support(input)?;
    Ok(())
}

fn validate_settlement_support(input: &Input<'_>) -> Result<()> {
    let decision_facts: std::collections::BTreeSet<_> = input
        .decision_support
        .iter()
        .map(|support| support.observation_identity.as_str())
        .collect();
    let settlement_facts: std::collections::BTreeSet<_> = input
        .settlement
        .supporting_fact_identities
        .iter()
        .map(String::as_str)
        .collect();
    if decision_facts != settlement_facts {
        return Err(refusal(
            ErrorCode::InvalidCombination,
            "settlement facts differ from the exact decision-support population",
            Usage::default(),
            None,
        ));
    }

    let decision_progress = input.axes.decision_scope_progress.identity().as_str();
    let surrounding_progress = input
        .axes
        .surrounding_execution_progress
        .identity()
        .as_str();
    let progress = &input.settlement.supporting_progress_identities;
    if !progress
        .iter()
        .any(|identity| identity == decision_progress)
        || progress
            .iter()
            .any(|identity| identity != decision_progress && identity != surrounding_progress)
    {
        return Err(refusal(
            ErrorCode::InvalidCombination,
            "settlement progress does not name the exact selected progress assertions",
            Usage::default(),
            None,
        ));
    }
    Ok(())
}

fn completeness_has_available(view: &completeness::View, observation_identity: &str) -> bool {
    view.payload().facts().any(|fact| {
        fact.observation_identity == Some(observation_identity)
            && fact.status == completeness::FactStatus::Available
    })
}

fn validate_unique_strings(input: &Input<'_>) -> Result<()> {
    fn unique(mut values: impl Iterator<Item = String>) -> bool {
        let mut seen = std::collections::BTreeSet::new();
        values.all(|value| !value.is_empty() && seen.insert(value))
    }
    if !unique(input.assumptions.iter().map(|value| value.identity.clone()))
        || !unique(input.limitations.iter().map(|value| value.code.clone()))
        || !unique(input.settlement.supporting_fact_identities.iter().cloned())
        || !unique(
            input
                .settlement
                .supporting_progress_identities
                .iter()
                .cloned(),
        )
    {
        return Err(refusal(
            ErrorCode::InvalidSelection,
            "a canonical result collection is empty-valued or duplicated",
            Usage::default(),
            None,
        ));
    }
    match &input.participation {
        Participation::Known {
            required_role_instance_refs,
            participating_role_instance_refs,
        } => {
            let required: std::collections::BTreeSet<_> =
                required_role_instance_refs.iter().collect();
            if required.len() != required_role_instance_refs.len()
                || participating_role_instance_refs
                    .iter()
                    .any(|value| !required.contains(value))
                || !unique(participating_role_instance_refs.iter().cloned())
            {
                return Err(refusal(
                    ErrorCode::InvalidSelection,
                    "known participation is duplicated or not a subset",
                    Usage::default(),
                    None,
                ));
            }
        }
        Participation::Unknown {
            participating_role_instance_refs,
            missing_dependency_refs,
            reasons,
        } => {
            if missing_dependency_refs.is_empty()
                || reasons.is_empty()
                || !unique(participating_role_instance_refs.iter().cloned())
                || !unique(missing_dependency_refs.iter().cloned())
                || !unique(reasons.iter().cloned())
            {
                return Err(refusal(ErrorCode::InvalidSelection, "unknown participation requires distinct participants, dependencies, and reasons", Usage::default(), None));
            }
        }
    }
    Ok(())
}

fn snapshot(input: &Input<'_>) -> Snapshot {
    let axes = [
        progress_axis(input.axes.decision_scope_progress),
        closure_axis(input.axes.decision_scope_closure),
        progress_axis(input.axes.surrounding_execution_progress),
        closure_axis(input.axes.surrounding_execution_closure),
    ];
    let completeness_facts = input
        .completeness
        .payload()
        .facts()
        .map(|fact| {
            fact.observation_identity
                .unwrap_or(fact.member_identity)
                .to_owned()
        })
        .collect();
    Snapshot {
        native_subject_identity: input.temporal_subject.document().identity().to_owned(),
        native_correspondence_identity: input.checked_predicate.document().identity().to_owned(),
        execution: input.execution,
        truth: input.truth,
        activation: input.activation,
        observation_adequacy: input.observation_adequacy,
        protocol_adequacy: input.protocol_adequacy,
        claim: input.claim.clone(),
        assumptions: input.assumptions.clone(),
        participation: input.participation.clone(),
        settlement: input.settlement.clone(),
        completeness: CompletenessView {
            state: input.completeness.payload().state(),
            document_identity: input.completeness.identity().as_str().to_owned(),
            document_digest: raw_digest(input.completeness.bytes()),
            population_identity: input
                .completeness
                .payload()
                .population_identity()
                .to_owned(),
            boundary_identity: input.completeness.payload().boundary_identity().to_owned(),
            fact_identities: completeness_facts,
        },
        axes,
        decision_support: input.decision_support.clone(),
        global_conformance_state: input.global_conformance.state,
        global_premises: input.global_conformance.premises.clone(),
        limitations: input.limitations.clone(),
        provenance: input.provenance.clone(),
        correction: input.correction.map(|value| CorrectionView {
            relation: value.relation,
            predecessor_result_id: value.predecessor.result_id().to_owned(),
            predecessor_digest: value.predecessor.digest().to_owned(),
            contradicted_premise_identity: value.contradicted_premise_identity.to_owned(),
            corrected_input_identity: value.corrected_input_identity.to_owned(),
        }),
    }
}

fn progress_axis(view: &progress::View) -> AxisView {
    AxisView {
        contract: progress::CONTRACT,
        schema_sha256: progress::SCHEMA_SHA256,
        document_identity: view.identity().as_str().to_owned(),
        document_digest: raw_digest(view.bytes()),
        authority_identity: view.authority().definition_identity.as_str().to_owned(),
        authority_revision: view.authority().definition_revision.as_str().to_owned(),
        authority_digest: digest_hex(view.authority().definition_digest.as_bytes()),
        scope_identity: view.subject().scope_identity.as_str().to_owned(),
        population_identity: view.subject().population_identity.as_str().to_owned(),
        state: open_closed(view.payload().state()),
    }
}

fn closure_axis(view: &observation_closure::View) -> AxisView {
    AxisView {
        contract: observation_closure::CONTRACT,
        schema_sha256: observation_closure::SCHEMA_SHA256,
        document_identity: view.identity().as_str().to_owned(),
        document_digest: raw_digest(view.bytes()),
        authority_identity: view.authority().definition_identity.as_str().to_owned(),
        authority_revision: view.authority().definition_revision.as_str().to_owned(),
        authority_digest: digest_hex(view.authority().definition_digest.as_bytes()),
        scope_identity: view.subject().scope_identity.as_str().to_owned(),
        population_identity: view.subject().population_identity.as_str().to_owned(),
        state: open_closed(view.payload().state()),
    }
}

const fn open_closed(state: quire_observation::authority::OpenClosed) -> ScopeState {
    match state {
        quire_observation::authority::OpenClosed::Open => ScopeState::Open,
        quire_observation::authority::OpenClosed::Closed => ScopeState::Closed,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultWire<'a> {
    activation: &'static str,
    assumptions: Vec<AssumptionWire<'a>>,
    claim: ClaimWire<'a>,
    completeness: CompletenessWire<'a>,
    contract_version: &'static str,
    correction: Option<CorrectionWire<'a>>,
    decision_scope_closure: AxisWire<'a>,
    decision_scope_progress: AxisWire<'a>,
    decision_support: Vec<SupportWire<'a>>,
    execution: &'static str,
    global_conformance: GlobalWire<'a>,
    kind: &'static str,
    limitations: Vec<LimitationWire<'a>>,
    native_correspondence_identity: &'a str,
    native_subject_identity: &'a str,
    observation_adequacy: &'static str,
    participation: ParticipationWire<'a>,
    protocol_adequacy: &'static str,
    provenance: ProvenanceWire<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_id: Option<&'a str>,
    settlement: SettlementWire<'a>,
    surrounding_execution_closure: AxisWire<'a>,
    surrounding_execution_progress: AxisWire<'a>,
    truth: &'static str,
}

#[derive(Serialize)]
struct AssumptionWire<'a> {
    identity: &'a str,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClaimWire<'a> {
    admitted_fragment_identity: Option<&'a str>,
    backend_identity: Option<&'a str>,
    kind: &'static str,
    strength: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AxisWire<'a> {
    authority_digest: &'a str,
    authority_identity: &'a str,
    authority_revision: &'a str,
    contract: &'static str,
    document_digest: &'a str,
    document_identity: &'a str,
    population_identity: &'a str,
    schema_sha256: &'static str,
    scope_identity: &'a str,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletenessWire<'a> {
    boundary_identity: &'a str,
    contract: &'static str,
    document_digest: &'a str,
    document_identity: &'a str,
    fact_identities: &'a [String],
    population_identity: &'a str,
    schema_sha256: &'static str,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CorrectionWire<'a> {
    contradicted_premise_identity: &'a str,
    corrected_input_identity: &'a str,
    predecessor_digest: &'a str,
    predecessor_result_id: &'a str,
    relation: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SupportWire<'a> {
    dependence_group: Option<&'a str>,
    observation_identity: &'a str,
    oracle_identity: &'a str,
    predecessor: Option<SupportRefWire<'a>>,
    relation: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct SupportRefWire<'a> {
    observation_identity: &'a str,
    oracle_identity: &'a str,
}

#[derive(Serialize)]
struct GlobalWire<'a> {
    premises: Vec<PremiseWire<'a>>,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PremiseWire<'a> {
    authority_identity: &'a str,
    identity: &'a str,
    kind: &'static str,
    scope_identity: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LimitationWire<'a> {
    affected_identity: &'a str,
    code: &'a str,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
enum ParticipationWire<'a> {
    #[serde(rename = "known")]
    Known {
        #[serde(rename = "participatingRoleInstanceRefs")]
        participating_role_instance_refs: &'a [String],
        #[serde(rename = "requiredRoleInstanceRefs")]
        required_role_instance_refs: &'a [String],
    },
    #[serde(rename = "unknown")]
    Unknown {
        #[serde(rename = "missingDependencyRefs")]
        missing_dependency_refs: &'a [String],
        #[serde(rename = "participatingRoleInstanceRefs")]
        participating_role_instance_refs: &'a [String],
        reasons: &'a [String],
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProvenanceWire<'a> {
    backend_identity: &'a str,
    binding_identity: &'a str,
    input_identity: &'a str,
    model_identity: &'a str,
    producer_identity: &'a str,
    profile_identity: &'a str,
    protocol_identity: &'a str,
    reproduction: ReproductionWire,
    request_identity: &'a str,
    source_identity: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReproductionWire {
    corpus_identity: String,
    seed: String,
    toolchain_identity: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettlementWire<'a> {
    basis: &'static str,
    supporting_fact_identities: &'a [String],
    supporting_progress_identities: &'a [String],
}

fn result_value(input: &Input<'_>, snapshot: &Snapshot, result_id: Option<&str>) -> Result<Value> {
    let axes = &snapshot.axes;
    let wire = ResultWire {
        activation: input.activation.label(),
        assumptions: input.assumptions.iter().map(assumption_wire).collect(),
        claim: claim_wire(&input.claim),
        completeness: completeness_wire(&snapshot.completeness),
        contract_version: CONTRACT,
        correction: snapshot.correction.as_ref().map(correction_wire),
        decision_scope_closure: axis_wire(&axes[1]),
        decision_scope_progress: axis_wire(&axes[0]),
        decision_support: input.decision_support.iter().map(support_wire).collect(),
        execution: input.execution.label(),
        global_conformance: global_wire(&input.global_conformance),
        kind: "ProtocolResult",
        limitations: input.limitations.iter().map(limitation_wire).collect(),
        native_correspondence_identity: &snapshot.native_correspondence_identity,
        native_subject_identity: &snapshot.native_subject_identity,
        observation_adequacy: input.observation_adequacy.label(),
        participation: participation_wire(&input.participation),
        protocol_adequacy: input.protocol_adequacy.label(),
        provenance: provenance_wire(&input.provenance)?,
        result_id,
        settlement: settlement_wire(&input.settlement),
        surrounding_execution_closure: axis_wire(&axes[3]),
        surrounding_execution_progress: axis_wire(&axes[2]),
        truth: input.truth.label(),
    };
    serde_json::to_value(wire).map_err(|_| {
        refusal(
            ErrorCode::InvalidSelection,
            "typed result record cannot be represented as JSON",
            Usage::default(),
            result_id,
        )
    })
}

fn assumption_wire(assumption: &Assumption) -> AssumptionWire<'_> {
    AssumptionWire {
        identity: &assumption.identity,
        state: assumption.state.label(),
    }
}

fn claim_wire(claim: &Claim) -> ClaimWire<'_> {
    ClaimWire {
        admitted_fragment_identity: claim.admitted_fragment_identity.as_deref(),
        backend_identity: claim.backend_identity.as_deref(),
        kind: claim.kind.label(),
        strength: claim.strength.map(ClaimStrength::label),
    }
}

fn axis_wire(axis: &AxisView) -> AxisWire<'_> {
    AxisWire {
        authority_digest: &axis.authority_digest,
        authority_identity: &axis.authority_identity,
        authority_revision: &axis.authority_revision,
        contract: axis.contract,
        document_digest: &axis.document_digest,
        document_identity: &axis.document_identity,
        population_identity: &axis.population_identity,
        schema_sha256: axis.schema_sha256,
        scope_identity: &axis.scope_identity,
        state: axis.state.label(),
    }
}

fn completeness_wire(completeness: &CompletenessView) -> CompletenessWire<'_> {
    CompletenessWire {
        boundary_identity: &completeness.boundary_identity,
        contract: completeness::CONTRACT,
        document_digest: &completeness.document_digest,
        document_identity: &completeness.document_identity,
        fact_identities: &completeness.fact_identities,
        population_identity: &completeness.population_identity,
        schema_sha256: completeness::SCHEMA_SHA256,
        state: completeness_label(completeness.state),
    }
}

fn correction_wire(correction: &CorrectionView) -> CorrectionWire<'_> {
    CorrectionWire {
        contradicted_premise_identity: &correction.contradicted_premise_identity,
        corrected_input_identity: &correction.corrected_input_identity,
        predecessor_digest: &correction.predecessor_digest,
        predecessor_result_id: &correction.predecessor_result_id,
        relation: correction.relation.label(),
    }
}

fn support_wire(support: &DecisionSupport) -> SupportWire<'_> {
    SupportWire {
        dependence_group: support.dependence_group.as_deref(),
        observation_identity: &support.observation_identity,
        oracle_identity: &support.oracle_identity,
        predecessor: support
            .predecessor
            .as_ref()
            .map(|predecessor| SupportRefWire {
                observation_identity: &predecessor.observation_identity,
                oracle_identity: &predecessor.oracle_identity,
            }),
        relation: support.relation.label(),
    }
}

fn global_wire(global: &GlobalConformanceInput) -> GlobalWire<'_> {
    GlobalWire {
        premises: global.premises.iter().map(premise_wire).collect(),
        state: global.state.label(),
    }
}

fn premise_wire(premise: &GlobalPremise) -> PremiseWire<'_> {
    PremiseWire {
        authority_identity: &premise.authority_identity,
        identity: &premise.identity,
        kind: premise.kind.label(),
        scope_identity: &premise.scope_identity,
    }
}

fn limitation_wire(limitation: &Limitation) -> LimitationWire<'_> {
    LimitationWire {
        affected_identity: &limitation.affected_identity,
        code: &limitation.code,
    }
}

fn provenance_wire(p: &ProvenanceRecord) -> Result<ProvenanceWire<'_>> {
    let toolchain = hex_identity(p.reproduction.toolchain().bytes()).ok_or_else(|| {
        refusal(
            ErrorCode::ResourceIncomplete,
            "the toolchain identity cannot be represented within the result domain",
            Usage {
                string_bytes: usize::MAX,
                ..Usage::default()
            },
            None,
        )
    })?;
    let corpus = hex_identity(p.reproduction.corpus().bytes()).ok_or_else(|| {
        refusal(
            ErrorCode::ResourceIncomplete,
            "the corpus identity cannot be represented within the result domain",
            Usage {
                string_bytes: usize::MAX,
                ..Usage::default()
            },
            None,
        )
    })?;
    Ok(ProvenanceWire {
        backend_identity: &p.backend_identity,
        binding_identity: &p.binding_identity,
        input_identity: &p.input_identity,
        model_identity: &p.model_identity,
        producer_identity: &p.producer_identity,
        profile_identity: &p.profile_identity,
        protocol_identity: &p.protocol_identity,
        reproduction: ReproductionWire {
            corpus_identity: corpus,
            seed: p.reproduction.seed().to_string(),
            toolchain_identity: toolchain,
        },
        request_identity: &p.request_identity,
        source_identity: &p.source_identity,
    })
}

fn hex_identity(bytes: &[u8]) -> Option<String> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let capacity = bytes.len().checked_mul(2)?;
    let mut encoded = String::with_capacity(capacity);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Some(encoded)
}
fn participation_wire(p: &Participation) -> ParticipationWire<'_> {
    match p {
        Participation::Known {
            required_role_instance_refs,
            participating_role_instance_refs,
        } => ParticipationWire::Known {
            participating_role_instance_refs,
            required_role_instance_refs,
        },
        Participation::Unknown {
            participating_role_instance_refs,
            missing_dependency_refs,
            reasons,
        } => ParticipationWire::Unknown {
            missing_dependency_refs,
            participating_role_instance_refs,
            reasons,
        },
    }
}

fn settlement_wire(settlement: &Settlement) -> SettlementWire<'_> {
    SettlementWire {
        basis: settlement.basis.label(),
        supporting_fact_identities: &settlement.supporting_fact_identities,
        supporting_progress_identities: &settlement.supporting_progress_identities,
    }
}
fn completeness_label(state: completeness::State) -> &'static str {
    match state {
        completeness::State::Complete => "complete",
        completeness::State::Incomplete => "incomplete",
        completeness::State::Contradicted => "contradicted",
    }
}

fn semantic_usage(input: &Input<'_>) -> Usage {
    let lineage_nodes = match input
        .decision_support
        .len()
        .checked_add(usize::from(input.correction.is_some()))
    {
        Some(lineage_nodes) => lineage_nodes,
        None => usize::MAX,
    };
    Usage {
        claims: 1,
        premises: input
            .global_conformance
            .premises
            .len()
            .max(input.global_conformance.required_premises.len()),
        support_records: input.decision_support.len(),
        limitations: input.limitations.len(),
        lineage_nodes,
        ..Usage::default()
    }
}
fn raw_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", ByteDigest::of(bytes))
}
fn digest_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use core::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
fn refusal(code: ErrorCode, detail: &'static str, usage: Usage, identity: Option<&str>) -> Refusal {
    Refusal::assembly(code, detail, usage, identity)
}

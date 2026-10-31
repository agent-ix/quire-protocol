// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed FR-240/FR-286 result construction and immutable document storage.

use engineering_assurance::claim_strength::ClaimStrength;
use quire_observation::authority::{closure as observation_closure, completeness, progress};
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};
use serde::Serialize;
use serde_json::Value;
use std::sync::LazyLock;

use crate::assessment::{AxisInputs, GlobalConformanceInput, GlobalPremise};
use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::closure::{AssessmentExecution, GlobalConformanceClosure, ScopeState};
use crate::digest::jcs::{canonical_len, canonicalize, encoded_string_content_len};
use crate::repro::ReproductionRecord;
use crate::ByteDigest;

use super::{global, lineage, reader::StructuralUsage, ErrorCode, Refusal, Result};

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
/// Runtime-derived lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub static SCHEMA_SHA256: LazyLock<String> =
    LazyLock::new(|| format!("{:x}", ByteDigest::of(SCHEMA_BYTES)));

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
    /// The checked predicate held under every admitted evaluation.
    Satisfied,
    /// The checked predicate did not hold under at least one admitted evaluation.
    Violated,
    /// The checked predicate's truth has not yet been settled.
    Pending,
    /// The checked predicate's truth could not be established.
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
    /// Settled because the evaluation scope closed with no further admission possible.
    ClosedScope,
    /// Settled by a decisive witness that establishes the truth directly.
    DecisiveWitness,
    /// Settled by a decisive counterexample that refutes the truth directly.
    DecisiveCounterexample,
    /// Not yet settled by any closed scope, witness, or counterexample.
    Unsettled,
    /// Settlement could not be established.
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
    /// The trigger fired; this result reports on an assessed instance.
    Triggered,
    /// The trigger's guard was admitted and false; no instance was assessed.
    Inactive,
    /// The trigger's guard could not be established.
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
    /// The observation evidence was sufficient to support the reported truth.
    Adequate,
    /// The observation evidence was insufficient to support the reported truth.
    Inadequate,
    /// Whether the observation evidence was sufficient could not be established.
    Unknown,
    /// Observation adequacy was not assessed.
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
    /// The protocol was demonstrated to be adequate for this evaluation.
    Demonstrated,
    /// The protocol was demonstrated to not be adequate for this evaluation.
    NotDemonstrated,
    /// Protocol adequacy does not apply to this evaluation.
    Inapplicable,
    /// Whether the protocol was adequate could not be established.
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
    /// The assumption was established to hold.
    Met,
    /// The assumption was established to not hold.
    NotMet,
    /// Whether the assumption holds could not be established.
    Unknown,
    /// The assumption was not assessed.
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
    /// The identified assumption's exact identity.
    pub identity: String,
    /// Whether the assumption was established to hold.
    pub state: AssumptionState,
}

/// FR-290 protocol claim kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ClaimKind {
    /// A claim about global conformance across the entire evaluated system.
    GlobalConformance,
    /// A claim about whether the property is monitorable at runtime.
    Monitorability,
    /// A claim about a projection restricted to one local participant.
    LocalProjection,
    /// A claim that one specification refines another.
    Refinement,
    /// A claim that the specification is realizable by some implementation.
    Realizability,
    /// A claim about the composition of independently verified components.
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

/// Exact claim, admitted-fragment, and backend selections retained by a result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    /// The kind of protocol claim being made.
    pub kind: ClaimKind,
    /// The strength of evidence backing the claim, when established.
    pub strength: Option<ClaimStrength>,
    /// The exact identity of the admitted fragment the claim covers, when scoped to one.
    pub admitted_fragment_identity: Option<String>,
    /// The exact identity of the verification backend that produced the claim, when known.
    pub backend_identity: Option<String>,
}

/// FR-282 dependence relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DependenceRelation {
    /// This decision-support record is independent of its predecessor.
    Independent,
    /// This decision-support record shares its evidence with its predecessor.
    Shared,
    /// This decision-support record is derived from its predecessor.
    Derived,
    /// The dependence relation to the predecessor could not be established.
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
    /// The exact identity of the referenced oracle.
    pub oracle_identity: String,
    /// The exact identity of the referenced observation.
    pub observation_identity: String,
}

/// FR-248 decision-support record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionSupport {
    /// The exact identity of the oracle backing this record.
    pub oracle_identity: String,
    /// The exact identity of the observation backing this record.
    pub observation_identity: String,
    /// This record's dependence relation to its predecessor.
    pub relation: DependenceRelation,
    /// The predecessor record this one depends on, when one exists.
    pub predecessor: Option<SupportRef>,
    /// The exact identity of the group of records that share this dependence, when grouped.
    pub dependence_group: Option<String>,
}

/// Settlement plus the exact fact/progress identities authorizing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// The basis on which this settlement was reached.
    pub basis: SettlementBasis,
    /// Exact identities of the facts that authorize this settlement.
    pub supporting_fact_identities: Vec<String>,
    /// Exact identities of the progress records that authorize this settlement.
    pub supporting_progress_identities: Vec<String>,
}

/// One typed retained limitation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Limitation {
    /// The stable code identifying this limitation's kind.
    pub code: String,
    /// The exact identity of the item this limitation affects.
    pub affected_identity: String,
}

/// FR-285 closed participation value object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Participation {
    /// Every required and participating role instance is fully established.
    Known {
        /// Exact references to every role instance this evaluation required.
        required_role_instance_refs: Vec<String>,
        /// Exact references to every role instance that actually participated.
        participating_role_instance_refs: Vec<String>,
    },
    /// Participation could not be fully established.
    Unknown {
        /// Exact references to the role instances known to have participated.
        participating_role_instance_refs: Vec<String>,
        /// Exact references to dependencies missing from the evaluation.
        missing_dependency_refs: Vec<String>,
        /// Reasons participation could not be fully established.
        reasons: Vec<String>,
    },
}

/// Complete provenance identity set retained independently of display text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    /// The exact identity of the source that produced this result.
    pub source_identity: String,
    /// The exact identity of the profile the evaluation was run under.
    pub profile_identity: String,
    /// The exact identity of the model that produced the evaluated behavior.
    pub model_identity: String,
    /// The exact identity of the protocol this result is bound to.
    pub protocol_identity: String,
    /// The exact identity of the request that produced this result.
    pub request_identity: String,
    /// The exact identity of the binding under which the request was evaluated.
    pub binding_identity: String,
    /// The exact identity of the input consumed to produce this result.
    pub input_identity: String,
    /// The exact identity of the backend that performed the evaluation.
    pub backend_identity: String,
    /// The exact identity of the producer that emitted this result.
    pub producer_identity: String,
    /// Version of the protocol producer that emits the result key.
    pub producer_version: String,
    /// Exact local-run inputs needed to reproduce this result.
    pub reproduction: ReproductionRecord,
}

/// Structural FR-286 result identity. It is never computed from result bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultKey {
    /// Identity and version of the protocol producer.
    pub producer: ResultProducerKey,
    /// Exact request identity answered by the result.
    pub request: String,
    /// Producer-assigned decimal U64String position for this request.
    pub ordinal: String,
}

/// Producer component of a structural result key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultProducerKey {
    /// Exact producer identity.
    pub identity: String,
    /// Exact producer version.
    pub version: String,
}

/// Explicit correction relation to the direct predecessor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectionRelation {
    /// This result supersedes its predecessor with an updated finding.
    Supersedes,
    /// This result invalidates its predecessor outright.
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
    /// How this result relates to its predecessor.
    pub relation: CorrectionRelation,
    /// The already-validated predecessor this result corrects.
    pub predecessor: &'a ValidatedResult,
    /// The exact identity of the premise the predecessor's finding contradicted.
    pub contradicted_premise_identity: &'a str,
    /// The exact identity of the input that was corrected.
    pub corrected_input_identity: &'a str,
}

/// Complete typed input to canonical result production.
#[derive(Clone, Debug)]
pub struct Input<'a> {
    /// The validated checked predicate this result reports on.
    pub checked_predicate: &'a checked_predicate::ValidatedCheckedPredicate,
    /// The validated temporal subject this result reports on.
    pub temporal_subject: &'a temporal_subject::ValidatedTemporalSubject,
    /// The complete set of validated axis inputs for this result.
    pub axes: AxisInputs<'a>,
    /// The observation-authority completeness view for this evaluation.
    pub completeness: &'a completeness::View,
    /// The assessment execution this result was produced under.
    pub execution: AssessmentExecution,
    /// The checked predicate's truth disposition.
    pub truth: Truth,
    /// The settlement basis and its authorizing identities.
    pub settlement: Settlement,
    /// The activation state of the evaluated trigger.
    pub activation: Activation,
    /// The closed participation value object for this evaluation.
    pub participation: Participation,
    /// Every independently identified assumption and its state.
    pub assumptions: Vec<Assumption>,
    /// Whether the observation evidence was adequate to support the reported truth.
    pub observation_adequacy: Adequacy,
    /// Whether the protocol was adequate for this evaluation.
    pub protocol_adequacy: ProtocolAdequacy,
    /// The protocol claim, admitted-fragment, and backend selections for this result.
    pub claim: Claim,
    /// Every decision-support record backing this result.
    pub decision_support: Vec<DecisionSupport>,
    /// The global-conformance input this result was assessed against.
    pub global_conformance: GlobalConformanceInput,
    /// Every typed limitation retained for this result.
    pub limitations: Vec<Limitation>,
    /// The complete provenance identity set for this result.
    pub provenance: ProvenanceRecord,
    /// Producer-assigned position in this request's result stream (FR-286 U64String).
    pub result_ordinal: u64,
    /// The direct correction this result makes to a predecessor, when it corrects one.
    pub correction: Option<CorrectionInput<'a>>,
}

/// Immutable canonical result bytes plus identities outside those bytes.
#[derive(Clone, Debug)]
pub struct Document {
    bytes: Vec<u8>,
    result_key: ResultKey,
    pub(crate) usage: Usage,
    pub(crate) snapshot: Snapshot,
}

impl Document {
    /// Complete canonical result bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Structural FR-286 result key.
    #[must_use]
    pub fn result_key(&self) -> &ResultKey {
        &self.result_key
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
    /// Structural FR-286 result key.
    #[must_use]
    pub fn result_key(&self) -> &ResultKey {
        self.document.result_key()
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
    pub(crate) predecessor_result_key: ResultKey,
    pub(crate) contradicted_premise_identity: String,
    pub(crate) corrected_input_identity: String,
}

impl CorrectionView {
    /// Correction relation.
    #[must_use]
    pub const fn relation(&self) -> CorrectionRelation {
        self.relation
    }
    /// Direct predecessor FR-286 result key.
    #[must_use]
    pub fn predecessor_result_key(&self) -> &ResultKey {
        &self.predecessor_result_key
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
    pub result_key: ResultKey,
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
    semantic_usage: Usage,
}

/// Produce one canonical FR-286 result or one typed refusal with no partial output.
pub(crate) fn produce(input: Input<'_>, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    let prepared = prepare_without_identity(input, effective)?;
    finish(prepared, effective)
}

/// Produce one canonical result under the shared QProtocol caller budget.
///
/// Exact string, depth and field requirements are charged before the governed
/// snapshot/value allocation. The exact final canonical-byte requirement is
/// then computed without allocating canonical text and charged before either
/// identity-preimage or final-output canonical bytes are allocated.
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
    ];
    if let Err(error) = resources.charge_all_mixed(&charges) {
        return OperationOutcome::from_charge(error);
    }
    // Preflight charges every structural unit before traversing it. Only after
    // all caller-controlled shape and text has been admitted may the retained
    // snapshot and typed JSON value be allocated.
    if let Err(error) = charge_output_preflight(&input, effective, resources) {
        return OperationOutcome::from_charge(error);
    }
    let prepared = match prepare_without_identity(input, effective) {
        Ok(prepared) => prepared,
        Err(error) => return OperationOutcome::DomainError(error),
    };
    let required = match canonical_output_len(&prepared.value) {
        Ok(required) => required,
        Err(_) => {
            return OperationOutcome::DomainError(refusal(
                ErrorCode::InvalidSelection,
                "result cannot determine its canonical output length",
                Usage::default(),
                None,
            ));
        }
    };
    if let Err(error) =
        resources.charge_with_limit(OUTPUT_SOURCE.id(), required, effective.max_output_bytes)
    {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(finish(prepared, effective))
}

fn prepare_without_identity(input: Input<'_>, effective: Limits) -> Result<Prepared> {
    validate_input(&input, effective)?;
    let semantic_usage = semantic_usage(&input);
    let snapshot = snapshot(&input);
    let value = result_value(&snapshot)?;
    Ok(Prepared {
        snapshot,
        value,
        semantic_usage,
    })
}

fn canonical_output_len(value: &Value) -> std::result::Result<usize, crate::digest::jcs::JcsError> {
    canonical_len(value)
}

fn finish(prepared: Prepared, effective: Limits) -> Result<Document> {
    let canonical = canonicalize(&prepared.value).map_err(|_| {
        refusal(
            ErrorCode::InvalidSelection,
            "result contains an unsupported JCS number",
            Usage::default(),
            Some(&prepared.snapshot.result_key.request),
        )
    })?;
    finish_canonical(prepared, canonical, effective)
}

fn finish_canonical(prepared: Prepared, canonical: String, effective: Limits) -> Result<Document> {
    let Prepared {
        snapshot,
        semantic_usage,
        ..
    } = prepared;
    let bytes = canonical.into_bytes();
    let usage = super::reader::preflight_output(&bytes, effective, semantic_usage)?;
    Ok(Document {
        bytes,
        result_key: snapshot.result_key.clone(),
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
            .max(decimal_len(provenance.reproduction.seed().value())),
    )
}

fn decimal_len(mut value: u64) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn base_output_fields(input: &Input<'_>) -> Option<usize> {
    fn add(total: &mut usize, value: usize) -> Option<()> {
        *total = total.checked_add(value)?;
        Some(())
    }

    fn add_records(total: &mut usize, count: usize, width: usize) -> Option<()> {
        add(total, count.checked_mul(width)?)
    }

    // The final result always contains 24 root members. Its structural
    // `resultKey` adds three key members and two nested producer members.
    // There are four ten-member axis
    // records, the four-member claim, the eight-member completeness record,
    // the two-member global record, the thirteen provenance/reproduction
    // members and the three-member settlement record.
    let mut fields = 24usize;
    add(&mut fields, 5)?;
    add_records(&mut fields, input.assumptions.len(), 3)?;
    add(&mut fields, 4 + 8)?;
    add(&mut fields, input.completeness.payload().fact_count())?;
    if input.correction.is_some() {
        add(&mut fields, 5)?;
    }
    add(&mut fields, 40)?;
    add_records(&mut fields, input.decision_support.len(), 6)?;
    add(&mut fields, 2)?;
    add_records(&mut fields, input.global_conformance.premises.len(), 5)?;
    add_records(&mut fields, input.limitations.len(), 3)?;
    match &input.participation {
        Participation::Known {
            required_role_instance_refs,
            participating_role_instance_refs,
        } => {
            add(
                &mut fields,
                3usize
                    .checked_add(required_role_instance_refs.len())?
                    .checked_add(participating_role_instance_refs.len())?,
            )?;
        }
        Participation::Unknown {
            participating_role_instance_refs,
            missing_dependency_refs,
            reasons,
        } => {
            add(
                &mut fields,
                4usize
                    .checked_add(participating_role_instance_refs.len())?
                    .checked_add(missing_dependency_refs.len())?
                    .checked_add(reasons.len())?,
            )?;
        }
    }
    add(&mut fields, 13 + 3)?;
    add(
        &mut fields,
        input
            .settlement
            .supporting_fact_identities
            .len()
            .checked_add(input.settlement.supporting_progress_identities.len())?,
    )?;

    Some(fields)
}

fn charge_output_preflight(
    input: &Input<'_>,
    effective: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> core::result::Result<StructuralUsage, ChargeError> {
    // Three container levels are unconditional. A global premise makes the
    // fourth level visible without traversal; a nested support predecessor
    // charges that fourth level immediately before it is entered below.
    let mut depth = if input.global_conformance.premises.is_empty() {
        3
    } else {
        4
    };
    resources.charge_with_limit(DEPTH_SOURCE.id(), depth, effective.max_depth)?;

    // The base field count uses only O(1) collection lengths and variant
    // discriminants. It deliberately excludes the two predecessor members,
    // which are charged immediately before each optional nested record.
    let mut fields =
        base_output_fields(input).unwrap_or_else(|| effective.max_visited_fields.saturating_add(1));
    resources.charge_with_limit(OPERATION_SOURCE.id(), fields, effective.max_visited_fields)?;
    for support in &input.decision_support {
        if support.predecessor.is_some() {
            if depth == 3 {
                resources.charge_with_limit(DEPTH_SOURCE.id(), 1, effective.max_depth)?;
                depth = 4;
            }
            resources.charge_with_limit(OPERATION_SOURCE.id(), 2, effective.max_visited_fields)?;
            fields = fields.saturating_add(2);
        }
    }

    // Reserve the largest FR-286 key member before traversing caller strings.
    // Each variable string then reserves its O(1) raw byte length before
    // scanning escapes, and reserves any escape expansion before the retained
    // snapshot is built.
    let mut string_bytes = 0usize;
    let result_key_string = input
        .provenance
        .producer_identity
        .len()
        .max(input.provenance.producer_version.len())
        .max(input.provenance.request_identity.len())
        .max(decimal_len(input.result_ordinal));
    charge_string_size(
        resources,
        effective.max_string_bytes,
        &mut string_bytes,
        result_key_string,
    )?;
    let reproduction_string = largest_reproduction_string(input)
        .unwrap_or_else(|| effective.max_string_bytes.saturating_add(1));
    charge_string_size(
        resources,
        effective.max_string_bytes,
        &mut string_bytes,
        reproduction_string,
    )?;
    let mut observe = |value: &str| {
        charge_encoded_string(
            resources,
            effective.max_string_bytes,
            &mut string_bytes,
            value,
        )
    };
    observe(input.checked_predicate.document().identity())?;
    observe(input.temporal_subject.document().identity())?;
    for assumption in &input.assumptions {
        observe(&assumption.identity)?;
    }
    if let Some(identity) = input.claim.admitted_fragment_identity.as_deref() {
        observe(identity)?;
    }
    if let Some(identity) = input.claim.backend_identity.as_deref() {
        observe(identity)?;
    }
    for view in [
        input.axes.decision_scope_progress,
        input.axes.surrounding_execution_progress,
    ] {
        observe(view.identity().as_str())?;
        observe(view.authority().definition_identity.as_str())?;
        observe(view.authority().definition_revision.as_str())?;
        observe(view.subject().scope_identity.as_str())?;
        observe(view.subject().population_identity.as_str())?;
    }
    for view in [
        input.axes.decision_scope_closure,
        input.axes.surrounding_execution_closure,
    ] {
        observe(view.identity().as_str())?;
        observe(view.authority().definition_identity.as_str())?;
        observe(view.authority().definition_revision.as_str())?;
        observe(view.subject().scope_identity.as_str())?;
        observe(view.subject().population_identity.as_str())?;
    }
    observe(input.completeness.identity().as_str())?;
    observe(input.completeness.payload().population_identity())?;
    observe(input.completeness.payload().boundary_identity())?;
    for fact in input.completeness.payload().facts() {
        observe(fact.observation_identity.unwrap_or(fact.member_identity))?;
    }
    for support in &input.decision_support {
        observe(&support.oracle_identity)?;
        observe(&support.observation_identity)?;
        if let Some(predecessor) = &support.predecessor {
            observe(&predecessor.oracle_identity)?;
            observe(&predecessor.observation_identity)?;
        }
        if let Some(group) = support.dependence_group.as_deref() {
            observe(group)?;
        }
    }
    for premise in &input.global_conformance.premises {
        observe(&premise.identity)?;
        observe(&premise.scope_identity)?;
        observe(&premise.authority_identity)?;
    }
    for limitation in &input.limitations {
        observe(&limitation.code)?;
        observe(&limitation.affected_identity)?;
    }
    match &input.participation {
        Participation::Known {
            required_role_instance_refs,
            participating_role_instance_refs,
        } => {
            for value in required_role_instance_refs
                .iter()
                .chain(participating_role_instance_refs)
            {
                observe(value)?;
            }
        }
        Participation::Unknown {
            participating_role_instance_refs,
            missing_dependency_refs,
            reasons,
        } => {
            for value in participating_role_instance_refs
                .iter()
                .chain(missing_dependency_refs)
                .chain(reasons)
            {
                observe(value)?;
            }
        }
    }
    for value in [
        input.provenance.source_identity.as_str(),
        input.provenance.profile_identity.as_str(),
        input.provenance.model_identity.as_str(),
        input.provenance.protocol_identity.as_str(),
        input.provenance.request_identity.as_str(),
        input.provenance.binding_identity.as_str(),
        input.provenance.input_identity.as_str(),
        input.provenance.backend_identity.as_str(),
        input.provenance.producer_identity.as_str(),
        input.provenance.producer_version.as_str(),
    ] {
        observe(value)?;
    }
    for value in input
        .settlement
        .supporting_fact_identities
        .iter()
        .chain(&input.settlement.supporting_progress_identities)
    {
        observe(value)?;
    }
    if let Some(correction) = input.correction {
        observe(&correction.predecessor.result_key().request)?;
        observe(correction.contradicted_premise_identity)?;
        observe(correction.corrected_input_identity)?;
    }

    Ok(StructuralUsage {
        fields,
        depth,
        string_bytes,
    })
}

fn charge_string_size(
    resources: &mut ResourceAccountant<'_>,
    limit: usize,
    largest: &mut usize,
    size: usize,
) -> core::result::Result<(), ChargeError> {
    if size > *largest {
        resources.charge_with_limit(STRING_SOURCE.id(), size - *largest, limit)?;
        *largest = size;
    }
    Ok(())
}

fn charge_encoded_string(
    resources: &mut ResourceAccountant<'_>,
    limit: usize,
    largest: &mut usize,
    value: &str,
) -> core::result::Result<(), ChargeError> {
    // `str::len` is O(1), so the raw bytes are reserved before the escaping
    // scan. JCS never shortens a UTF-8 string; only the expansion delta can
    // remain after this reservation, and it is charged before retention.
    charge_string_size(resources, limit, largest, value.len())?;
    let encoded = match encoded_string_content_len(value) {
        Ok(encoded) => encoded,
        Err(_) => return force_string_refusal(resources, limit, *largest),
    };
    charge_string_size(resources, limit, largest, encoded)
}

fn force_string_refusal(
    resources: &mut ResourceAccountant<'_>,
    limit: usize,
    consumed: usize,
) -> core::result::Result<(), ChargeError> {
    resources.charge_with_limit(
        STRING_SOURCE.id(),
        limit.saturating_sub(consumed).saturating_add(1),
        limit,
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
        p.producer_version.as_str(),
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
        result_key: ResultKey {
            producer: ResultProducerKey {
                identity: input.provenance.producer_identity.clone(),
                version: input.provenance.producer_version.clone(),
            },
            request: input.provenance.request_identity.clone(),
            ordinal: input.result_ordinal.to_string(),
        },
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
            predecessor_result_key: value.predecessor.result_key().clone(),
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
    result_key: &'a ResultKey,
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
    predecessor_result_key: &'a ResultKey,
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

fn result_value(snapshot: &Snapshot) -> Result<Value> {
    let axes = &snapshot.axes;
    let wire = ResultWire {
        activation: snapshot.activation.label(),
        assumptions: snapshot.assumptions.iter().map(assumption_wire).collect(),
        claim: claim_wire(&snapshot.claim),
        completeness: completeness_wire(&snapshot.completeness),
        contract_version: CONTRACT,
        correction: snapshot.correction.as_ref().map(correction_wire),
        decision_scope_closure: axis_wire(&axes[1]),
        decision_scope_progress: axis_wire(&axes[0]),
        decision_support: snapshot.decision_support.iter().map(support_wire).collect(),
        execution: snapshot.execution.label(),
        global_conformance: GlobalWire {
            premises: snapshot.global_premises.iter().map(premise_wire).collect(),
            state: snapshot.global_conformance_state.label(),
        },
        kind: "ProtocolResult",
        limitations: snapshot.limitations.iter().map(limitation_wire).collect(),
        native_correspondence_identity: &snapshot.native_correspondence_identity,
        native_subject_identity: &snapshot.native_subject_identity,
        observation_adequacy: snapshot.observation_adequacy.label(),
        participation: participation_wire(&snapshot.participation),
        protocol_adequacy: snapshot.protocol_adequacy.label(),
        provenance: provenance_wire(&snapshot.provenance)?,
        result_key: &snapshot.result_key,
        settlement: settlement_wire(&snapshot.settlement),
        surrounding_execution_closure: axis_wire(&axes[3]),
        surrounding_execution_progress: axis_wire(&axes[2]),
        truth: snapshot.truth.label(),
    };
    serde_json::to_value(wire).map_err(|_| {
        refusal(
            ErrorCode::InvalidSelection,
            "typed result record cannot be represented as JSON",
            Usage::default(),
            Some(&snapshot.result_key.request),
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
        strength: claim.strength.map(ClaimStrength::wire_name),
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
        predecessor_result_key: &correction.predecessor_result_key,
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

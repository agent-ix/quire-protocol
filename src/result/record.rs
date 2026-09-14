// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed FR-240/FR-286 result construction and immutable document storage.

use quire_observation::authority::{closure as observation_closure, completeness, progress};
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};
use serde_json::{json, Value};

use crate::assessment::{AxisInputs, GlobalConformanceInput, GlobalPremise};
use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind};
use crate::closure::{AssessmentExecution, ScopeState};
use crate::digest::jcs::{canonical_len, canonicalize, ResultDigest};
use crate::ByteDigest;

use super::{global, lineage, ErrorCode, Refusal, Result};

pub(crate) const ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/item/retain"),
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

/// Canonical result contract label.
pub const CONTRACT: &str = "quire.protocol.result/v1-draft.1";
/// Immutable schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/protocol-result-v1-draft-1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "2623a83afcb30a9c5307922d8a0f169ad2509ca477590e527b06172d6f5a3fa9";
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
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
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
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.document.bytes()
    }
    #[must_use]
    pub fn result_id(&self) -> &str {
        self.document.result_id()
    }
    #[must_use]
    pub fn digest(&self) -> &str {
        self.document.digest()
    }
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.document.usage()
    }
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        self.document.snapshot.execution
    }
    #[must_use]
    pub const fn truth(&self) -> Truth {
        self.document.snapshot.truth
    }
    #[must_use]
    pub const fn settlement_basis(&self) -> SettlementBasis {
        self.document.snapshot.settlement_basis
    }
    #[must_use]
    pub const fn completeness_state(&self) -> completeness::State {
        self.document.snapshot.completeness_state
    }
    #[must_use]
    pub fn native_subject_identity(&self) -> &str {
        &self.document.snapshot.native_subject_identity
    }
    #[must_use]
    pub fn native_correspondence_identity(&self) -> &str {
        &self.document.snapshot.native_correspondence_identity
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OwnerAxis {
    pub contract: &'static str,
    pub schema_sha256: &'static str,
    pub document_identity: String,
    pub document_digest: String,
    pub authority_identity: String,
    pub authority_revision: String,
    pub authority_digest: String,
    pub scope_identity: String,
    pub population_identity: String,
    pub state: ScopeState,
}

#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub native_subject_identity: String,
    pub native_correspondence_identity: String,
    pub input_identity: String,
    pub execution: AssessmentExecution,
    pub truth: Truth,
    pub settlement_basis: SettlementBasis,
    pub settlement: Settlement,
    pub completeness_state: completeness::State,
    pub completeness_identity: String,
    pub completeness_digest: String,
    pub completeness_population_identity: String,
    pub completeness_boundary_identity: String,
    pub completeness_fact_identities: Vec<String>,
    pub axes: [OwnerAxis; 4],
    pub decision_support: Vec<DecisionSupport>,
    pub global_premises: Vec<GlobalPremise>,
    pub correction: Option<CorrectionSnapshot>,
}

#[derive(Clone, Debug)]
pub(crate) struct CorrectionSnapshot {
    pub relation: CorrectionRelation,
    pub predecessor_result_id: String,
    pub predecessor_digest: String,
    pub contradicted_premise_identity: String,
    pub corrected_input_identity: String,
}

struct Prepared {
    snapshot: Snapshot,
    value: Value,
    result_id: String,
    semantic_usage: Usage,
}

/// Produce one canonical FR-286 result or one typed refusal with no partial output.
pub fn produce(input: Input<'_>, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    finish(prepare(input, effective)?, effective)
}

/// Produce one canonical result under the shared QProtocol caller budget.
///
/// The exact canonical byte length is computed without an output buffer, then
/// checked against both owner and caller ceilings before canonical allocation.
pub fn produce_bounded(
    input: Input<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Document, Refusal> {
    let effective = limits.effective();
    let prepared = match prepare(input, effective) {
        Ok(prepared) => prepared,
        Err(error) => return OperationOutcome::DomainError(error),
    };
    let required = match canonical_len(&prepared.value) {
        Ok(required) => required,
        Err(_) => {
            return OperationOutcome::DomainError(refusal(
                ErrorCode::InvalidSelection,
                "result cannot determine its canonical output length",
                Usage::default(),
                Some(&prepared.result_id),
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

fn prepare(input: Input<'_>, effective: Limits) -> Result<Prepared> {
    validate_input(&input, effective)?;
    let semantic_usage = semantic_usage(&input);
    let snapshot = snapshot(&input);
    let mut value = result_value(&input, &snapshot, None);
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
    value = result_value(&input, &snapshot, Some(&result_id));
    Ok(Prepared {
        snapshot,
        value,
        result_id,
        semantic_usage,
    })
}

fn finish(prepared: Prepared, effective: Limits) -> Result<Document> {
    let Prepared {
        snapshot,
        value,
        result_id,
        semantic_usage,
    } = prepared;
    let canonical = canonicalize(&value).map_err(|_| {
        refusal(
            ErrorCode::InvalidSelection,
            "result contains an unsupported JCS number",
            Usage::default(),
            Some(&result_id),
        )
    })?;
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
    validate_nonempty(input)?;
    global::validate(&input.claim, &input.global_conformance, limits)?;
    lineage::validate_support(&input.decision_support, limits)?;
    lineage::validate_correction(input.correction, limits)?;
    validate_axes(input)?;
    validate_combination(input)?;
    validate_unique_strings(input)?;
    Ok(())
}

fn validate_nonempty(input: &Input<'_>) -> Result<()> {
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
    let valid = match (
        input.execution,
        input.truth,
        input.settlement.basis,
        input.activation,
    ) {
        (E::Completed, T::Satisfied, S::ClosedScope, Activation::Triggered)
        | (E::Completed, T::Satisfied, S::DecisiveWitness, Activation::Triggered)
        | (E::Completed, T::Violated, S::ClosedScope, Activation::Triggered)
        | (E::Completed, T::Violated, S::DecisiveCounterexample, Activation::Triggered) => {
            !input.decision_support.is_empty()
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
        input_identity: input.provenance.input_identity.clone(),
        execution: input.execution,
        truth: input.truth,
        settlement_basis: input.settlement.basis,
        settlement: input.settlement.clone(),
        completeness_state: input.completeness.payload().state(),
        completeness_identity: input.completeness.identity().as_str().to_owned(),
        completeness_digest: raw_digest(input.completeness.bytes()),
        completeness_population_identity: input
            .completeness
            .payload()
            .population_identity()
            .to_owned(),
        completeness_boundary_identity: input.completeness.payload().boundary_identity().to_owned(),
        completeness_fact_identities: completeness_facts,
        axes,
        decision_support: input.decision_support.clone(),
        global_premises: input.global_conformance.premises.clone(),
        correction: input.correction.map(|value| CorrectionSnapshot {
            relation: value.relation,
            predecessor_result_id: value.predecessor.result_id().to_owned(),
            predecessor_digest: value.predecessor.digest().to_owned(),
            contradicted_premise_identity: value.contradicted_premise_identity.to_owned(),
            corrected_input_identity: value.corrected_input_identity.to_owned(),
        }),
    }
}

fn progress_axis(view: &progress::View) -> OwnerAxis {
    OwnerAxis {
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

fn closure_axis(view: &observation_closure::View) -> OwnerAxis {
    OwnerAxis {
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

fn result_value(input: &Input<'_>, snapshot: &Snapshot, result_id: Option<&str>) -> Value {
    let axes = &snapshot.axes;
    let mut value = json!({
        "activation": input.activation.label(),
        "assumptions": input.assumptions.iter().map(|a| json!({"identity": a.identity, "state": a.state.label()})).collect::<Vec<_>>(),
        "claim": claim_value(&input.claim),
        "completeness": {"boundaryIdentity": snapshot.completeness_boundary_identity, "contract": completeness::CONTRACT, "documentDigest": snapshot.completeness_digest, "documentIdentity": snapshot.completeness_identity, "factIdentities": snapshot.completeness_fact_identities, "populationIdentity": snapshot.completeness_population_identity, "schemaSha256": completeness::SCHEMA_SHA256, "state": completeness_label(snapshot.completeness_state)},
        "contractVersion": CONTRACT,
        "correction": snapshot.correction.as_ref().map(correction_value),
        "decisionScopeClosure": axis_value(&axes[1]),
        "decisionScopeProgress": axis_value(&axes[0]),
        "decisionSupport": input.decision_support.iter().map(support_value).collect::<Vec<_>>(),
        "execution": input.execution.label(),
        "globalConformance": {"premises": input.global_conformance.premises.iter().map(premise_value).collect::<Vec<_>>(), "state": input.global_conformance.state.label()},
        "kind": "ProtocolResult",
        "limitations": input.limitations.iter().map(|l| json!({"affectedIdentity": l.affected_identity, "code": l.code})).collect::<Vec<_>>(),
        "nativeCorrespondenceIdentity": snapshot.native_correspondence_identity,
        "nativeSubjectIdentity": snapshot.native_subject_identity,
        "observationAdequacy": input.observation_adequacy.label(),
        "participation": participation_value(&input.participation),
        "protocolAdequacy": input.protocol_adequacy.label(),
        "provenance": provenance_value(&input.provenance),
        "resultId": result_id,
        "settlement": {"basis": input.settlement.basis.label(), "supportingFactIdentities": input.settlement.supporting_fact_identities, "supportingProgressIdentities": input.settlement.supporting_progress_identities},
        "surroundingExecutionClosure": axis_value(&axes[3]),
        "surroundingExecutionProgress": axis_value(&axes[2]),
        "truth": input.truth.label()
    });
    if result_id.is_none() {
        if let Some(object) = value.as_object_mut() {
            object.remove("resultId");
        }
    }
    value
}

fn axis_value(axis: &OwnerAxis) -> Value {
    json!({"authorityDigest": axis.authority_digest, "authorityIdentity": axis.authority_identity, "authorityRevision": axis.authority_revision, "contract": axis.contract, "documentDigest": axis.document_digest, "documentIdentity": axis.document_identity, "populationIdentity": axis.population_identity, "schemaSha256": axis.schema_sha256, "scopeIdentity": axis.scope_identity, "state": axis.state.label()})
}
fn claim_value(claim: &Claim) -> Value {
    json!({"admittedFragmentIdentity": claim.admitted_fragment_identity, "backendIdentity": claim.backend_identity, "kind": claim.kind.label(), "strength": claim.strength.map(ClaimStrength::label)})
}
fn support_value(s: &DecisionSupport) -> Value {
    json!({"dependenceGroup": s.dependence_group, "observationIdentity": s.observation_identity, "oracleIdentity": s.oracle_identity, "predecessor": s.predecessor.as_ref().map(|p| json!({"observationIdentity": p.observation_identity, "oracleIdentity": p.oracle_identity})), "relation": s.relation.label()})
}
fn premise_value(p: &GlobalPremise) -> Value {
    json!({"authorityIdentity": p.authority_identity, "identity": p.identity, "kind": p.kind.label(), "scopeIdentity": p.scope_identity})
}
fn correction_value(c: &CorrectionSnapshot) -> Value {
    json!({"contradictedPremiseIdentity": c.contradicted_premise_identity, "correctedInputIdentity": c.corrected_input_identity, "predecessorDigest": c.predecessor_digest, "predecessorResultId": c.predecessor_result_id, "relation": c.relation.label()})
}
fn provenance_value(p: &ProvenanceRecord) -> Value {
    json!({"backendIdentity": p.backend_identity, "bindingIdentity": p.binding_identity, "inputIdentity": p.input_identity, "modelIdentity": p.model_identity, "producerIdentity": p.producer_identity, "profileIdentity": p.profile_identity, "protocolIdentity": p.protocol_identity, "requestIdentity": p.request_identity, "sourceIdentity": p.source_identity})
}
fn participation_value(p: &Participation) -> Value {
    match p {
        Participation::Known {
            required_role_instance_refs,
            participating_role_instance_refs,
        } => {
            json!({"kind":"known", "participatingRoleInstanceRefs": participating_role_instance_refs, "requiredRoleInstanceRefs": required_role_instance_refs})
        }
        Participation::Unknown {
            participating_role_instance_refs,
            missing_dependency_refs,
            reasons,
        } => {
            json!({"kind":"unknown", "missingDependencyRefs": missing_dependency_refs, "participatingRoleInstanceRefs": participating_role_instance_refs, "reasons": reasons})
        }
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
    Usage {
        claims: 1,
        premises: input
            .global_conformance
            .premises
            .len()
            .max(input.global_conformance.required_premises.len()),
        support_records: input.decision_support.len(),
        limitations: input.limitations.len(),
        lineage_nodes: input
            .decision_support
            .len()
            .saturating_add(usize::from(input.correction.is_some())),
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
    Refusal::new(code, detail, usage, identity)
}

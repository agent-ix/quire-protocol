// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact, read-only Contract-IR result mapping derived from a validated result.

use serde::Serialize;
use serde_json::Value;
use std::sync::OnceLock;

use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::closure::AssessmentExecution;
use crate::digest::jcs::{canonical_len, canonicalize};
use crate::refusal::catalog::{CodeEntry, CodeModule, Provenance};
use crate::refusal::{RefusalDomain, RetryDecision};
use crate::ByteDigest;

use super::record::{
    AxisView, CorrectionView, DecisionSupport, Limits, Settlement, SettlementBasis, Snapshot,
    Truth, Usage, ValidatedResult, CONTRACT as RESULT_CONTRACT,
    SCHEMA_SHA256 as RESULT_SCHEMA_SHA256,
};
use super::{ErrorCode, Refusal, Result};

pub(crate) const DERIVED_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/derive"),
    "result::contract_ir::map",
    BudgetDimension::DerivedArtifact,
    SiteKind::Retention,
);
pub(crate) const ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/claim/retain"),
    "result::contract_ir::map",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const SUPPORT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/support/retain"),
    "result::contract_ir::map",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const LINEAGE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/lineage/traverse"),
    "result::contract_ir::map",
    BudgetDimension::Lineage,
    SiteKind::Traversal,
);
pub(crate) const OUTPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/output/allocate"),
    "result::contract_ir::map",
    BudgetDimension::OutputByte,
    SiteKind::Allocation,
);
pub(crate) const INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/input/decode"),
    "result::contract_ir::read",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
pub(crate) const STRING_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/string/retain"),
    "result::contract_ir::read",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
pub(crate) const DEPTH_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/container/depth"),
    "result::contract_ir::read",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/field/visit"),
    "result::contract_ir::read",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/view/allocate"),
    "result::contract_ir::map",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
pub(crate) const READ_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/input/allocate"),
    "result::contract_ir::read",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

const READ_SOURCES: super::reader::StructuralSources = super::reader::StructuralSources::new(
    INPUT_SOURCE,
    STRING_SOURCE,
    DEPTH_SOURCE,
    OPERATION_SOURCE,
);

/// Selected mapping contract.
pub const CONTRACT: &str = "quire.protocol.contract-ir-result-map/v1";
/// Immutable schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/contract-ir-result-map-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "c1df7b18d0e93c70a3b1d67c6a71de8d5ff8b9780510265ebdad69cdd14b79bd";

static MAPPING_SCHEMA_VALIDATOR: OnceLock<Option<jsonschema::JSONSchema>> = OnceLock::new();

pub(crate) const MAPPING_MISMATCH: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "mapping.result-mismatch",
    domain: RefusalDomain::Projection,
    retry: RetryDecision::Never,
    summary: "the mapping differs from the source-derived view",
};

/// Contract-IR mapping codes registered with the crate-wide refusal catalog.
pub const MAPPING_CODES: CodeModule = CodeModule {
    root: "mapping",
    owner: "C-28",
    module_path: "quire_protocol::result::contract_ir",
    entries: &[MAPPING_MISMATCH],
};

/// Exact mapping and source-contract selections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MappingSelection<'a> {
    mapping_contract: &'a str,
    mapping_schema_sha256: &'a str,
    source_contract: &'a str,
    source_schema_sha256: &'a str,
}

impl<'a> MappingSelection<'a> {
    /// Construct an explicit mapping selection; [`map`] checks every member.
    #[must_use]
    pub const fn new(
        mapping_contract: &'a str,
        mapping_schema_sha256: &'a str,
        source_contract: &'a str,
        source_schema_sha256: &'a str,
    ) -> Self {
        Self {
            mapping_contract,
            mapping_schema_sha256,
            source_contract,
            source_schema_sha256,
        }
    }

    /// Exact currently published selection.
    #[must_use]
    pub const fn current() -> Self {
        Self::new(
            CONTRACT,
            SCHEMA_SHA256,
            RESULT_CONTRACT,
            RESULT_SCHEMA_SHA256,
        )
    }
}

/// Complete source facts retained for a non-Boolean mapping row.
#[derive(Clone, Copy, Debug)]
pub struct NonValue<'a> {
    source: &'a ValidatedResult,
}

impl NonValue<'_> {
    /// Whether the source assessment executed.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        self.source.execution()
    }
    /// Native truth disposition retained without Boolean coercion.
    #[must_use]
    pub const fn truth(&self) -> Truth {
        self.source.truth()
    }
    /// Exact source settlement basis.
    #[must_use]
    pub const fn settlement(&self) -> SettlementBasis {
        self.source.settlement_basis()
    }
    /// Exact source completeness state.
    #[must_use]
    pub const fn completeness(&self) -> quire_observation::authority::completeness::State {
        self.source.completeness_state()
    }
}

/// One progress or closure assertion in the selected mapping view.
#[derive(Clone, Copy, Debug)]
pub struct MappedAxisView<'a> {
    source: &'a AxisView,
}

impl MappedAxisView<'_> {
    /// Definition authority identity.
    #[must_use]
    pub fn authority_identity(&self) -> &str {
        self.source.authority_identity()
    }
    /// Definition authority revision.
    #[must_use]
    pub fn authority_revision(&self) -> &str {
        self.source.authority_revision()
    }
    /// Digest of the canonical owner assertion bytes.
    #[must_use]
    pub fn document_digest(&self) -> &str {
        self.source.document_digest()
    }
    /// Canonical owner assertion identity.
    #[must_use]
    pub fn document_identity(&self) -> &str {
        self.source.document_identity()
    }
    /// Exact scope identity.
    #[must_use]
    pub fn scope_identity(&self) -> &str {
        self.source.scope_identity()
    }
    /// Independent open/closed state.
    #[must_use]
    pub const fn state(&self) -> crate::closure::ScopeState {
        self.source.state()
    }
}

/// Completeness assertion facts retained by the selected mapping view.
#[derive(Clone, Copy, Debug)]
pub struct MappedCompletenessView<'a> {
    source: &'a super::record::CompletenessView,
}

impl MappedCompletenessView<'_> {
    /// Exact temporal boundary identity.
    #[must_use]
    pub fn boundary_identity(&self) -> &str {
        self.source.boundary_identity()
    }
    /// Digest of the canonical completeness assertion bytes.
    #[must_use]
    pub fn document_digest(&self) -> &str {
        self.source.document_digest()
    }
    /// Canonical completeness assertion identity.
    #[must_use]
    pub fn document_identity(&self) -> &str {
        self.source.document_identity()
    }
    /// Exact fact identities establishing or contradicting completeness.
    #[must_use]
    pub fn fact_identities(&self) -> &[String] {
        self.source.fact_identities()
    }
    /// Exact population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        self.source.population_identity()
    }
    /// Complete, incomplete, or contradicted state.
    #[must_use]
    pub const fn state(&self) -> quire_observation::authority::completeness::State {
        self.source.state()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Outcome {
    Value(bool),
    NonValue,
}

/// Constructor-private mapping view. It can only be derived from a validated result.
#[derive(Clone, Debug)]
pub struct MappedResultView<'a> {
    bytes: Vec<u8>,
    identity: String,
    source: &'a ValidatedResult,
    outcome: Outcome,
    usage: Usage,
}

struct PreparedMapping<'a> {
    value: Value,
    identity: String,
    source: &'a ValidatedResult,
    outcome: Outcome,
    semantic_usage: Usage,
}

impl MappedResultView<'_> {
    /// Canonical bytes of the selected mapping view.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Mapping identity, distinct from the source result identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
    /// Exact source result identity.
    #[must_use]
    pub fn source_result_id(&self) -> &str {
        self.source.result_id()
    }
    /// Exact source result digest.
    #[must_use]
    pub fn source_result_digest(&self) -> &str {
        self.source.digest()
    }
    /// Native temporal-subject identity retained from the validated source.
    #[must_use]
    pub fn native_subject_identity(&self) -> &str {
        self.source.native_subject_identity()
    }
    /// Source-bound checked-predicate/correspondence identity retained from the source.
    #[must_use]
    pub fn native_correspondence_identity(&self) -> &str {
        self.source.native_correspondence_identity()
    }
    /// Whether the source assessment executed.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        self.source.execution()
    }
    /// Native source truth disposition.
    #[must_use]
    pub const fn truth(&self) -> Truth {
        self.source.truth()
    }
    /// Returns a Boolean only for completed+satisfied or completed+violated.
    #[must_use]
    pub const fn value(&self) -> Option<bool> {
        match &self.outcome {
            Outcome::Value(value) => Some(*value),
            Outcome::NonValue => None,
        }
    }
    /// Complete source facts when the mapping does not carry a Boolean value.
    #[must_use]
    pub const fn non_value(&self) -> Option<NonValue<'_>> {
        match &self.outcome {
            Outcome::Value(_) => None,
            Outcome::NonValue => Some(NonValue {
                source: self.source,
            }),
        }
    }
    /// Decision-scope progress assertion.
    #[must_use]
    pub fn decision_scope_progress(&self) -> MappedAxisView<'_> {
        MappedAxisView {
            source: self.source.decision_scope_progress(),
        }
    }
    /// Decision-scope closure assertion.
    #[must_use]
    pub fn decision_scope_closure(&self) -> MappedAxisView<'_> {
        MappedAxisView {
            source: self.source.decision_scope_closure(),
        }
    }
    /// Surrounding-execution progress assertion.
    #[must_use]
    pub fn surrounding_execution_progress(&self) -> MappedAxisView<'_> {
        MappedAxisView {
            source: self.source.surrounding_execution_progress(),
        }
    }
    /// Surrounding-execution closure assertion.
    #[must_use]
    pub fn surrounding_execution_closure(&self) -> MappedAxisView<'_> {
        MappedAxisView {
            source: self.source.surrounding_execution_closure(),
        }
    }
    /// Complete owner assertion and fact population.
    #[must_use]
    pub fn completeness(&self) -> MappedCompletenessView<'_> {
        MappedCompletenessView {
            source: self.source.completeness(),
        }
    }
    /// Settlement and its exact supporting identities.
    #[must_use]
    pub const fn settlement(&self) -> &Settlement {
        self.source.settlement()
    }
    /// Decision-support records in declared causal order.
    #[must_use]
    pub fn decision_support(&self) -> &[DecisionSupport] {
        self.source.decision_support()
    }
    /// Direct correction predecessor, when present.
    #[must_use]
    pub const fn correction(&self) -> Option<&CorrectionView> {
        self.source.correction()
    }
    /// Work observed by mapping or strict reading.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

/// Derive the exact selected mapping from a validated source result.
pub fn map<'a>(
    source: &'a ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
) -> Result<MappedResultView<'a>> {
    validate_selection(selection)?;
    finish_mapping(prepare_mapping(source)?, limits.effective())
}

/// Derive the selected mapping under the shared QProtocol caller budget.
///
/// The derived-artifact unit is charged before constructing the view. Its
/// exact canonical byte length is then charged before allocating output bytes.
pub fn map_bounded<'a>(
    source: &'a ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView<'a>, Refusal> {
    resources.run_unit(
        UnitLabel::new("result::contract_ir::map_bounded"),
        |resources| map_bounded_unit(source, selection, limits, resources),
    )
}

fn map_bounded_unit<'a>(
    source: &'a ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView<'a>, Refusal> {
    if let Err(error) = validate_selection(selection) {
        return OperationOutcome::DomainError(error);
    }
    let effective = limits.effective();
    let semantic = mapping_usage(source);
    if let Err(error) = resources.charge_all_mixed(&[
        (DERIVED_SOURCE.id(), 1, None),
        (
            ITEM_SOURCE.id(),
            semantic.claims,
            Some(effective.max_claims),
        ),
        (
            SUPPORT_SOURCE.id(),
            semantic.support_records,
            Some(effective.max_support_records),
        ),
        (
            LINEAGE_SOURCE.id(),
            semantic.lineage_nodes,
            Some(effective.max_lineage_nodes),
        ),
        (ALLOCATION_SOURCE.id(), 1, None),
    ]) {
        return OperationOutcome::from_charge(error);
    }
    let prepared = match prepare_mapping(source) {
        Ok(prepared) => prepared,
        Err(error) => return OperationOutcome::DomainError(error),
    };
    let required = match canonical_len(&prepared.value) {
        Ok(required) => required,
        Err(_) => {
            return OperationOutcome::DomainError(Refusal::new(
                ErrorCode::MappingMismatch,
                "mapping cannot determine its canonical output length",
                Usage::default(),
                Some(source.result_id()),
            ));
        }
    };
    if let Err(error) =
        resources.charge_with_limit(OUTPUT_SOURCE.id(), required, effective.max_output_bytes)
    {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(finish_mapping(prepared, effective))
}

fn prepare_mapping(source: &ValidatedResult) -> Result<PreparedMapping<'_>> {
    let snapshot = &source.document.snapshot;
    let outcome = match (snapshot.execution, snapshot.truth) {
        (AssessmentExecution::Completed, Truth::Satisfied) => Outcome::Value(true),
        (AssessmentExecution::Completed, Truth::Violated) => Outcome::Value(false),
        (AssessmentExecution::Completed, Truth::Pending | Truth::Unavailable)
        | (
            AssessmentExecution::Refused
            | AssessmentExecution::Unsupported
            | AssessmentExecution::ResourceIncomplete
            | AssessmentExecution::Failed,
            Truth::Satisfied | Truth::Violated | Truth::Pending | Truth::Unavailable,
        ) => Outcome::NonValue,
    };
    let without_identity = mapping_value(source, snapshot, &outcome, None)?;
    let canonical_without_identity = canonicalize(&without_identity).map_err(|_| {
        Refusal::new(
            ErrorCode::MappingMismatch,
            "mapping preimage is outside the JCS subset",
            Usage::default(),
            Some(source.result_id()),
        )
    })?;
    let mut identity_preimage =
        Vec::with_capacity(CONTRACT.len() + 1 + canonical_without_identity.len());
    identity_preimage.extend_from_slice(CONTRACT.as_bytes());
    identity_preimage.push(0);
    identity_preimage.extend_from_slice(canonical_without_identity.as_bytes());
    let identity = format!("sha256:{:x}", ByteDigest::of(&identity_preimage));
    let value = mapping_value(source, snapshot, &outcome, Some(&identity))?;
    Ok(PreparedMapping {
        value,
        identity,
        source,
        outcome,
        semantic_usage: mapping_usage(source),
    })
}

fn mapping_usage(source: &ValidatedResult) -> Usage {
    let snapshot = &source.document.snapshot;
    Usage {
        claims: 1,
        support_records: snapshot.decision_support.len(),
        lineage_nodes: usize::from(snapshot.correction.is_some()),
        ..Usage::default()
    }
}

fn finish_mapping(prepared: PreparedMapping<'_>, limits: Limits) -> Result<MappedResultView<'_>> {
    let PreparedMapping {
        value,
        identity,
        source,
        outcome,
        semantic_usage,
    } = prepared;
    let bytes = canonicalize(&value)
        .map_err(|_| {
            Refusal::new(
                ErrorCode::MappingMismatch,
                "mapping cannot be canonicalized",
                Usage::default(),
                Some(source.result_id()),
            )
        })?
        .into_bytes();
    let usage = super::reader::preflight_output(&bytes, limits, semantic_usage)?;
    Ok(MappedResultView {
        bytes,
        identity,
        source,
        outcome,
        usage,
    })
}

/// Strict-read mapping bytes by re-deriving them from the validated source.
pub fn read<'a>(
    bytes: &[u8],
    source: &'a ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
) -> Result<MappedResultView<'a>> {
    validate_selection(selection)?;
    let effective = limits.effective();
    let observed = validate_offered(bytes, source, effective)?;
    let derived = map(source, selection, effective)?;
    compare_offered(bytes, source, observed, derived)
}

/// Strict-read mapping bytes under the shared QProtocol caller budget.
///
/// Hostile input structure is charged before JSON allocation. Re-deriving the
/// selected mapping then charges the same derived-artifact and output sites as
/// [`map_bounded`].
pub fn read_bounded<'a>(
    bytes: &[u8],
    source: &'a ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView<'a>, Refusal> {
    resources.run_unit(
        UnitLabel::new("result::contract_ir::read_bounded"),
        |resources| read_bounded_unit(bytes, source, selection, limits, resources),
    )
}

fn read_bounded_unit<'a>(
    bytes: &[u8],
    source: &'a ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView<'a>, Refusal> {
    if let Err(error) = validate_selection(selection) {
        return OperationOutcome::DomainError(error);
    }
    let effective = limits.effective();
    if let Err(error) = super::reader::charge_structural(bytes, effective, resources, READ_SOURCES)
    {
        return OperationOutcome::from_charge(error);
    }
    if let Err(error) = resources.charge(READ_ALLOCATION_SOURCE.id(), 1) {
        return OperationOutcome::from_charge(error);
    }
    let observed = match validate_offered(bytes, source, effective) {
        Ok(observed) => observed,
        Err(error) => return OperationOutcome::DomainError(error),
    };
    let derived = match map_bounded(source, selection, effective, resources) {
        OperationOutcome::Completed(derived) => derived,
        OperationOutcome::DomainError(error) => return OperationOutcome::DomainError(error),
        OperationOutcome::ResourceIncomplete(value) => {
            return OperationOutcome::ResourceIncomplete(value)
        }
        OperationOutcome::AccountingFault(fault) => {
            return OperationOutcome::AccountingFault(fault)
        }
    };
    OperationOutcome::from_domain(compare_offered(bytes, source, observed, derived))
}

fn validate_offered(bytes: &[u8], source: &ValidatedResult, limits: Limits) -> Result<Usage> {
    let observed = super::reader::preflight_external(bytes, limits)?;
    let value: Value = serde_json::from_slice(bytes).map_err(|_| {
        Refusal::new(
            ErrorCode::MappingMismatch,
            "mapping is not one valid JSON document",
            observed,
            Some(source.result_id()),
        )
    })?;
    if !super::reader::schema_accepts(&value, SCHEMA_BYTES, &MAPPING_SCHEMA_VALIDATOR) {
        return Err(Refusal::new(
            ErrorCode::MappingMismatch,
            "mapping is not a closed instance of the selected schema",
            observed,
            Some(source.result_id()),
        ));
    }
    Ok(observed)
}

fn compare_offered<'a>(
    bytes: &[u8],
    source: &ValidatedResult,
    observed: Usage,
    mut derived: MappedResultView<'a>,
) -> Result<MappedResultView<'a>> {
    if bytes != derived.bytes {
        return Err(Refusal::new(
            ErrorCode::MappingMismatch,
            "mapping bytes differ from the source-derived canonical view",
            observed,
            Some(source.result_id()),
        ));
    }
    derived.usage = observed;
    Ok(derived)
}

fn validate_selection(selection: MappingSelection<'_>) -> Result<()> {
    if selection.mapping_contract != CONTRACT
        || selection.mapping_schema_sha256 != SCHEMA_SHA256
        || selection.source_contract != RESULT_CONTRACT
        || selection.source_schema_sha256 != RESULT_SCHEMA_SHA256
    {
        return Err(Refusal::new(
            ErrorCode::ContractMismatch,
            "mapping or source contract/schema selection differs",
            Usage::default(),
            None,
        ));
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingWire<'a> {
    completeness: MappingCompletenessWire<'a>,
    contract: &'static str,
    correction: Option<MappingCorrectionWire<'a>>,
    decision_support: Vec<MappingSupportWire<'a>>,
    execution: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity: Option<&'a str>,
    native_correspondence_identity: &'a str,
    native_subject_identity: &'a str,
    outcome: OutcomeWire,
    progress_and_closure: ProgressAndClosureWire<'a>,
    schema_sha256: &'static str,
    settlement: MappingSettlementWire<'a>,
    source_result: SourceResultWire<'a>,
    truth: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceResultWire<'a> {
    contract: &'static str,
    digest: &'a str,
    result_id: &'a str,
    schema_sha256: &'static str,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
enum OutcomeWire {
    #[serde(rename = "value")]
    Value { value: bool },
    #[serde(rename = "non-value", rename_all = "camelCase")]
    NonValue {
        completeness: &'static str,
        execution: &'static str,
        settlement: &'static str,
        truth: &'static str,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressAndClosureWire<'a> {
    decision_scope_closure: MappingAxisWire<'a>,
    decision_scope_progress: MappingAxisWire<'a>,
    surrounding_execution_closure: MappingAxisWire<'a>,
    surrounding_execution_progress: MappingAxisWire<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingAxisWire<'a> {
    authority_identity: &'a str,
    authority_revision: &'a str,
    document_digest: &'a str,
    document_identity: &'a str,
    scope_identity: &'a str,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingCompletenessWire<'a> {
    boundary_identity: &'a str,
    document_digest: &'a str,
    document_identity: &'a str,
    fact_identities: &'a [String],
    population_identity: &'a str,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingSettlementWire<'a> {
    basis: &'static str,
    supporting_fact_identities: &'a [String],
    supporting_progress_identities: &'a [String],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingSupportWire<'a> {
    dependence_group: Option<&'a str>,
    observation_identity: &'a str,
    oracle_identity: &'a str,
    predecessor: Option<MappingSupportRefWire<'a>>,
    relation: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingSupportRefWire<'a> {
    observation_identity: &'a str,
    oracle_identity: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingCorrectionWire<'a> {
    contradicted_premise_identity: &'a str,
    corrected_input_identity: &'a str,
    predecessor_digest: &'a str,
    predecessor_result_id: &'a str,
    relation: &'static str,
}

fn mapping_value(
    source: &ValidatedResult,
    snapshot: &Snapshot,
    outcome: &Outcome,
    identity: Option<&str>,
) -> Result<Value> {
    let wire = MappingWire {
        completeness: mapping_completeness_wire(snapshot),
        contract: CONTRACT,
        correction: snapshot.correction.as_ref().map(mapping_correction_wire),
        decision_support: snapshot
            .decision_support
            .iter()
            .map(mapping_support_wire)
            .collect(),
        execution: snapshot.execution.label(),
        identity,
        native_correspondence_identity: &snapshot.native_correspondence_identity,
        native_subject_identity: &snapshot.native_subject_identity,
        outcome: outcome_wire(outcome, snapshot),
        progress_and_closure: ProgressAndClosureWire {
            decision_scope_closure: mapping_axis_wire(&snapshot.axes[1]),
            decision_scope_progress: mapping_axis_wire(&snapshot.axes[0]),
            surrounding_execution_closure: mapping_axis_wire(&snapshot.axes[3]),
            surrounding_execution_progress: mapping_axis_wire(&snapshot.axes[2]),
        },
        schema_sha256: SCHEMA_SHA256,
        settlement: MappingSettlementWire {
            basis: snapshot.settlement.basis.label(),
            supporting_fact_identities: &snapshot.settlement.supporting_fact_identities,
            supporting_progress_identities: &snapshot.settlement.supporting_progress_identities,
        },
        source_result: SourceResultWire {
            contract: RESULT_CONTRACT,
            digest: source.digest(),
            result_id: source.result_id(),
            schema_sha256: RESULT_SCHEMA_SHA256,
        },
        truth: snapshot.truth.label(),
    };
    serde_json::to_value(wire).map_err(|_| {
        Refusal::new(
            ErrorCode::MappingMismatch,
            "typed mapping view cannot be represented as JSON",
            Usage::default(),
            Some(source.result_id()),
        )
    })
}

fn outcome_wire(outcome: &Outcome, snapshot: &Snapshot) -> OutcomeWire {
    match outcome {
        Outcome::Value(value) => OutcomeWire::Value { value: *value },
        Outcome::NonValue => OutcomeWire::NonValue {
            completeness: completeness_label(snapshot.completeness.state),
            execution: snapshot.execution.label(),
            settlement: snapshot.settlement.basis.label(),
            truth: snapshot.truth.label(),
        },
    }
}

fn mapping_axis_wire(axis: &AxisView) -> MappingAxisWire<'_> {
    MappingAxisWire {
        authority_identity: &axis.authority_identity,
        authority_revision: &axis.authority_revision,
        document_digest: &axis.document_digest,
        document_identity: &axis.document_identity,
        scope_identity: &axis.scope_identity,
        state: axis.state.label(),
    }
}

fn mapping_completeness_wire(snapshot: &Snapshot) -> MappingCompletenessWire<'_> {
    MappingCompletenessWire {
        boundary_identity: &snapshot.completeness.boundary_identity,
        document_digest: &snapshot.completeness.document_digest,
        document_identity: &snapshot.completeness.document_identity,
        fact_identities: &snapshot.completeness.fact_identities,
        population_identity: &snapshot.completeness.population_identity,
        state: completeness_label(snapshot.completeness.state),
    }
}

fn mapping_support_wire(support: &DecisionSupport) -> MappingSupportWire<'_> {
    MappingSupportWire {
        dependence_group: support.dependence_group.as_deref(),
        observation_identity: &support.observation_identity,
        oracle_identity: &support.oracle_identity,
        predecessor: support
            .predecessor
            .as_ref()
            .map(|predecessor| MappingSupportRefWire {
                observation_identity: &predecessor.observation_identity,
                oracle_identity: &predecessor.oracle_identity,
            }),
        relation: support.relation.label(),
    }
}

fn mapping_correction_wire(correction: &CorrectionView) -> MappingCorrectionWire<'_> {
    MappingCorrectionWire {
        contradicted_premise_identity: &correction.contradicted_premise_identity,
        corrected_input_identity: &correction.corrected_input_identity,
        predecessor_digest: &correction.predecessor_digest,
        predecessor_result_id: &correction.predecessor_result_id,
        relation: correction.relation.label(),
    }
}
fn completeness_label(state: quire_observation::authority::completeness::State) -> &'static str {
    match state {
        quire_observation::authority::completeness::State::Complete => "complete",
        quire_observation::authority::completeness::State::Incomplete => "incomplete",
        quire_observation::authority::completeness::State::Contradicted => "contradicted",
    }
}

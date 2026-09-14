// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact, read-only Contract-IR result mapping derived from a validated result.

use serde_json::{json, Value};
use std::sync::OnceLock;

use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::closure::AssessmentExecution;
use crate::digest::jcs::{canonical_len, canonicalize};
use crate::refusal::catalog::{CodeEntry, CodeModule, Provenance};
use crate::refusal::{RefusalDomain, RetryDecision};
use crate::ByteDigest;

use super::record::{
    CorrectionSnapshot, DecisionSupport, Limits, OwnerAxis, SettlementBasis, Snapshot, Truth,
    Usage, ValidatedResult, CONTRACT as RESULT_CONTRACT, SCHEMA_SHA256 as RESULT_SCHEMA_SHA256,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonValue {
    execution: AssessmentExecution,
    truth: Truth,
    settlement: SettlementBasis,
    completeness: quire_observation::authority::completeness::State,
}

impl NonValue {
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        self.execution
    }
    #[must_use]
    pub const fn truth(&self) -> Truth {
        self.truth
    }
    #[must_use]
    pub const fn settlement(&self) -> SettlementBasis {
        self.settlement
    }
    #[must_use]
    pub const fn completeness(&self) -> quire_observation::authority::completeness::State {
        self.completeness
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Outcome {
    Value(bool),
    NonValue(NonValue),
}

/// Constructor-private mapping view. It can only be derived from a validated result.
#[derive(Clone, Debug)]
pub struct MappedResultView {
    bytes: Vec<u8>,
    identity: String,
    source_result_id: String,
    source_result_digest: String,
    outcome: Outcome,
    usage: Usage,
}

struct PreparedMapping {
    value: Value,
    identity: String,
    source_result_id: String,
    source_result_digest: String,
    outcome: Outcome,
    semantic_usage: Usage,
}

impl MappedResultView {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
    #[must_use]
    pub fn source_result_id(&self) -> &str {
        &self.source_result_id
    }
    #[must_use]
    pub fn source_result_digest(&self) -> &str {
        &self.source_result_digest
    }
    /// Returns a Boolean only for completed+satisfied or completed+violated.
    #[must_use]
    pub const fn value(&self) -> Option<bool> {
        match &self.outcome {
            Outcome::Value(value) => Some(*value),
            Outcome::NonValue(_) => None,
        }
    }
    #[must_use]
    pub const fn non_value(&self) -> Option<&NonValue> {
        match &self.outcome {
            Outcome::Value(_) => None,
            Outcome::NonValue(value) => Some(value),
        }
    }
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

/// Derive the exact selected mapping from a validated source result.
pub fn map(
    source: &ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
) -> Result<MappedResultView> {
    validate_selection(selection)?;
    finish_mapping(prepare_mapping(source)?, limits.effective())
}

/// Derive the selected mapping under the shared QProtocol caller budget.
///
/// The derived-artifact unit is charged before constructing the view. Its
/// exact canonical byte length is then charged before allocating output bytes.
pub fn map_bounded(
    source: &ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView, Refusal> {
    resources.run_unit(
        UnitLabel::new("result::contract_ir::map_bounded"),
        |resources| map_bounded_unit(source, selection, limits, resources),
    )
}

fn map_bounded_unit(
    source: &ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView, Refusal> {
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

fn prepare_mapping(source: &ValidatedResult) -> Result<PreparedMapping> {
    let snapshot = &source.document.snapshot;
    let outcome = match (snapshot.execution, snapshot.truth) {
        (AssessmentExecution::Completed, Truth::Satisfied) => Outcome::Value(true),
        (AssessmentExecution::Completed, Truth::Violated) => Outcome::Value(false),
        (execution, truth) => Outcome::NonValue(NonValue {
            execution,
            truth,
            settlement: snapshot.settlement_basis,
            completeness: snapshot.completeness_state,
        }),
    };
    let without_identity = mapping_value(source, snapshot, &outcome, None);
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
    let value = mapping_value(source, snapshot, &outcome, Some(&identity));
    Ok(PreparedMapping {
        value,
        identity,
        source_result_id: source.result_id().to_owned(),
        source_result_digest: source.digest().to_owned(),
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

fn finish_mapping(prepared: PreparedMapping, limits: Limits) -> Result<MappedResultView> {
    let PreparedMapping {
        value,
        identity,
        source_result_id,
        source_result_digest,
        outcome,
        semantic_usage,
    } = prepared;
    let bytes = canonicalize(&value)
        .map_err(|_| {
            Refusal::new(
                ErrorCode::MappingMismatch,
                "mapping cannot be canonicalized",
                Usage::default(),
                Some(&source_result_id),
            )
        })?
        .into_bytes();
    let usage = super::reader::preflight_output(&bytes, limits, semantic_usage)?;
    Ok(MappedResultView {
        bytes,
        identity,
        source_result_id,
        source_result_digest,
        outcome,
        usage,
    })
}

/// Strict-read mapping bytes by re-deriving them from the validated source.
pub fn read(
    bytes: &[u8],
    source: &ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
) -> Result<MappedResultView> {
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
pub fn read_bounded(
    bytes: &[u8],
    source: &ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView, Refusal> {
    resources.run_unit(
        UnitLabel::new("result::contract_ir::read_bounded"),
        |resources| read_bounded_unit(bytes, source, selection, limits, resources),
    )
}

fn read_bounded_unit(
    bytes: &[u8],
    source: &ValidatedResult,
    selection: MappingSelection<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<MappedResultView, Refusal> {
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

fn compare_offered(
    bytes: &[u8],
    source: &ValidatedResult,
    observed: Usage,
    mut derived: MappedResultView,
) -> Result<MappedResultView> {
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

fn mapping_value(
    source: &ValidatedResult,
    snapshot: &Snapshot,
    outcome: &Outcome,
    identity: Option<&str>,
) -> Value {
    let mut value = json!({
        "completeness": {"boundaryIdentity": snapshot.completeness_boundary_identity, "documentDigest": snapshot.completeness_digest, "documentIdentity": snapshot.completeness_identity, "factIdentities": snapshot.completeness_fact_identities, "populationIdentity": snapshot.completeness_population_identity, "state": completeness_label(snapshot.completeness_state)},
        "contract": CONTRACT,
        "correction": snapshot.correction.as_ref().map(correction_value),
        "decisionSupport": snapshot.decision_support.iter().map(support_value).collect::<Vec<_>>(),
        "execution": snapshot.execution.label(),
        "identity": identity,
        "nativeCorrespondenceIdentity": snapshot.native_correspondence_identity,
        "nativeSubjectIdentity": snapshot.native_subject_identity,
        "outcome": outcome_value(outcome),
        "progressAndClosure": {"decisionScopeClosure": axis_value(&snapshot.axes[1]), "decisionScopeProgress": axis_value(&snapshot.axes[0]), "surroundingExecutionClosure": axis_value(&snapshot.axes[3]), "surroundingExecutionProgress": axis_value(&snapshot.axes[2])},
        "schemaSha256": SCHEMA_SHA256,
        "settlement": {"basis": snapshot.settlement_basis.label(), "supportingFactIdentities": snapshot.settlement.supporting_fact_identities, "supportingProgressIdentities": snapshot.settlement.supporting_progress_identities},
        "sourceResult": {"contract": RESULT_CONTRACT, "digest": source.digest(), "resultId": source.result_id(), "schemaSha256": RESULT_SCHEMA_SHA256},
        "truth": snapshot.truth.label()
    });
    if identity.is_none() {
        if let Some(object) = value.as_object_mut() {
            object.remove("identity");
        }
    }
    value
}

fn outcome_value(outcome: &Outcome) -> Value {
    match outcome {
        Outcome::Value(value) => json!({"kind": "value", "value": value}),
        Outcome::NonValue(value) => {
            json!({"completeness": completeness_label(value.completeness), "execution": value.execution.label(), "kind": "non-value", "settlement": value.settlement.label(), "truth": value.truth.label()})
        }
    }
}

fn axis_value(axis: &OwnerAxis) -> Value {
    json!({"authorityIdentity": axis.authority_identity, "authorityRevision": axis.authority_revision, "documentDigest": axis.document_digest, "documentIdentity": axis.document_identity, "scopeIdentity": axis.scope_identity, "state": axis.state.label()})
}
fn support_value(s: &DecisionSupport) -> Value {
    json!({"dependenceGroup": s.dependence_group, "observationIdentity": s.observation_identity, "oracleIdentity": s.oracle_identity, "predecessor": s.predecessor.as_ref().map(|p| json!({"observationIdentity": p.observation_identity, "oracleIdentity": p.oracle_identity})), "relation": s.relation.label()})
}
fn correction_value(c: &CorrectionSnapshot) -> Value {
    json!({"contradictedPremiseIdentity": c.contradicted_premise_identity, "correctedInputIdentity": c.corrected_input_identity, "predecessorDigest": c.predecessor_digest, "predecessorResultId": c.predecessor_result_id, "relation": c.relation.label()})
}
fn completeness_label(state: quire_observation::authority::completeness::State) -> &'static str {
    match state {
        quire_observation::authority::completeness::State::Complete => "complete",
        quire_observation::authority::completeness::State::Incomplete => "incomplete",
        quire_observation::authority::completeness::State::Contradicted => "contradicted",
    }
}

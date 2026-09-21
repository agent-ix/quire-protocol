// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-bound projection of canonical protocol results to qverify E02.
//!
//! QProtocol owns the mapping and its refusal policy. Qverify owns every E02
//! wire type, the RFC 8785 implementation used for projection identity, and
//! the final strict reader. The emitted bytes retain open E02 finding detail;
//! the accompanying owner-generated type is returned only after those exact
//! bytes pass qverify's accepted reader.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use quire_verification_contracts::wire::{
    ObservationReference, OracleReference, SharedArtifactEnvelope, TechniqueResult,
};
use quire_verification_contracts::{
    self as contracts, jcs_canonicalize, jcs_sha256, PublicContract, VerificationErrorCode,
};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome as BoundedOutcome, ResourceAccountant,
    SourceSite, Usage,
};
use crate::bounds::{Clock, SiteId, SiteKind, UnitLabel};
use crate::closure::AssessmentExecution;
use crate::digest::jcs::canonical_len;
use crate::outcome::OperationOutcome;
use crate::refusal::catalog::{
    CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode, RetainedBytes,
};
use crate::refusal::{
    AffectedIdentity, Prerequisite, PrerequisiteState, ProtocolCause, ProtocolRefusal,
    ProtocolStage, RefusalAxis, RefusalDomain, ResourceUsage, RetryDecision, Unavailability,
};
use crate::result::{
    self, Activation, Adequacy, AssumptionState, DependenceRelation, Expected, Limits,
    Participation, Truth, ValidatedResult,
};
use crate::version_lock::{
    self, EntryKey, InterfaceSelection, ManifestEntry, ResolutionBudget, ResolutionRequestIdentity,
    ResolvedRequest, VersionLockManifest,
};

/// Exact revision of `quire-verification-contracts` that owns the accepted public E02 Rust
/// interface (PLAT-861; previously the private `quire-verification` revision directly).
pub const QVERIFY_E02_REVISION: &str = "da8d7272051b1cf02768ec459539551414fd6c21";
/// Owner of the selected E02 interface.
pub const QVERIFY_OWNER: &str = "agent-ix/quire-verification-contracts";
/// Projection-identity preimage version selected by FR-034.
pub const IDENTITY_VERSION: &str = "quire.protocol.e02-projection-identity/v1-draft.1";
/// Media type of the one retained canonical-result native artifact.
pub const SOURCE_MEDIA_TYPE: &str =
    "application/vnd.agent-ix.quire-protocol-result+json;version=1-draft.1";

const IMMUTABILITY_ASSERTION: &[u8] = b"owner asserts immutable and published";

pub(crate) const DERIVED_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/derive"),
    "projection::project_e02",
    BudgetDimension::DerivedArtifact,
    SiteKind::Retention,
);
pub(crate) const ITEM_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/item/retain"),
    "projection::project_e02",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/item/visit"),
    "projection::project_e02",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const CANDIDATE_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/candidate/allocate"),
    "projection::project_e02",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
pub(crate) const OUTPUT_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/output/allocate"),
    "projection::project_e02",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
pub(crate) const TARGET_READ_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/target-read/allocate"),
    "projection::project_e02",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
pub(crate) const SOURCE_COPY_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/source-copy/allocate"),
    "projection::project_e02",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
pub(crate) const OUTPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("projection/e02/output/retain"),
    "projection::project_e02",
    BudgetDimension::OutputByte,
    SiteKind::Retention,
);

/// The one qverify contract this adapter is compiled to consume.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum E02Contract {
    /// `quire-verification/e02-draft-2` strict reader and generated Rust types.
    Draft2,
}

impl E02Contract {
    /// Exact owner contract version imported from qverify.
    #[must_use]
    pub fn contract_version(self) -> &'static str {
        match self {
            Self::Draft2 => contracts::contract_version(),
        }
    }

    /// Componentwise version-lock key.
    #[must_use]
    pub fn key(self) -> EntryKey {
        EntryKey::new(QVERIFY_OWNER.as_bytes(), self.contract_version().as_bytes())
            .expect("the compiled qverify selection has nonempty components")
    }

    /// Exact Protocol-authored manifest entry for the accepted interface.
    #[must_use]
    pub fn manifest_entry(self) -> ManifestEntry {
        ManifestEntry::pinned_interface(
            self.key(),
            Some(QVERIFY_E02_REVISION.as_bytes()),
            Some(IMMUTABILITY_ASSERTION),
            None,
        )
    }
}

/// Nonempty identity binding one resolution to one projection request.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectionRequestId(Box<[u8]>);

impl ProjectionRequestId {
    /// Retain one exact nonempty request identity.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyProjectionRequestId`] when `identity` is empty.
    pub fn new(identity: &[u8]) -> Result<Self, EmptyProjectionRequestId> {
        if identity.is_empty() {
            Err(EmptyProjectionRequestId)
        } else {
            Ok(Self(identity.into()))
        }
    }

    /// Exact request bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A projection request identity was empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyProjectionRequestId;

impl fmt::Display for EmptyProjectionRequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("projection request identity must not be empty")
    }
}

impl std::error::Error for EmptyProjectionRequestId {}

/// Opaque request-bound authorization for qverify E02 projection.
pub struct E02Resolution {
    request: ProjectionRequestId,
    manifest: Vec<ManifestEntry>,
    usage: Usage,
    resolved: ResolvedRequest,
}

impl fmt::Debug for E02Resolution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("E02Resolution")
            .field("request", &self.request)
            .field("manifest", &self.manifest)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

impl E02Resolution {
    /// Exact request this capability authorizes.
    #[must_use]
    pub const fn request(&self) -> &ProjectionRequestId {
        &self.request
    }

    /// Complete selected manifest state.
    #[must_use]
    pub fn manifest_entries(&self) -> &[ManifestEntry] {
        &self.manifest
    }

    /// Resolver usage retained with the capability.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    fn authorize(&self, request: &ProjectionRequestId) -> OperationOutcome<()> {
        let request = ResolutionRequestIdentity::new(request.as_bytes())
            .expect("public request construction rejected empty identity bytes");
        let contract = E02Contract::Draft2;
        let selected =
            InterfaceSelection::<E02Contract>::new(contract.key(), QVERIFY_E02_REVISION.as_bytes());
        let capability = match self.resolved.interface(&selected) {
            OperationOutcome::Completed(capability) => capability,
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete);
            }
        };
        capability.authorize(&request, &selected)
    }
}

/// Resolve the exact qverify E02 interface for one projection request.
#[must_use]
pub fn resolve_e02_contract<C: Clock + 'static>(
    request: &ProjectionRequestId,
    manifest: &VersionLockManifest,
    budget: ResolutionBudget,
    clock: C,
) -> OperationOutcome<E02Resolution> {
    let private_request = ResolutionRequestIdentity::new(request.as_bytes())
        .expect("public request construction rejected empty identity bytes");
    let contract = E02Contract::Draft2;
    let resolved = match version_lock::resolve(
        private_request,
        manifest,
        &[contract.key()],
        &[],
        budget,
        clock,
    ) {
        OperationOutcome::Completed(resolved) => resolved,
        OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
        OperationOutcome::ResourceIncomplete(incomplete) => {
            return OperationOutcome::ResourceIncomplete(incomplete);
        }
    };
    let selected =
        InterfaceSelection::<E02Contract>::new(contract.key(), QVERIFY_E02_REVISION.as_bytes());
    match resolved.interface(&selected) {
        OperationOutcome::Completed(_) => {}
        OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
        OperationOutcome::ResourceIncomplete(incomplete) => {
            return OperationOutcome::ResourceIncomplete(incomplete);
        }
    }
    let manifest = resolved.entries().to_vec();
    let usage = resolved.usage();
    OperationOutcome::Completed(E02Resolution {
        request: request.clone(),
        manifest,
        usage,
        resolved,
    })
}

/// Target reference pair for one exact source decision-support identity pair.
#[derive(Clone, Debug)]
pub struct DecisionSupportBinding {
    /// Exact source oracle identity.
    pub source_oracle_identity: String,
    /// Exact source observation identity.
    pub source_observation_identity: String,
    /// Selected E02 oracle reference carrying the same identity.
    pub oracle_ref: OracleReference,
    /// Selected E02 observation reference carrying the same identity.
    pub observation_ref: ObservationReference,
}

/// Complete caller-selected E02 execution binding and projection provenance.
#[derive(Clone, Debug)]
pub struct ProjectionContext {
    /// Source contract selected independently of the source bytes.
    pub source_contract_version: String,
    /// Target contract selected independently of the output candidate.
    pub target_contract_version: String,
    /// Required target features; draft-2 currently accepts none.
    pub required_features: Vec<String>,
    /// URI that dereferences to the unchanged canonical source bytes.
    pub source_uri: String,
    /// Independently selected digest of those source bytes.
    pub source_digest: String,
    /// Optional independently asserted projection identity.
    pub expected_result_id: Option<String>,
    /// Exact E02 plan identity.
    pub plan_id: String,
    /// Exact E02 task identity.
    pub task_id: String,
    /// Exact selected technique identity.
    pub technique_id: String,
    /// Exact run reference.
    pub run_ref: SharedArtifactEnvelope,
    /// Exact property/obligation reference.
    pub property_ref: SharedArtifactEnvelope,
    /// Exact ordered subject references.
    pub subject_refs: Vec<SharedArtifactEnvelope>,
    /// Exact profile identity.
    pub profile: String,
    /// Exact semantically ordered binding references.
    pub binding_refs: Vec<SharedArtifactEnvelope>,
    /// Exact dependency-closure identity.
    pub dependency_closure: String,
    /// Exact open E02 execution-scope object.
    pub execution_scope: Map<String, Value>,
    /// RFC 3339 production time supplied by the execution context.
    pub produced_at: String,
    /// Protocol producer version selected by the caller.
    pub producer_version: String,
    /// Adapter identity.
    pub adapter_identity: String,
    /// Adapter version.
    pub adapter_version: String,
    /// Exact target references for every source decision-support record.
    pub decision_support_bindings: Vec<DecisionSupportBinding>,
}

/// Exact successful projection: lossless canonical bytes plus qverify's typed view.
#[derive(Clone, Debug)]
pub struct ProjectedTechniqueResult {
    result: TechniqueResult,
    bytes: Box<[u8]>,
    source_bytes: Box<[u8]>,
}

impl ProjectedTechniqueResult {
    /// Owner-generated qverify type accepted from the exact emitted bytes.
    #[must_use]
    pub const fn result(&self) -> &TechniqueResult {
        &self.result
    }

    /// Canonical E02 result bytes, including all retained open-axis detail.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Unchanged strict-reader-accepted canonical source bytes.
    #[must_use]
    pub fn source_bytes(&self) -> &[u8] {
        &self.source_bytes
    }

    /// Derived projection identity.
    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result.result_id
    }
}

/// Project canonical source bytes through the qverify strict reader.
#[must_use]
pub fn project_e02(
    source_bytes: &[u8],
    source_expected: Expected<'_>,
    source_limits: Limits,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
    context: &ProjectionContext,
    resources: &mut ResourceAccountant<'_>,
) -> BoundedOutcome<ProjectedTechniqueResult, ProtocolRefusal> {
    resources.run_unit(UnitLabel::new("projection::project_e02"), |resources| {
        project_e02_unit(
            source_bytes,
            source_expected,
            source_limits,
            request,
            resolution,
            context,
            resources,
        )
    })
}

fn project_e02_unit(
    source_bytes: &[u8],
    source_expected: Expected<'_>,
    source_limits: Limits,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
    context: &ProjectionContext,
    resources: &mut ResourceAccountant<'_>,
) -> BoundedOutcome<ProjectedTechniqueResult, ProtocolRefusal> {
    match resolution.authorize(request) {
        OperationOutcome::Completed(()) => {}
        OperationOutcome::Refused(refusal) => return BoundedOutcome::DomainError(refusal),
        OperationOutcome::ResourceIncomplete(incomplete) => {
            return BoundedOutcome::ResourceIncomplete(incomplete);
        }
    }
    let source_affected = source_expected.enclosing_digest;
    let source = match result::read_bounded(source_bytes, source_expected, source_limits, resources)
    {
        BoundedOutcome::Completed(source) => source,
        BoundedOutcome::DomainError(_) => {
            return BoundedOutcome::DomainError(projection_refusal(
                ProjectionRefusalCode::InvalidSourceResult,
                source_affected.as_bytes(),
            ));
        }
        BoundedOutcome::ResourceIncomplete(incomplete) => {
            return BoundedOutcome::ResourceIncomplete(incomplete);
        }
        BoundedOutcome::AccountingFault(fault) => {
            return BoundedOutcome::AccountingFault(fault);
        }
    };
    match project_validated(&source, context, resources) {
        Ok(projected) => BoundedOutcome::Completed(projected),
        Err(ProjectionFailure::Domain(code, identity)) => {
            BoundedOutcome::DomainError(projection_refusal(code, identity.as_bytes()))
        }
        Err(ProjectionFailure::Resource(error)) => BoundedOutcome::from_charge(error),
    }
}

enum ProjectionFailure {
    Domain(ProjectionRefusalCode, String),
    Resource(ChargeError),
}

impl From<(ProjectionRefusalCode, String)> for ProjectionFailure {
    fn from((code, identity): (ProjectionRefusalCode, String)) -> Self {
        Self::Domain(code, identity)
    }
}

impl From<ChargeError> for ProjectionFailure {
    fn from(error: ChargeError) -> Self {
        Self::Resource(error)
    }
}

fn project_validated(
    source: &ValidatedResult,
    context: &ProjectionContext,
    resources: &mut ResourceAccountant<'_>,
) -> Result<ProjectedTechniqueResult, ProjectionFailure> {
    let item_count = context
        .subject_refs
        .len()
        .saturating_add(context.binding_refs.len())
        .saturating_add(source.decision_support().len())
        .saturating_add(source.assumptions().len())
        .saturating_add(source.limitations().len().saturating_mul(2))
        .saturating_add(1);
    resources.charge_all(&[
        (DERIVED_SOURCE.id(), 1),
        (ITEM_SOURCE.id(), item_count),
        (OPERATION_SOURCE.id(), item_count),
        (CANDIDATE_ALLOCATION_SOURCE.id(), 1),
    ])?;
    validate_versions(context)?;
    validate_nonempty_context(context)?;
    let source_value: Value = serde_json::from_slice(source.bytes()).map_err(|_| {
        affected(
            ProjectionRefusalCode::InvalidSourceResult,
            source.result_id(),
        )
    })?;
    let recomputed_digest = jcs_sha256(&source_value).map_err(|_| {
        affected(
            ProjectionRefusalCode::InvalidSourceResult,
            source.result_id(),
        )
    })?;
    if recomputed_digest != source.digest() || recomputed_digest != context.source_digest {
        return Err(affected(
            ProjectionRefusalCode::InvalidSourceResult,
            &context.source_digest,
        )
        .into());
    }

    let target_binding = json!({
        "propertyRef": value(&context.property_ref)?,
        "planId": context.plan_id,
        "taskId": context.task_id,
        "runRef": value(&context.run_ref)?,
        "profile": context.profile,
        "bindingRefs": values(&context.binding_refs)?,
        "dependencyClosure": context.dependency_closure,
        "executionScope": context.execution_scope,
    });
    let identity_preimage = json!({
        "identityVersion": IDENTITY_VERSION,
        "sourceResultRef": {
            "identity": source.result_id(),
            "digest": recomputed_digest,
        },
        "targetContractVersion": context.target_contract_version,
        "targetBinding": target_binding,
        "adapter": {
            "identity": context.adapter_identity,
            "version": context.adapter_version,
        },
    });
    let result_id = jcs_sha256(&identity_preimage)
        .map_err(|_| affected(ProjectionRefusalCode::IdentityMismatch, source.result_id()))?;
    if result_id == source.result_id()
        || context
            .expected_result_id
            .as_ref()
            .is_some_and(|expected| expected != &result_id)
    {
        return Err(affected(
            ProjectionRefusalCode::IdentityMismatch,
            context
                .expected_result_id
                .as_deref()
                .unwrap_or(source.result_id()),
        )
        .into());
    }

    let contributions = map_contributions(source, context, &result_id)?;
    let participation = map_participation(source.participation());
    let limitations = source
        .limitations()
        .iter()
        .map(|limitation| {
            json!({
                "code": limitation.code,
                "affectedIdentity": limitation.affected_identity,
            })
        })
        .collect::<Vec<_>>();
    let candidate = json!({
        "contractVersion": context.target_contract_version,
        "kind": "TechniqueResult",
        "resultId": result_id,
        "planId": context.plan_id,
        "taskId": context.task_id,
        "runRef": value(&context.run_ref)?,
        "techniqueId": context.technique_id,
        "producedAt": context.produced_at,
        "propertyRef": value(&context.property_ref)?,
        "subjectRefs": values(&context.subject_refs)?,
        "profile": context.profile,
        "bindingRefs": values(&context.binding_refs)?,
        "dependencyClosure": context.dependency_closure,
        "executionScope": context.execution_scope,
        "execution": {
            "status": map_execution(source.execution()),
            "sourceDisposition": source.execution().label(),
        },
        "logical": {"outcome": map_truth(source)},
        "activation": {"status": map_activation(source.activation())},
        "participation": participation,
        "assumptions": source.assumptions().iter().map(|assumption| json!({
            "assumptionId": assumption.identity,
            "status": map_assumption(assumption.state),
        })).collect::<Vec<_>>(),
        "adequacy": {
            "status": map_adequacy(source.observation_adequacy()),
            "sourceLimitations": limitations,
        },
        "provenance": {
            "producer": source.provenance().producer_identity,
            "producerVersion": context.producer_version,
            "adapter": context.adapter_identity,
            "adapterVersion": context.adapter_version,
            "sourceRevision": source.digest(),
        },
        "nativeArtifacts": [{
            "uri": context.source_uri,
            "digest": recomputed_digest,
            "mediaType": SOURCE_MEDIA_TYPE,
        }],
        "oracleContributions": contributions,
        "sourceLimitations": limitations,
    });
    verify_projection(source, context, &candidate)?;

    let required = canonical_len(&candidate)
        .map_err(|_| affected(ProjectionRefusalCode::FindingLoss, source.result_id()))?;
    resources.charge_all(&[
        (OUTPUT_ALLOCATION_SOURCE.id(), 1),
        (OUTPUT_SOURCE.id(), required),
    ])?;
    let bytes = jcs_canonicalize(&candidate)
        .map_err(|_| affected(ProjectionRefusalCode::FindingLoss, source.result_id()))?;
    resources.charge(TARGET_READ_ALLOCATION_SOURCE.id(), 1)?;
    let reread = contracts::parse_bounded_json(&bytes)
        .and_then(contracts::validate_contract)
        .map_err(|error| map_target_error(error.code(), source.result_id()))?;
    let PublicContract::TechniqueResult(result) = reread else {
        return Err(affected(ProjectionRefusalCode::FindingLoss, source.result_id()).into());
    };
    resources.charge(SOURCE_COPY_ALLOCATION_SOURCE.id(), 1)?;
    Ok(ProjectedTechniqueResult {
        result: *result,
        bytes: bytes.into_boxed_slice(),
        source_bytes: source.bytes().into(),
    })
}

fn validate_versions(context: &ProjectionContext) -> Result<(), (ProjectionRefusalCode, String)> {
    if context.source_contract_version != result::CONTRACT {
        return Err(affected(
            ProjectionRefusalCode::UnsupportedSourceVersion,
            &context.source_contract_version,
        ));
    }
    if context.target_contract_version != contracts::contract_version() {
        return Err(affected(
            ProjectionRefusalCode::UnsupportedTargetVersion,
            &context.target_contract_version,
        ));
    }
    if let Some(feature) = context.required_features.first() {
        return Err(affected(
            ProjectionRefusalCode::UnsupportedTargetVersion,
            feature,
        ));
    }
    Ok(())
}

fn validate_nonempty_context(
    context: &ProjectionContext,
) -> Result<(), (ProjectionRefusalCode, String)> {
    for (field, value) in [
        ("sourceUri", context.source_uri.as_str()),
        ("sourceDigest", context.source_digest.as_str()),
        ("planId", context.plan_id.as_str()),
        ("taskId", context.task_id.as_str()),
        ("techniqueId", context.technique_id.as_str()),
        ("profile", context.profile.as_str()),
        ("dependencyClosure", context.dependency_closure.as_str()),
        ("producedAt", context.produced_at.as_str()),
        ("producerVersion", context.producer_version.as_str()),
        ("adapterIdentity", context.adapter_identity.as_str()),
        ("adapterVersion", context.adapter_version.as_str()),
    ] {
        if value.is_empty() {
            return Err(affected(ProjectionRefusalCode::MissingContext, field));
        }
    }
    Ok(())
}

fn map_execution(execution: AssessmentExecution) -> &'static str {
    match execution {
        AssessmentExecution::Completed => "completed",
        AssessmentExecution::Refused => "invalid-input",
        AssessmentExecution::Unsupported => "unsupported",
        AssessmentExecution::ResourceIncomplete => "exhausted",
        AssessmentExecution::Failed => "failed",
    }
}

fn map_truth(source: &ValidatedResult) -> &'static str {
    match source.truth() {
        Truth::Satisfied => "satisfied",
        Truth::Violated => "violated",
        Truth::Pending => "inconclusive",
        Truth::Unavailable
            if source.execution() != AssessmentExecution::Completed
                || source.activation() == Activation::Inactive =>
        {
            "not-evaluated"
        }
        Truth::Unavailable => "unknown",
    }
}

fn map_activation(activation: Activation) -> &'static str {
    match activation {
        Activation::Triggered => "activated",
        Activation::Inactive => "not-activated",
        Activation::ActivationUnknown => "unknown",
    }
}

fn map_assumption(state: AssumptionState) -> &'static str {
    match state {
        AssumptionState::Met => "met",
        AssumptionState::NotMet => "not-met",
        AssumptionState::Unknown => "unknown",
        AssumptionState::NotAssessed => "not-assessed",
    }
}

fn map_adequacy(adequacy: Adequacy) -> &'static str {
    match adequacy {
        Adequacy::Adequate => "adequate",
        Adequacy::Inadequate => "inadequate",
        Adequacy::Unknown => "unknown",
        Adequacy::NotAssessed => "not-assessed",
    }
}

fn map_participation(participation: &Participation) -> Value {
    match participation {
        Participation::Known {
            required_role_instance_refs,
            participating_role_instance_refs,
        } => {
            let participating = participating_role_instance_refs
                .iter()
                .collect::<BTreeSet<_>>();
            let missing = required_role_instance_refs
                .iter()
                .filter(|required| !participating.contains(required))
                .cloned()
                .collect::<Vec<_>>();
            let status = if required_role_instance_refs.is_empty() {
                "not-required"
            } else if missing.is_empty() {
                "complete"
            } else if participating_role_instance_refs.is_empty() {
                "not-started"
            } else {
                "partial"
            };
            json!({
                "status": status,
                "requiredParticipants": required_role_instance_refs,
                "participants": participating_role_instance_refs,
                "missingParticipants": missing,
            })
        }
        Participation::Unknown {
            participating_role_instance_refs,
            missing_dependency_refs,
            reasons,
        } => json!({
            "status": "unknown",
            "participants": participating_role_instance_refs,
            "missingDependencies": missing_dependency_refs,
            "reasons": reasons,
        }),
    }
}

fn map_contributions(
    source: &ValidatedResult,
    context: &ProjectionContext,
    result_id: &str,
) -> Result<Vec<Value>, (ProjectionRefusalCode, String)> {
    let support = source.decision_support();
    if support.len() != context.decision_support_bindings.len() {
        return Err(affected(
            ProjectionRefusalCode::MissingContext,
            "decisionSupportBindings",
        ));
    }
    let mut bindings = BTreeMap::new();
    for binding in &context.decision_support_bindings {
        let key = (
            binding.source_oracle_identity.as_str(),
            binding.source_observation_identity.as_str(),
        );
        if bindings.insert(key, binding).is_some()
            || oracle_identity(&binding.oracle_ref) != binding.source_oracle_identity
            || observation_identity(&binding.observation_ref) != binding.source_observation_identity
        {
            return Err(affected(
                ProjectionRefusalCode::IdentityMismatch,
                &binding.source_oracle_identity,
            ));
        }
    }
    let ids = support
        .iter()
        .enumerate()
        .map(|(index, item)| {
            (
                (
                    item.oracle_identity.as_str(),
                    item.observation_identity.as_str(),
                ),
                format!("protocol-support-{index}"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    support
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let key = (
                item.oracle_identity.as_str(),
                item.observation_identity.as_str(),
            );
            let binding = bindings.get(&key).ok_or_else(|| {
                affected(
                    ProjectionRefusalCode::IdentityMismatch,
                    &item.oracle_identity,
                )
            })?;
            let mut contribution = json!({
                "contributionId": format!("protocol-support-{index}"),
                "oracleRef": value(&binding.oracle_ref)?,
                "observationRef": value(&binding.observation_ref)?,
                "relation": relation(item.relation),
            });
            match item.relation {
                DependenceRelation::Shared => {
                    let group = item.dependence_group.as_ref().ok_or_else(|| {
                        affected(ProjectionRefusalCode::FindingLoss, &item.oracle_identity)
                    })?;
                    contribution["dependenceGroupId"] = Value::String(group.clone());
                }
                DependenceRelation::Derived => {
                    let predecessor = item.predecessor.as_ref().ok_or_else(|| {
                        affected(ProjectionRefusalCode::FindingLoss, &item.oracle_identity)
                    })?;
                    let predecessor_id = ids
                        .get(&(
                            predecessor.oracle_identity.as_str(),
                            predecessor.observation_identity.as_str(),
                        ))
                        .ok_or_else(|| {
                            affected(ProjectionRefusalCode::FindingLoss, &item.oracle_identity)
                        })?;
                    contribution["derivedFrom"] = json!({
                        "resultId": result_id,
                        "contributionId": predecessor_id,
                    });
                }
                DependenceRelation::Independent | DependenceRelation::Unknown => {}
            }
            Ok(contribution)
        })
        .collect()
}

fn relation(relation: DependenceRelation) -> &'static str {
    match relation {
        DependenceRelation::Independent => "independent",
        DependenceRelation::Shared => "shared",
        DependenceRelation::Derived => "derived",
        DependenceRelation::Unknown => "unknown",
    }
}

fn oracle_identity(reference: &OracleReference) -> &str {
    &reference.value.identity
}

fn observation_identity(reference: &ObservationReference) -> &str {
    &reference.value.identity
}

fn value<T: Serialize + ?Sized>(value: &T) -> Result<Value, (ProjectionRefusalCode, String)> {
    serde_json::to_value(value)
        .map_err(|_| affected(ProjectionRefusalCode::MissingContext, "targetContext"))
}

fn values<T: Serialize>(values: &[T]) -> Result<Value, (ProjectionRefusalCode, String)> {
    value(values)
}

fn verify_projection(
    source: &ValidatedResult,
    context: &ProjectionContext,
    candidate: &Value,
) -> Result<(), (ProjectionRefusalCode, String)> {
    let expected = [
        ("/execution/status", map_execution(source.execution())),
        ("/logical/outcome", map_truth(source)),
        ("/activation/status", map_activation(source.activation())),
        (
            "/adequacy/status",
            map_adequacy(source.observation_adequacy()),
        ),
    ];
    for (pointer, expected) in expected {
        if candidate.pointer(pointer).and_then(Value::as_str) != Some(expected) {
            return Err(affected(ProjectionRefusalCode::FindingLoss, pointer));
        }
    }
    if candidate["assumptions"].as_array().map(Vec::len) != Some(source.assumptions().len())
        || candidate["oracleContributions"].as_array().map(Vec::len)
            != Some(source.decision_support().len())
        || candidate["bindingRefs"] != value(&context.binding_refs)?
        || candidate["nativeArtifacts"].as_array().map(Vec::len) != Some(1)
        || candidate.get("sufficiencyDecision").is_some()
    {
        return Err(affected(
            ProjectionRefusalCode::FindingLoss,
            source.result_id(),
        ));
    }
    Ok(())
}

fn map_target_error(
    code: VerificationErrorCode,
    identity: &str,
) -> (ProjectionRefusalCode, String) {
    let mapped = match code {
        VerificationErrorCode::UnsupportedContractVersion
        | VerificationErrorCode::UnsupportedReferenceVersion
        | VerificationErrorCode::UnknownRequiredFeature => {
            ProjectionRefusalCode::UnsupportedTargetVersion
        }
        VerificationErrorCode::IdentityMismatch
        | VerificationErrorCode::IdentityContentConflict => ProjectionRefusalCode::IdentityMismatch,
        VerificationErrorCode::InvalidInput
        | VerificationErrorCode::InvalidTimestamp
        | VerificationErrorCode::InvalidTemporalOrder
        | VerificationErrorCode::InvalidJson
        | VerificationErrorCode::SchemaViolation
        | VerificationErrorCode::InvalidWire => ProjectionRefusalCode::MissingContext,
        VerificationErrorCode::JsonDocumentTooLarge
        | VerificationErrorCode::JsonNestingTooDeep
        | VerificationErrorCode::JsonArrayTooLarge
        | VerificationErrorCode::UnknownSchemaDefinition
        | VerificationErrorCode::CanonicalizationFailed
        | VerificationErrorCode::DuplicateTaskId
        | VerificationErrorCode::UnknownTaskDependency
        | VerificationErrorCode::TaskDependencyCycle
        | VerificationErrorCode::DuplicateTechniqueId
        | VerificationErrorCode::DuplicatePropertyId
        | VerificationErrorCode::UnknownProperty
        | VerificationErrorCode::AmbiguousCapability
        | VerificationErrorCode::DuplicateResultId
        | VerificationErrorCode::DuplicateOracleContributionId
        | VerificationErrorCode::InvalidOracleLineage
        | VerificationErrorCode::Stale
        | VerificationErrorCode::MalformedSourceMap
        | VerificationErrorCode::UnsupportedCanonicalization
        | VerificationErrorCode::AmbiguousEquivalence
        | VerificationErrorCode::IoFailure
        | VerificationErrorCode::UnhandledFixture
        | VerificationErrorCode::CatalogParityMismatch
        | VerificationErrorCode::InvalidExecutionDeadline
        | VerificationErrorCode::InvalidExpectedExitStatus
        | VerificationErrorCode::InvalidExecutablePath
        | VerificationErrorCode::ExecutionTimeout
        | VerificationErrorCode::OutputLimitExceeded
        | VerificationErrorCode::ProcessSignaled
        | VerificationErrorCode::UnexpectedExitStatus
        | VerificationErrorCode::InvalidProcessOutput
        | VerificationErrorCode::MalformedMethodResponse
        | VerificationErrorCode::InvalidMeasurementSeries
        | VerificationErrorCode::InvalidCorpusManifest
        | VerificationErrorCode::InvalidQualificationOracleManifest
        | VerificationErrorCode::InvalidQualificationOracleBinding
        | VerificationErrorCode::InvalidQualificationTechniqueManifest
        | VerificationErrorCode::InvalidQualificationCase
        | VerificationErrorCode::QualificationTechniqueUnavailable
        | VerificationErrorCode::DuplicateDisposition
        | VerificationErrorCode::ForeignDisposition
        | VerificationErrorCode::UnsupportedDisposition
        | VerificationErrorCode::InvalidReportDigest
        | VerificationErrorCode::UnsupportedExecutionProfile
        | VerificationErrorCode::UnexpectedProcessDescendant
        | VerificationErrorCode::ProcessContainmentFailure
        | VerificationErrorCode::InvalidModelDomain
        | VerificationErrorCode::UnsatisfiableModelDomain
        | VerificationErrorCode::ModelConstructionBoundExceeded
        | VerificationErrorCode::ModelCaseBoundExceeded => ProjectionRefusalCode::FindingLoss,
    };
    affected(mapped, identity)
}

fn affected(code: ProjectionRefusalCode, identity: &str) -> (ProjectionRefusalCode, String) {
    (code, identity.to_owned())
}

/// FR-283 projection refusal codes, in requirement order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProjectionRefusalCode {
    /// Source strict reading or source digest validation failed.
    InvalidSourceResult,
    /// The selected source revision is unknown.
    UnsupportedSourceVersion,
    /// The selected target revision or feature is unknown.
    UnsupportedTargetVersion,
    /// An exact source, context, reference, or derived identity disagrees.
    IdentityMismatch,
    /// A valid selected source state has no target mapping.
    UnrepresentedSourceState,
    /// Required exact target context is absent.
    MissingContext,
    /// Projection would omit an applicable independent finding.
    FindingLoss,
}

impl ProjectionRefusalCode {
    /// Exact FR-283 spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidSourceResult => "invalid-source-result",
            Self::UnsupportedSourceVersion => "unsupported-source-version",
            Self::UnsupportedTargetVersion => "unsupported-target-version",
            Self::IdentityMismatch => "identity-mismatch",
            Self::UnrepresentedSourceState => "unrepresented-source-state",
            Self::MissingContext => "missing-context",
            Self::FindingLoss => "finding-loss",
        }
    }

    const fn entry(self) -> &'static CodeEntry {
        match self {
            Self::InvalidSourceResult => &INVALID_SOURCE_RESULT,
            Self::UnsupportedSourceVersion => &UNSUPPORTED_SOURCE_VERSION,
            Self::UnsupportedTargetVersion => &UNSUPPORTED_TARGET_VERSION,
            Self::IdentityMismatch => &IDENTITY_MISMATCH,
            Self::UnrepresentedSourceState => &UNREPRESENTED_SOURCE_STATE,
            Self::MissingContext => &MISSING_CONTEXT,
            Self::FindingLoss => &FINDING_LOSS,
        }
    }
}

fn projection_refusal(code: ProjectionRefusalCode, identity: &[u8]) -> ProtocolRefusal {
    let refusal_code = RefusalCode::of(code.entry());
    let retained = RetainedBytes::new(identity);
    let cause = match code {
        ProjectionRefusalCode::InvalidSourceResult | ProjectionRefusalCode::MissingContext => {
            ProtocolCause::IncompleteEvidence { evidence: retained }
        }
        ProjectionRefusalCode::UnsupportedSourceVersion
        | ProjectionRefusalCode::UnsupportedTargetVersion => ProtocolCause::UnsupportedCapability {
            capability: retained,
        },
        ProjectionRefusalCode::IdentityMismatch => ProtocolCause::IdentityMismatch {
            expected: RetainedBytes::new(code.as_str().as_bytes()),
            observed: retained,
        },
        ProjectionRefusalCode::UnrepresentedSourceState => ProtocolCause::LogicalViolation {
            obligation: retained,
        },
        ProjectionRefusalCode::FindingLoss => {
            ProtocolCause::ProjectionFindingLoss { finding: retained }
        }
    };
    ProtocolRefusal::builder(
        refusal_code,
        CatalogSelection::local(),
        ProtocolStage::Projection,
        RefusalAxis::from_registered_code(refusal_code),
        cause.into(),
        ResourceUsage::protocol("quire.protocol/projection-usage/1", &[]),
    )
    .expect("projection codes belong to the projection stage")
    .prerequisite(Prerequisite::new(
        ProtocolStage::ResultRead,
        PrerequisiteState::Established,
    ))
    .unavailable_location(Unavailability::NotApplicableToStage)
    .identity(AffectedIdentity::retained(identity))
    .emitting_nothing()
}

macro_rules! projection_code {
    ($name:ident, $code:literal, $retry:expr, $summary:literal) => {
        const $name: CodeEntry = CodeEntry {
            provenance: Provenance::SpecNamed,
            code: $code,
            domain: RefusalDomain::Projection,
            retry: $retry,
            summary: $summary,
        };
    };
}

projection_code!(
    INVALID_SOURCE_RESULT,
    "invalid-source-result",
    RetryDecision::Never,
    "the canonical source failed strict reading or source digest validation"
);
projection_code!(
    UNSUPPORTED_SOURCE_VERSION,
    "unsupported-source-version",
    RetryDecision::AfterSelectionChange,
    "the selected canonical-result revision is unsupported"
);
projection_code!(
    UNSUPPORTED_TARGET_VERSION,
    "unsupported-target-version",
    RetryDecision::AfterSelectionChange,
    "the selected E02 revision or required feature is unsupported"
);
projection_code!(
    IDENTITY_MISMATCH,
    "identity-mismatch",
    RetryDecision::Never,
    "an exact source, target-context, reference, or derived identity differs"
);
projection_code!(
    UNREPRESENTED_SOURCE_STATE,
    "unrepresented-source-state",
    RetryDecision::Never,
    "a valid selected source state has no mapping in the selected target"
);
projection_code!(
    MISSING_CONTEXT,
    "missing-context",
    RetryDecision::AfterSelectionChange,
    "required exact target execution context is absent"
);
projection_code!(
    FINDING_LOSS,
    "finding-loss",
    RetryDecision::Never,
    "projection would omit an applicable independent finding"
);

macro_rules! code_module {
    ($name:ident, $code:literal, $entry:ident) => {
        /// One FR-283 spec-named refusal-code registration.
        pub const $name: CodeModule = CodeModule {
            root: $code,
            owner: "Task-007",
            module_path: "quire_protocol::projection",
            entries: &[$entry],
        };
    };
}

code_module!(
    INVALID_SOURCE_RESULT_CODES,
    "invalid-source-result",
    INVALID_SOURCE_RESULT
);
code_module!(
    UNSUPPORTED_SOURCE_VERSION_CODES,
    "unsupported-source-version",
    UNSUPPORTED_SOURCE_VERSION
);
code_module!(
    UNSUPPORTED_TARGET_VERSION_CODES,
    "unsupported-target-version",
    UNSUPPORTED_TARGET_VERSION
);
code_module!(
    IDENTITY_MISMATCH_CODES,
    "identity-mismatch",
    IDENTITY_MISMATCH
);
code_module!(
    UNREPRESENTED_SOURCE_STATE_CODES,
    "unrepresented-source-state",
    UNREPRESENTED_SOURCE_STATE
);
code_module!(MISSING_CONTEXT_CODES, "missing-context", MISSING_CONTEXT);
code_module!(FINDING_LOSS_CODES, "finding-loss", FINDING_LOSS);

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    #[test]
    #[trace("TC-007", "FR-035-AC-6", "FR-035-AC-7")]
    fn tc_007_mapping_completeness_refusals_are_distinct_and_emit_nothing() {
        for (code, expected) in [
            (
                ProjectionRefusalCode::UnrepresentedSourceState,
                "unrepresented-source-state",
            ),
            (ProjectionRefusalCode::FindingLoss, "finding-loss"),
        ] {
            let refusal = projection_refusal(code, b"source-state:future-member");

            assert_eq!(refusal.code().as_str(), expected);
            assert_ne!(refusal.code().as_str(), "invalid-source-result");
            assert_eq!(refusal.stage(), ProtocolStage::Projection);
            assert_eq!(
                refusal.partial_output(),
                crate::refusal::PartialOutput::NoneEmitted
            );
        }

        for target_error in [
            VerificationErrorCode::InvalidModelDomain,
            VerificationErrorCode::UnsatisfiableModelDomain,
            VerificationErrorCode::ModelConstructionBoundExceeded,
            VerificationErrorCode::ModelCaseBoundExceeded,
        ] {
            assert_eq!(
                map_target_error(target_error, "source-state:model-domain"),
                (
                    ProjectionRefusalCode::FindingLoss,
                    "source-state:model-domain".to_owned()
                )
            );
        }
    }
}

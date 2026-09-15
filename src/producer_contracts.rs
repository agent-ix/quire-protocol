// SPDX-License-Identifier: AGPL-3.0-or-later
//! Request-bound resolution of the exact QObs and QSL contracts consumed by assessment.
//!
//! This module owns no producer schema or semantic copy. Its closed inventory
//! points at the owner crates' contract constants and binds their accepted git
//! revisions through the Protocol version-lock resolver. The opaque successful
//! value can authorize an assessment request but exposes no raw resolver
//! capability for retargeting.

use core::fmt;

use quire_observation::authority::{
    availability, capture, clock, closure, completeness, observation, population, position,
    progress,
};
use quire_spec_language::protocol_artifact::native_temporal::{request, result, v2};

use crate::bounds::inventory::Usage;
use crate::bounds::Clock;
use crate::outcome::OperationOutcome;
use crate::version_lock::{
    self, EntryKey, InterfaceSelection, ManifestEntry, ResolutionBudget, ResolutionRequestIdentity,
    ResolvedRequest, VersionLockManifest,
};

/// Accepted QObs revision that supplies every observation contract below.
pub const QOBS_REVISION: &str = "93bc565072fb05d1ecf29f15725b75c0a77378fb";
/// Accepted QSL revision that first supplies the native-temporal v1 contracts.
pub const QSL_NATIVE_TEMPORAL_REVISION: &str = "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07";
/// Accepted QSL revision compiled for the opaque-trigger v2 contracts.
pub const QSL_OPAQUE_TRIGGER_REVISION: &str = "0c2f13e261c0ffeaf89f0638931b573ec038e99b";

const QOBS_OWNER: &str = "agent-ix/quire-observation";
const QSL_OWNER: &str = "agent-ix/quire-spec-language";
const IMMUTABILITY_ASSERTION: &[u8] = b"owner asserts immutable and published";

/// Closed producer-contract inventory required by IT-002 assessment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProducerContract {
    /// Qualified observation record.
    ObservationRecord,
    /// Qualified population record.
    ObservationPopulation,
    /// Exact explicit-members selection used to qualify a population.
    ObservationExplicitMembers,
    /// Ordered observation-position ledger.
    ObservationPositionLedger,
    /// Clock selection and boundary.
    ObservationClockBinding,
    /// Immutable trigger/capture environment.
    ObservationCaptureEnvironment,
    /// Decision or surrounding progress assertion.
    ObservationProgressAssertion,
    /// Decision or surrounding closure assertion.
    ObservationClosureAssertion,
    /// Population completeness assertion.
    ObservationCompletenessAssertion,
    /// Required-result availability assertion.
    ObservationResultAvailability,
    /// Native-temporal v1 request used behind the v2 owner adapter.
    NativeTemporalRequestV1,
    /// Native-temporal v1 result used behind the v2 owner adapter.
    NativeTemporalResultV1,
    /// Opaque-trigger native-temporal request consumed by Protocol.
    NativeTemporalRequestV2,
    /// Opaque-trigger native-temporal result consumed by Protocol.
    NativeTemporalResultV2,
}

impl ProducerContract {
    /// Every required contract in stable declaration order.
    pub const ALL: [Self; 14] = [
        Self::ObservationRecord,
        Self::ObservationPopulation,
        Self::ObservationExplicitMembers,
        Self::ObservationPositionLedger,
        Self::ObservationClockBinding,
        Self::ObservationCaptureEnvironment,
        Self::ObservationProgressAssertion,
        Self::ObservationClosureAssertion,
        Self::ObservationCompletenessAssertion,
        Self::ObservationResultAvailability,
        Self::NativeTemporalRequestV1,
        Self::NativeTemporalResultV1,
        Self::NativeTemporalRequestV2,
        Self::NativeTemporalResultV2,
    ];

    /// Exact producer owner used in the componentwise version-lock key.
    #[must_use]
    pub const fn owner(self) -> &'static str {
        match self {
            Self::ObservationRecord
            | Self::ObservationPopulation
            | Self::ObservationExplicitMembers
            | Self::ObservationPositionLedger
            | Self::ObservationClockBinding
            | Self::ObservationCaptureEnvironment
            | Self::ObservationProgressAssertion
            | Self::ObservationClosureAssertion
            | Self::ObservationCompletenessAssertion
            | Self::ObservationResultAvailability => QOBS_OWNER,
            Self::NativeTemporalRequestV1
            | Self::NativeTemporalResultV1
            | Self::NativeTemporalRequestV2
            | Self::NativeTemporalResultV2 => QSL_OWNER,
        }
    }

    /// Exact owner contract/version label.
    #[must_use]
    pub const fn contract_version(self) -> &'static str {
        match self {
            Self::ObservationRecord => observation::CONTRACT,
            Self::ObservationPopulation => population::CONTRACT,
            Self::ObservationExplicitMembers => population::MEMBERSHIP_CONTRACT,
            Self::ObservationPositionLedger => position::CONTRACT,
            Self::ObservationClockBinding => clock::CONTRACT,
            Self::ObservationCaptureEnvironment => capture::CONTRACT,
            Self::ObservationProgressAssertion => progress::CONTRACT,
            Self::ObservationClosureAssertion => closure::CONTRACT,
            Self::ObservationCompletenessAssertion => completeness::CONTRACT,
            Self::ObservationResultAvailability => availability::CONTRACT,
            Self::NativeTemporalRequestV1 => request::CONTRACT,
            Self::NativeTemporalResultV1 => result::CONTRACT,
            Self::NativeTemporalRequestV2 => v2::REQUEST_CONTRACT,
            Self::NativeTemporalResultV2 => v2::RESULT_CONTRACT,
        }
    }

    /// Exact accepted git revision compiled into the corresponding adapter.
    #[must_use]
    pub const fn expected_revision(self) -> &'static str {
        match self {
            Self::ObservationRecord
            | Self::ObservationPopulation
            | Self::ObservationExplicitMembers
            | Self::ObservationPositionLedger
            | Self::ObservationClockBinding
            | Self::ObservationCaptureEnvironment
            | Self::ObservationProgressAssertion
            | Self::ObservationClosureAssertion
            | Self::ObservationCompletenessAssertion
            | Self::ObservationResultAvailability => QOBS_REVISION,
            Self::NativeTemporalRequestV1 | Self::NativeTemporalResultV1 => {
                QSL_NATIVE_TEMPORAL_REVISION
            }
            Self::NativeTemporalRequestV2 | Self::NativeTemporalResultV2 => {
                QSL_OPAQUE_TRIGGER_REVISION
            }
        }
    }

    /// Componentwise version-lock key for this exact owner contract.
    #[must_use]
    pub fn key(self) -> EntryKey {
        EntryKey::new(self.owner().as_bytes(), self.contract_version().as_bytes())
            .expect("static producer-contract keys have nonempty components")
    }

    /// Exact pinned interface entry for a Protocol-authored manifest.
    #[must_use]
    pub fn manifest_entry(self) -> ManifestEntry {
        ManifestEntry::pinned_interface(
            self.key(),
            Some(self.expected_revision().as_bytes()),
            Some(IMMUTABILITY_ASSERTION),
            None,
        )
    }
}

/// A nonempty assessment-resolution identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProducerResolutionRequest(Box<[u8]>);

impl ProducerResolutionRequest {
    /// Retain an exact nonempty componentwise assessment identity.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyProducerResolutionRequest`] when `identity` is empty.
    pub fn new(identity: &[u8]) -> Result<Self, EmptyProducerResolutionRequest> {
        if identity.is_empty() {
            Err(EmptyProducerResolutionRequest)
        } else {
            Ok(Self(identity.into()))
        }
    }

    /// Exact retained identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An assessment-resolution identity was empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyProducerResolutionRequest;

impl fmt::Display for EmptyProducerResolutionRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("producer resolution request identity must not be empty")
    }
}

impl std::error::Error for EmptyProducerResolutionRequest {}

/// One exact contract retained by a completed producer resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProducerContract {
    contract: ProducerContract,
    revision: Box<[u8]>,
}

impl ResolvedProducerContract {
    /// Closed contract member that was resolved.
    #[must_use]
    pub const fn contract(&self) -> ProducerContract {
        self.contract
    }

    /// Exact accepted revision bytes.
    #[must_use]
    pub fn revision(&self) -> &[u8] {
        &self.revision
    }
}

/// Opaque request-bound authorization for the complete IT-002 producer set.
pub struct ProducerResolution {
    request: ProducerResolutionRequest,
    manifest: Vec<ManifestEntry>,
    contracts: Vec<ResolvedProducerContract>,
    usage: Usage,
    resolved: ResolvedRequest,
}

/// Durable attribution ingredients retained by a completed assessment.
///
/// This is not the final FR-017 scenario evidence record. It is the exact
/// request, resolved manifest population, selected contracts and resolver
/// usage that Task-018 can compose with the operation outcome and trace binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerResolutionEvidence {
    request: ProducerResolutionRequest,
    manifest: Vec<ManifestEntry>,
    contracts: Vec<ResolvedProducerContract>,
    usage: Usage,
}

impl ProducerResolutionEvidence {
    /// Exact assessment request the retained resolution authorized.
    #[must_use]
    pub const fn request(&self) -> &ProducerResolutionRequest {
        &self.request
    }

    /// Complete resolved manifest state in deterministic key order.
    #[must_use]
    pub fn manifest_entries(&self) -> &[ManifestEntry] {
        &self.manifest
    }

    /// Complete selected producer-contract population in declaration order.
    #[must_use]
    pub fn contracts(&self) -> &[ResolvedProducerContract] {
        &self.contracts
    }

    /// Exact Protocol resolution resource usage.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

impl fmt::Debug for ProducerResolution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProducerResolution")
            .field("request", &self.request)
            .field("manifest", &self.manifest)
            .field("contracts", &self.contracts)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

impl ProducerResolution {
    /// Exact assessment request this capability authorizes.
    #[must_use]
    pub const fn request(&self) -> &ProducerResolutionRequest {
        &self.request
    }

    /// Complete resolved manifest state in deterministic key order.
    #[must_use]
    pub fn manifest_entries(&self) -> &[ManifestEntry] {
        &self.manifest
    }

    /// Complete resolved producer-contract population in declaration order.
    #[must_use]
    pub fn contracts(&self) -> &[ResolvedProducerContract] {
        &self.contracts
    }

    /// Exact Protocol resolution resource usage.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    pub(crate) fn evidence(&self) -> ProducerResolutionEvidence {
        ProducerResolutionEvidence {
            request: self.request.clone(),
            manifest: self.manifest.clone(),
            contracts: self.contracts.clone(),
            usage: self.usage,
        }
    }

    pub(crate) fn authorize(&self, request: &ProducerResolutionRequest) -> OperationOutcome<()> {
        let private_request = ResolutionRequestIdentity::new(request.as_bytes())
            .expect("public request construction already rejected empty identity bytes");
        for contract in ProducerContract::ALL {
            let selected = InterfaceSelection::<ProducerContract>::new(
                contract.key(),
                contract.expected_revision().as_bytes(),
            );
            let capability = match self.resolved.interface(&selected) {
                OperationOutcome::Completed(capability) => capability,
                OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
                OperationOutcome::ResourceIncomplete(incomplete) => {
                    return OperationOutcome::ResourceIncomplete(incomplete)
                }
            };
            match capability.authorize(&private_request, &selected) {
                OperationOutcome::Completed(()) => {}
                OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
                OperationOutcome::ResourceIncomplete(incomplete) => {
                    return OperationOutcome::ResourceIncomplete(incomplete)
                }
            }
        }
        OperationOutcome::Completed(())
    }
}

/// Resolve and authorize the complete producer-contract inventory for one request.
#[must_use]
pub fn resolve_producer_contracts<C: Clock + 'static>(
    request: &ProducerResolutionRequest,
    manifest: &VersionLockManifest,
    budget: ResolutionBudget,
    clock: C,
) -> OperationOutcome<ProducerResolution> {
    let private_request = ResolutionRequestIdentity::new(request.as_bytes())
        .expect("public request construction already rejected empty identity bytes");
    let required = ProducerContract::ALL
        .iter()
        .copied()
        .map(ProducerContract::key)
        .collect::<Vec<_>>();
    let resolved =
        match version_lock::resolve(private_request, manifest, &required, &[], budget, clock) {
            OperationOutcome::Completed(resolved) => resolved,
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete)
            }
        };

    for contract in ProducerContract::ALL {
        let selected = InterfaceSelection::<ProducerContract>::new(
            contract.key(),
            contract.expected_revision().as_bytes(),
        );
        match resolved.interface(&selected) {
            OperationOutcome::Completed(_) => {}
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
    }

    let manifest_entries = resolved.entries().to_vec();
    let contracts = ProducerContract::ALL
        .iter()
        .copied()
        .map(|contract| {
            let revision = manifest_entries
                .iter()
                .find(|entry| entry.key() == &contract.key())
                .and_then(ManifestEntry::revision)
                .expect("completed resolution retained every pinned required revision");
            ResolvedProducerContract {
                contract,
                revision: revision.as_bytes().into(),
            }
        })
        .collect();
    let usage = resolved.usage();
    OperationOutcome::Completed(ProducerResolution {
        request: request.clone(),
        manifest: manifest_entries,
        contracts,
        usage,
        resolved,
    })
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Request-bound resolution of the exact QObs and QSL contracts consumed by assessment.
//!
//! This module owns no producer schema or semantic copy. Its closed inventory
//! points at the owner crates' contract constants and binds their accepted git
//! revisions through the Protocol version-lock resolver. The successful value
//! retains only immutable resolution evidence and exposes no raw resolver
//! capability for retargeting.

use core::fmt;

use quire_observation::authority::{
    activation, availability, bundle, capture, clock, closure, completeness, observation, partial,
    population, position, progress,
};
use quire_spec_language::protocol_artifact::{
    native_temporal::{request, result, v2 as temporal_v2},
    v2 as artifact_v2, v3 as artifact_v3,
};
use quire_spec_language::ByteDigest;

use crate::bounds::inventory::ResourceIncomplete;
use crate::bounds::inventory::Usage;
use crate::bounds::Clock;
use crate::outcome::OperationOutcome;
use crate::refusal::ProtocolRefusal;
use crate::version_lock::{
    self, EntryKey, InterfaceRequirement, ManifestEntry, ResolutionBudget,
    ResolutionRequestIdentity, VersionLockManifest,
};

/// Accepted QObs revision that supplies every observation contract below.
///
/// This exact commit is the accepted merge of QObs pull request 27. A later
/// owner revision requires an explicit repin and renewed compatibility review.
pub const QOBS_REVISION: &str = "924006300f45b38483be1cbdf99b68f899b7d368";
/// Accepted QSL revision that first supplies the native-temporal v1 contracts.
pub const QSL_NATIVE_TEMPORAL_REVISION: &str = "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07";
/// Accepted QSL revision compiled for the opaque-trigger v2 contracts.
pub const QSL_OPAQUE_TRIGGER_REVISION: &str = "0c2f13e261c0ffeaf89f0638931b573ec038e99b";

const QOBS_OWNER: &str = "agent-ix/quire-observation";
const QSL_OWNER: &str = "agent-ix/quire-spec-language";
const IMMUTABILITY_ASSERTION: &[u8] = b"owner asserts immutable and published";

/// One exact owner schema selection retained beside a producer contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerSchema {
    bytes: &'static [u8],
    sha256: &'static str,
}

impl OwnerSchema {
    /// Exact schema bytes exported by the owner crate.
    #[must_use]
    pub const fn bytes(self) -> &'static [u8] {
        self.bytes
    }

    /// Independently selected lowercase hexadecimal SHA-256.
    #[must_use]
    pub const fn sha256(self) -> &'static str {
        self.sha256
    }
}

macro_rules! define_producer_contracts {
    ($(
        $(#[$variant_meta:meta])*
        $variant:ident {
            owner: $owner:expr,
            contract: $contract:expr,
            revision: $revision:expr,
            schema: $schema:expr,
        }
    ),+ $(,)?) => {
        /// Closed producer-contract inventory required by IT-002 assessment.
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum ProducerContract {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl ProducerContract {
            /// Every required contract in stable declaration order.
            pub const ALL: [Self; [$(stringify!($variant)),+].len()] = [
                $(Self::$variant),+
            ];

            /// Exact producer owner used in the componentwise version-lock key.
            #[must_use]
            pub const fn owner(self) -> &'static str {
                match self {
                    $(Self::$variant => $owner),+
                }
            }

            /// Exact owner contract/version label.
            #[must_use]
            pub const fn contract_version(self) -> &'static str {
                match self {
                    $(Self::$variant => $contract),+
                }
            }

            /// Exact accepted git revision compiled into the corresponding adapter.
            #[must_use]
            pub const fn expected_revision(self) -> &'static str {
                match self {
                    $(Self::$variant => $revision),+
                }
            }

            /// Exact owner-published schema bytes and independently selected SHA-256.
            ///
            /// QSL native-temporal entries are Rust interfaces with no schema selection
            /// in this inventory, so they return `None` rather than a synthesized value.
            #[must_use]
            pub const fn owner_schema(self) -> Option<OwnerSchema> {
                match self {
                    $(Self::$variant => $schema),+
                }
            }
        }
    };
}

define_producer_contracts! {
    /// Qualified observation record.
    ObservationRecord {
        owner: QOBS_OWNER,
        contract: observation::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: observation::SCHEMA_BYTES, sha256: "8737683a5971aabcb39bbc5f0842fa1d7c1db5c8f283b8b2544c6364c682f882" }),
    },
    /// Qualified population record.
    ObservationPopulation {
        owner: QOBS_OWNER,
        contract: population::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: population::SCHEMA_BYTES, sha256: "bceba1a2a69d150af05a7f7bea52769560849fd24a8582cb35b0937eee263c01" }),
    },
    /// Exact explicit-members selection used to qualify a population.
    ObservationExplicitMembers {
        owner: QOBS_OWNER,
        contract: population::MEMBERSHIP_CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: population::MEMBERSHIP_SCHEMA_BYTES, sha256: "408c9d2908c3660d657cea574f52d8531e9ab749ec78c8d058fe5f163df23cce" }),
    },
    /// Ordered observation-position ledger.
    ObservationPositionLedger {
        owner: QOBS_OWNER,
        contract: position::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: position::SCHEMA_BYTES, sha256: "aac2fd7fc1b129e24afec397900647f3fe8977341b9907662a3852ac5430211a" }),
    },
    /// Clock selection and boundary.
    ObservationClockBinding {
        owner: QOBS_OWNER,
        contract: clock::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: clock::SCHEMA_BYTES, sha256: "c9a7b154a2c2d775ba71ba001842e6f0595e123c07b23a2bb37215c1f05de38d" }),
    },
    /// Qualified partial observation fact.
    ObservationPartialFact {
        owner: QOBS_OWNER,
        contract: partial::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: partial::SCHEMA_BYTES, sha256: "9dc068376a7db5a7509810910c36e2efff6b9c3701d83c474701ec572f13f23c" }),
    },
    /// Immutable trigger/capture environment.
    ObservationCaptureEnvironment {
        owner: QOBS_OWNER,
        contract: capture::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: capture::SCHEMA_BYTES, sha256: "6d9260d3c4de65b5c3304debdb9baa7816aaea55949839924182d1db4c24c516" }),
    },
    /// Decision or surrounding progress assertion.
    ObservationProgressAssertion {
        owner: QOBS_OWNER,
        contract: progress::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: progress::SCHEMA_BYTES, sha256: "91223fb982b68d4fb7c66b3737370429bd4841321f5b561ebc6c1e9691bd7f1c" }),
    },
    /// Strict-read activation-scope authority.
    ObservationActivationAuthority {
        owner: QOBS_OWNER,
        contract: activation::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: activation::SCHEMA_BYTES, sha256: "c17ad8b3ec10c48e535bc3a762738b979206c01af20c31775d8a9a9e0a97fab6" }),
    },
    /// Decision or surrounding closure assertion.
    ObservationClosureAssertion {
        owner: QOBS_OWNER,
        contract: closure::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: closure::SCHEMA_BYTES, sha256: "8638fb9f4dd5a26d44ae9975388fc734670992914422e23c27eaae82149245f4" }),
    },
    /// Population completeness assertion.
    ObservationCompletenessAssertion {
        owner: QOBS_OWNER,
        contract: completeness::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: completeness::SCHEMA_BYTES, sha256: "b7c3c48d5f907cfb5bd7b4450e009b6c95f7d558095ece8b81d2ae1d5821bcf0" }),
    },
    /// Required-result availability assertion.
    ObservationResultAvailability {
        owner: QOBS_OWNER,
        contract: availability::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: availability::SCHEMA_BYTES, sha256: "2e6c00d6dc94a00b0859b346dc12a14c7142735a565e35d15b85895d311e8416" }),
    },
    /// Canonical observation-authority bundle version 1.
    ObservationAuthorityBundleV1 {
        owner: QOBS_OWNER,
        contract: bundle::CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: bundle::SCHEMA_BYTES, sha256: "9614d8077bd1e667507b2107eab08e0f7934337115ce3e10c8c55b5d5538cdab" }),
    },
    /// Canonical observation-authority bundle version 2.
    ObservationAuthorityBundleV2 {
        owner: QOBS_OWNER,
        contract: bundle::V2_CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: bundle::V2_SCHEMA_BYTES, sha256: "f92335c5e5e1e3434f1c2484557ca1f512c22b496eb97319c94fcac3491f116a" }),
    },
    /// Bounded observation-authority lineage.
    ObservationAuthorityLineage {
        owner: QOBS_OWNER,
        contract: bundle::LINEAGE_CONTRACT,
        revision: QOBS_REVISION,
        schema: Some(OwnerSchema { bytes: bundle::LINEAGE_SCHEMA_BYTES, sha256: "d58c70516e53bde40ab2d276df143fffb05740348b98d09a71e89857a82a22ef" }),
    },
    /// Native-temporal v1 request used behind the v2 owner adapter.
    NativeTemporalRequestV1 {
        owner: QSL_OWNER,
        contract: request::CONTRACT,
        revision: QSL_NATIVE_TEMPORAL_REVISION,
        schema: None,
    },
    /// Native-temporal v1 result used behind the v2 owner adapter.
    NativeTemporalResultV1 {
        owner: QSL_OWNER,
        contract: result::CONTRACT,
        revision: QSL_NATIVE_TEMPORAL_REVISION,
        schema: None,
    },
    /// Opaque-trigger native-temporal request consumed by Protocol.
    NativeTemporalRequestV2 {
        owner: QSL_OWNER,
        contract: temporal_v2::REQUEST_CONTRACT,
        revision: QSL_OPAQUE_TRIGGER_REVISION,
        schema: None,
    },
    /// Opaque-trigger native-temporal result consumed by Protocol.
    NativeTemporalResultV2 {
        owner: QSL_OWNER,
        contract: temporal_v2::RESULT_CONTRACT,
        revision: QSL_OPAQUE_TRIGGER_REVISION,
        schema: None,
    },
    /// Strict compiled-protocol v2 package consumed by the linked assessment.
    CompiledProtocolV2 {
        owner: QSL_OWNER,
        contract: artifact_v2::WIRE,
        revision: QSL_OPAQUE_TRIGGER_REVISION,
        schema: None,
    },
    /// Strict compiled-protocol v3 activation mapping consumed by FR-300.
    ActivationMappingV3 {
        owner: QSL_OWNER,
        contract: artifact_v3::WIRE,
        revision: QSL_OPAQUE_TRIGGER_REVISION,
        schema: None,
    },
}

impl ProducerContract {
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

/// QObs capabilities for which Protocol intentionally has no consumer adapter.
///
/// These are not producer contracts and therefore mint no version-lock entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnsupportedObservationCapability {
    /// QObs repair-plan derivation and coordination.
    Repair,
    /// QObs bounded aggregate query.
    AggregateQuery,
}

impl UnsupportedObservationCapability {
    /// Complete stable unsupported-capability inventory.
    pub const ALL: [Self; 2] = [Self::Repair, Self::AggregateQuery];
}

/// A fixed-size, independently selected assessment-resolution identity.
///
/// The identity binds a resolution to the selected compiled package and static
/// binding artifact and, for temporal assessments, to one exact QSL request
/// document. Callers cannot supply arbitrary or unbounded identity bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProducerResolutionRequest(Box<[u8]>);

impl ProducerResolutionRequest {
    /// Select one non-temporal assessment request from fixed-size owner
    /// digests known before producer values are loaded.
    #[must_use]
    pub fn for_assessment(package: ByteDigest, static_binding: [u8; 32]) -> Self {
        Self::encode(package, static_binding, None)
    }

    /// Select one temporal assessment request from fixed-size owner digests
    /// known before producer values are loaded.
    #[must_use]
    pub fn for_temporal_assessment(
        package: ByteDigest,
        static_binding: [u8; 32],
        temporal_request: ByteDigest,
    ) -> Self {
        Self::encode(package, static_binding, Some(temporal_request))
    }

    fn encode(
        package: ByteDigest,
        static_binding: [u8; 32],
        temporal_request: Option<ByteDigest>,
    ) -> Self {
        let package = format!("{package:x}");
        let temporal = temporal_request.map(|digest| format!("{digest:x}"));
        let mut identity = Vec::with_capacity(1 + 64 + 32 + 64);
        identity.push(u8::from(temporal.is_some()));
        identity.extend_from_slice(package.as_bytes());
        identity.extend_from_slice(&static_binding);
        if let Some(temporal) = temporal {
            identity.extend_from_slice(temporal.as_bytes());
        }
        Self(identity.into())
    }

    /// Exact retained identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One exact contract retained by a completed producer resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedProducerContract {
    contract: ProducerContract,
    revision: &'static [u8],
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
        self.revision
    }
}

/// Immutable request-bound resolution evidence for the complete IT-002 producer set.
pub struct ProducerResolution {
    request: ProducerResolutionRequest,
    contracts: [ResolvedProducerContract; ProducerContract::ALL.len()],
    resolved: version_lock::ResolvedRequest,
}

/// Whether a producer-dependent operation may execute.
///
/// The executable value is constructed only after the complete manifest has
/// resolved. A producer refusal is reported as unexecutable rather than as an
/// assessment result, and resource exhaustion remains independently typed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProducerReadiness<T> {
    /// Every required producer contract resolved before producer access.
    Executable(T),
    /// Producer prerequisites did not resolve, so no assessment was run.
    Unexecutable(ProtocolRefusal),
    /// Resolution stopped at a caller-declared resource boundary.
    ResourceIncomplete(ResourceIncomplete),
}

impl fmt::Debug for ProducerResolution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProducerResolution")
            .field("request", &self.request)
            .field("manifest", &self.resolved.entries())
            .field("contracts", &self.contracts)
            .field("usage", &self.resolved.usage())
            .finish_non_exhaustive()
    }
}

impl ProducerResolution {
    /// Exact assessment request whose producer contracts were resolved.
    #[must_use]
    pub const fn request(&self) -> &ProducerResolutionRequest {
        &self.request
    }

    /// Complete resolved manifest state in deterministic key order.
    #[must_use]
    pub fn manifest_entries(&self) -> &[ManifestEntry] {
        self.resolved.entries()
    }

    /// Complete resolved producer-contract population in declaration order.
    #[must_use]
    pub fn contracts(&self) -> &[ResolvedProducerContract] {
        &self.contracts
    }

    /// Exact Protocol resolution resource usage.
    #[must_use]
    pub fn usage(&self) -> Usage {
        self.resolved.usage()
    }

    pub(crate) fn authorize(&self, actual: &ProducerResolutionRequest) -> OperationOutcome<()> {
        let actual = ResolutionRequestIdentity::new(actual.as_bytes())
            .expect("fixed producer request identities are nonempty");
        self.resolved.authorize_request(&actual)
    }
}

/// Resolve the complete producer-contract inventory for one request.
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
        .map(|contract| {
            InterfaceRequirement::new(contract.key(), contract.expected_revision().as_bytes())
        })
        .collect::<Vec<_>>();
    let resolved =
        match version_lock::resolve_interfaces(private_request, manifest, &required, budget, clock)
        {
            OperationOutcome::Completed(resolved) => resolved,
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete)
            }
        };

    let contracts = ProducerContract::ALL.map(|contract| ResolvedProducerContract {
        contract,
        revision: contract.expected_revision().as_bytes(),
    });
    OperationOutcome::Completed(ProducerResolution {
        request: request.clone(),
        contracts,
        resolved,
    })
}

/// Resolve the complete producer inventory before permitting producer access.
///
/// `on_resolved` is never invoked for an absent, unresolved, unpinned, or
/// repointed manifest. This is the staging primitive used by the public
/// assessment boundary, so assessment cannot reinterpret prerequisite failure
/// as a claim result.
#[must_use]
pub(crate) fn resolve_before_producer_read<C, T>(
    request: &ProducerResolutionRequest,
    manifest: &VersionLockManifest,
    budget: ResolutionBudget,
    clock: C,
    on_resolved: impl FnOnce(ProducerResolution) -> T,
) -> ProducerReadiness<T>
where
    C: Clock + 'static,
{
    match resolve_producer_contracts(request, manifest, budget, clock) {
        OperationOutcome::Completed(resolution) => {
            ProducerReadiness::Executable(on_resolved(resolution))
        }
        OperationOutcome::Refused(refusal) => ProducerReadiness::Unexecutable(refusal),
        OperationOutcome::ResourceIncomplete(incomplete) => {
            ProducerReadiness::ResourceIncomplete(incomplete)
        }
    }
}

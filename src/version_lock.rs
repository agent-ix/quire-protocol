// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Task-013 establishes private resolver capabilities consumed by Tasks 009-011"
    )
)]
//! Task-013: deterministic, bounded, request-bound external-contract resolution.
//!
//! Resolution consumes only caller-provided values. It performs no discovery,
//! I/O, VCS access, process execution, or network access. Artifact bytes are
//! digested exactly as offered and retained in a private capability; interface
//! capabilities are minted only after an adapter-owned selection matches the
//! manifest entry exactly.

use core::{fmt, marker::PhantomData};
use std::time::Duration;

use crate::bounds::inventory::{
    self, AccountingFault, Budget, BudgetDimension, BudgetError, ChargeError, ResourceAccountant,
    SourceSite, Usage,
};
use crate::bounds::{Clock, Phase, SiteId, SiteKind, UnitLabel};
use crate::outcome::OperationOutcome;
use crate::refusal::catalog::{self, CatalogSelection, RetainedBytes};
use crate::refusal::{
    AffectedIdentity, ProtocolCause, ProtocolRefusal, ProtocolStage, RefusalAxis, ResourceUsage,
    Unavailability,
};
use crate::ByteDigest;

#[cfg(test)]
mod tests;

const MANIFEST_UNIT: UnitLabel = UnitLabel::new("version-lock-manifest-validation");
const DELIVERY_UNIT: UnitLabel = UnitLabel::new("version-lock-delivery-validation");
const DIGEST_UNIT: UnitLabel = UnitLabel::new("version-lock-digest");
const REQUEST_SELECTION_UNIT: UnitLabel = UnitLabel::new("producer-request-selection-verification");

pub(crate) const RESOLUTION_ENTRY_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("resolution/entry/validate"),
    "version_lock::resolve",
    BudgetDimension::ResolutionEntryCount,
    SiteKind::Iteration,
);
pub(crate) const RESOLUTION_BYTES_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("resolution/bytes/digest"),
    "version_lock::resolve",
    BudgetDimension::ResolutionBytesDigested,
    SiteKind::Iteration,
);
pub(crate) const RESOLUTION_DEADLINE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("resolution/unit/deadline"),
    "version_lock::resolve",
    BudgetDimension::ResolutionDeadline,
    SiteKind::Iteration,
);

/// Bytes retained without parsing, normalization, or re-encoding.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ExactBytes(Box<[u8]>);

impl ExactBytes {
    /// Retain `bytes` exactly.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }

    /// The retained bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A version-lock key must name both its owner and contract version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKeyError {
    /// The owner component was empty.
    EmptyOwner,
    /// The contract-version component was empty.
    EmptyContractVersion,
}

impl fmt::Display for EntryKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyOwner => "version-lock owner is empty",
            Self::EmptyContractVersion => "version-lock contract version is empty",
        })
    }
}

impl std::error::Error for EntryKeyError {}

/// The componentwise `(owner, contract version)` identity of one entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct EntryKey {
    owner: ExactBytes,
    contract_version: ExactBytes,
}

impl EntryKey {
    /// Build a key, rejecting either absent identity component.
    ///
    /// # Errors
    ///
    /// [`EntryKeyError`] when either component is empty.
    pub fn new(owner: &[u8], contract_version: &[u8]) -> Result<Self, EntryKeyError> {
        if owner.is_empty() {
            return Err(EntryKeyError::EmptyOwner);
        }
        if contract_version.is_empty() {
            return Err(EntryKeyError::EmptyContractVersion);
        }
        Ok(Self {
            owner: ExactBytes::new(owner),
            contract_version: ExactBytes::new(contract_version),
        })
    }

    /// Exact owner bytes.
    #[must_use]
    pub const fn owner(&self) -> &ExactBytes {
        &self.owner
    }

    /// Exact contract-version bytes.
    #[must_use]
    pub const fn contract_version(&self) -> &ExactBytes {
        &self.contract_version
    }

    fn identity_bytes(&self) -> Vec<u8> {
        let mut identity = Vec::with_capacity(
            32usize
                .saturating_add(self.owner.as_bytes().len())
                .saturating_add(self.contract_version.as_bytes().len()),
        );
        identity.extend_from_slice(&(self.owner.as_bytes().len() as u128).to_be_bytes());
        identity.extend_from_slice(self.owner.as_bytes());
        identity.extend_from_slice(&(self.contract_version.as_bytes().len() as u128).to_be_bytes());
        identity.extend_from_slice(self.contract_version.as_bytes());
        identity
    }
}

/// The only two states a version-lock entry may carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EntryStatus {
    /// An owner has asserted an immutable published revision.
    Pinned,
    /// The entry cannot yet be pinned, for a retained reason.
    Unresolved,
}

/// Whether an entry names bytes or only an interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EntryKind {
    /// A delivered byte artifact, selected by member name and digest.
    Artifact,
    /// A Rust-callable interface with no delivered bytes and no digest.
    Interface,
}

#[derive(Clone, Debug)]
enum EntrySubject {
    Artifact {
        member: ExactBytes,
        digest: ByteDigest,
    },
    UnresolvedArtifact {
        member: ExactBytes,
    },
    Interface {
        digest: Option<ByteDigest>,
    },
}

/// One Protocol-authored manifest entry.
///
/// Equality is entry identity: exact `(owner, contract version)` only. Revision,
/// assertion, status and subject remain independently inspectable state and are
/// never folded into or substituted for that identity.
#[derive(Clone, Debug)]
pub struct ManifestEntry {
    key: EntryKey,
    status: EntryStatus,
    revision: Option<ExactBytes>,
    immutability_assertion: Option<ExactBytes>,
    unresolved_reason: Option<ExactBytes>,
    subject: EntrySubject,
}

impl PartialEq for ManifestEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for ManifestEntry {}

impl ManifestEntry {
    /// Declare a pinned artifact entry.
    #[must_use]
    pub fn pinned_artifact(
        key: EntryKey,
        revision: Option<&[u8]>,
        immutability_assertion: Option<&[u8]>,
        member: &[u8],
        digest: ByteDigest,
    ) -> Self {
        Self {
            key,
            status: EntryStatus::Pinned,
            revision: revision.map(ExactBytes::new),
            immutability_assertion: immutability_assertion.map(ExactBytes::new),
            unresolved_reason: None,
            subject: EntrySubject::Artifact {
                member: ExactBytes::new(member),
                digest,
            },
        }
    }

    /// Declare a pinned interface entry.
    #[must_use]
    pub fn pinned_interface(
        key: EntryKey,
        revision: Option<&[u8]>,
        immutability_assertion: Option<&[u8]>,
        digest: Option<ByteDigest>,
    ) -> Self {
        Self {
            key,
            status: EntryStatus::Pinned,
            revision: revision.map(ExactBytes::new),
            immutability_assertion: immutability_assertion.map(ExactBytes::new),
            unresolved_reason: None,
            subject: EntrySubject::Interface { digest },
        }
    }

    /// Declare an unresolved artifact entry with its exact reason.
    #[must_use]
    pub fn unresolved_artifact(key: EntryKey, member: &[u8], reason: &[u8]) -> Self {
        Self {
            key,
            status: EntryStatus::Unresolved,
            revision: None,
            immutability_assertion: None,
            unresolved_reason: Some(ExactBytes::new(reason)),
            subject: EntrySubject::UnresolvedArtifact {
                member: ExactBytes::new(member),
            },
        }
    }

    /// Declare an unresolved interface entry with its exact reason.
    #[must_use]
    pub fn unresolved_interface(key: EntryKey, reason: &[u8]) -> Self {
        Self {
            key,
            status: EntryStatus::Unresolved,
            revision: None,
            immutability_assertion: None,
            unresolved_reason: Some(ExactBytes::new(reason)),
            subject: EntrySubject::Interface { digest: None },
        }
    }

    /// The exact owner/contract-version identity.
    #[must_use]
    pub const fn key(&self) -> &EntryKey {
        &self.key
    }

    /// Whether the entry names artifact bytes or an independently linked interface.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        match &self.subject {
            EntrySubject::Artifact { .. } | EntrySubject::UnresolvedArtifact { .. } => {
                EntryKind::Artifact
            }
            EntrySubject::Interface { .. } => EntryKind::Interface,
        }
    }

    /// The entry's explicit pinned or unresolved state.
    #[must_use]
    pub const fn status(&self) -> EntryStatus {
        self.status
    }

    /// The exact owner-published revision, when pinned.
    #[must_use]
    pub const fn revision(&self) -> Option<&ExactBytes> {
        self.revision.as_ref()
    }

    /// The owner's exact immutability/publication assertion, when pinned.
    #[must_use]
    pub const fn immutability_assertion(&self) -> Option<&ExactBytes> {
        self.immutability_assertion.as_ref()
    }

    /// The exact unresolved reason, when unresolved.
    #[must_use]
    pub const fn unresolved_reason(&self) -> Option<&ExactBytes> {
        self.unresolved_reason.as_ref()
    }

    /// The selected delivery member for an artifact entry.
    #[must_use]
    pub const fn member(&self) -> Option<&ExactBytes> {
        match &self.subject {
            EntrySubject::Artifact { member, .. } | EntrySubject::UnresolvedArtifact { member } => {
                Some(member)
            }
            EntrySubject::Interface { .. } => None,
        }
    }

    /// The exact expected artifact digest; interface entries carry none.
    #[must_use]
    pub const fn digest(&self) -> Option<&ByteDigest> {
        match &self.subject {
            EntrySubject::Artifact { digest, .. } => Some(digest),
            EntrySubject::Interface { digest } => digest.as_ref(),
            EntrySubject::UnresolvedArtifact { .. } => None,
        }
    }
}

/// The Protocol-authored set of producer locks. Requiredness is absent.
#[derive(Clone, Debug, Default)]
pub struct VersionLockManifest {
    entries: Vec<ManifestEntry>,
}

impl VersionLockManifest {
    /// Retain the offered entries in their offered presentation order.
    #[must_use]
    pub const fn new(entries: Vec<ManifestEntry>) -> Self {
        Self { entries }
    }

    /// Offered entries. Resolution, not construction, validates the set.
    #[must_use]
    pub fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }
}

/// One named byte sequence offered by a producer delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactDelivery {
    member: ExactBytes,
    bytes: Box<[u8]>,
}

impl ArtifactDelivery {
    /// Retain one offered member name and its bytes exactly.
    #[must_use]
    pub fn new(member: &[u8], bytes: &[u8]) -> Self {
        Self {
            member: ExactBytes::new(member),
            bytes: bytes.into(),
        }
    }

    /// Exact offered member name.
    #[must_use]
    pub const fn member(&self) -> &ExactBytes {
        &self.member
    }

    /// Exact bytes offered for this member.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Caller grants for the three FR-009 resolution dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionBudget(Budget);

/// A caller supplied a resolution budget that cannot be represented exactly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionBudgetError {
    /// The deadline's exact nanosecond count does not fit the shared budget domain.
    DeadlineOutOfRange {
        /// Exact nanoseconds supplied by the caller.
        nanoseconds: u128,
    },
    /// One of the shared caller-grant invariants was not satisfied.
    InvalidGrant(BudgetError),
}

impl fmt::Display for ResolutionBudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeadlineOutOfRange { nanoseconds } => write!(
                formatter,
                "resolution deadline {nanoseconds}ns does not fit the budget domain"
            ),
            Self::InvalidGrant(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ResolutionBudgetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DeadlineOutOfRange { .. } => None,
            Self::InvalidGrant(error) => Some(error),
        }
    }
}

impl ResolutionBudget {
    /// Construct three explicit positive grants without truncating the deadline.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionBudgetError::DeadlineOutOfRange`] when the exact
    /// deadline does not fit the shared counter domain, or
    /// [`ResolutionBudgetError::InvalidGrant`] when any grant is zero or cannot
    /// be represented by the shared accountant.
    pub fn new(
        entries: usize,
        bytes: usize,
        deadline: Duration,
    ) -> Result<Self, ResolutionBudgetError> {
        let deadline_nanos_u128 = deadline.as_nanos();
        let deadline_nanos = usize::try_from(deadline_nanos_u128).map_err(|_| {
            ResolutionBudgetError::DeadlineOutOfRange {
                nanoseconds: deadline_nanos_u128,
            }
        })?;
        let shared = Budget::nothing_granted()
            .grant(BudgetDimension::ResolutionEntryCount, entries)
            .and_then(|budget| budget.grant(BudgetDimension::ResolutionBytesDigested, bytes))
            .and_then(|budget| budget.grant(BudgetDimension::ResolutionDeadline, deadline_nanos))
            .map_err(ResolutionBudgetError::InvalidGrant)?;
        Ok(Self(shared))
    }

    fn into_shared(self) -> Budget {
        self.0
    }
}

/// Exact identity of one resolution/request. Capabilities retain it unchanged.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub(crate) struct ResolutionRequestIdentity(ExactBytes);

impl ResolutionRequestIdentity {
    pub(crate) fn new(bytes: &[u8]) -> Result<Self, EmptyResolutionRequestIdentity> {
        if bytes.is_empty() {
            Err(EmptyResolutionRequestIdentity)
        } else {
            Ok(Self(ExactBytes::new(bytes)))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A non-empty resolution/request identity was required.
pub(crate) struct EmptyResolutionRequestIdentity;

impl fmt::Display for EmptyResolutionRequestIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("resolution request identity must not be empty")
    }
}

impl std::error::Error for EmptyResolutionRequestIdentity {}

/// Adapter-owned exact artifact selection. Construction is crate-private.
#[derive(Debug)]
pub(crate) struct ArtifactSelection<C> {
    key: EntryKey,
    revision: ExactBytes,
    contract: PhantomData<fn() -> C>,
}

impl<C> ArtifactSelection<C> {
    /// Compile an adapter's exact artifact owner/version/revision selection.
    pub(crate) fn new(key: EntryKey, revision: &[u8]) -> Self {
        Self {
            key,
            revision: ExactBytes::new(revision),
            contract: PhantomData,
        }
    }
}

/// Adapter-compiled accepted interface selection. Construction is crate-private.
#[derive(Debug)]
pub(crate) struct InterfaceSelection<C> {
    key: EntryKey,
    revision: ExactBytes,
    contract: PhantomData<fn() -> C>,
}

/// Erased adapter-compiled interface selection used to validate a complete
/// closed inventory inside the resolver's deadline boundary.
#[derive(Debug)]
pub(crate) struct InterfaceRequirement {
    key: EntryKey,
    revision: ExactBytes,
    immutability_assertion: ExactBytes,
}

impl<C> InterfaceSelection<C> {
    /// Compile an adapter's exact interface owner/version/revision selection.
    pub(crate) fn new(key: EntryKey, revision: &[u8]) -> Self {
        Self {
            key,
            revision: ExactBytes::new(revision),
            contract: PhantomData,
        }
    }
}

/// Stable, independently assertable version-lock refusal axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ResolutionRefusalCode {
    /// A required owner/version key is absent.
    EntryAbsent,
    /// The offered manifest repeats an owner/version key.
    DuplicateEntryKey,
    /// The operation repeats a required owner/version key.
    DuplicateRequiredKey,
    /// A pinned entry carries no exact revision.
    RevisionAbsent,
    /// A pinned entry carries no owner assertion.
    ImmutabilityAssertionAbsent,
    /// An artifact entry carries no delivery member.
    ArtifactMemberAbsent,
    /// An interface-only entry improperly carries an artifact digest.
    InterfaceDigestPresent,
    /// A required artifact member has no delivery.
    DeliveryMemberAbsent,
    /// A delivery member was not selected by the operation.
    UnexpectedDeliveryMember,
    /// The delivery repeats a member name.
    DuplicateDeliveryMember,
    /// The exact delivered bytes do not match the pinned digest.
    DigestMismatch,
    /// An unresolved entry carries no reason.
    UnresolvedReasonAbsent,
    /// A required entry is explicitly unresolved.
    EntryUnresolved,
    /// The manifest does not match the adapter-compiled interface selection.
    InterfaceSelectionMismatch,
    /// A capability was presented for another request.
    CapabilityRequestMismatch,
    /// A capability was presented for another key, revision, or contract marker.
    CapabilityContractMismatch,
    /// Protocol's static resolution accounting invariant failed.
    AccountingFault,
}

impl ResolutionRefusalCode {
    /// Every version-lock refusal code, in declared order.
    pub const ALL: [Self; 17] = [
        Self::EntryAbsent,
        Self::DuplicateEntryKey,
        Self::DuplicateRequiredKey,
        Self::RevisionAbsent,
        Self::ImmutabilityAssertionAbsent,
        Self::ArtifactMemberAbsent,
        Self::InterfaceDigestPresent,
        Self::DeliveryMemberAbsent,
        Self::UnexpectedDeliveryMember,
        Self::DuplicateDeliveryMember,
        Self::DigestMismatch,
        Self::UnresolvedReasonAbsent,
        Self::EntryUnresolved,
        Self::InterfaceSelectionMismatch,
        Self::CapabilityRequestMismatch,
        Self::CapabilityContractMismatch,
        Self::AccountingFault,
    ];

    /// Stable registered refusal-code spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EntryAbsent => "version-lock.entry.absent",
            Self::DuplicateEntryKey => "version-lock.entry.duplicate-key",
            Self::DuplicateRequiredKey => "version-lock.requirement.duplicate-key",
            Self::RevisionAbsent => "version-lock.entry.revision-absent",
            Self::ImmutabilityAssertionAbsent => "version-lock.entry.immutability-assertion-absent",
            Self::ArtifactMemberAbsent => "version-lock.artifact.member-absent",
            Self::InterfaceDigestPresent => "version-lock.interface.digest-present",
            Self::DeliveryMemberAbsent => "version-lock.delivery.member-absent",
            Self::UnexpectedDeliveryMember => "version-lock.delivery.unexpected-member",
            Self::DuplicateDeliveryMember => "version-lock.delivery.duplicate-member",
            Self::DigestMismatch => "version-lock.delivery.digest-mismatch",
            Self::UnresolvedReasonAbsent => "version-lock.entry.unresolved-reason-absent",
            Self::EntryUnresolved => "version-lock.entry.unresolved",
            Self::InterfaceSelectionMismatch => "version-lock.interface.selection-mismatch",
            Self::CapabilityRequestMismatch => "version-lock.capability.request-mismatch",
            Self::CapabilityContractMismatch => "version-lock.capability.contract-mismatch",
            Self::AccountingFault => "version-lock.accounting.fault",
        }
    }
}

impl fmt::Display for ResolutionRefusalCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedArtifactValue {
    key: EntryKey,
    bytes: Box<[u8]>,
}

/// A fully resolved request. Only this value can mint typed capabilities.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedRequest {
    request: ResolutionRequestIdentity,
    entries: Vec<ManifestEntry>,
    artifacts: Vec<ResolvedArtifactValue>,
    usage: Usage,
}

impl ResolvedRequest {
    /// Exact identity to which every capability from this resolution is bound.
    pub(crate) fn request_identity(&self) -> &ResolutionRequestIdentity {
        &self.request
    }

    /// Required entries in deterministic lexicographic key order.
    pub(crate) fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }

    /// Resource use retained for this completed resolution.
    pub(crate) fn usage(&self) -> Usage {
        self.usage
    }

    /// Validate an adapter-compiled interface selection without cloning or
    /// minting a capability that the caller will discard.
    pub(crate) fn validate_interface<C>(
        &self,
        selected: &InterfaceSelection<C>,
    ) -> OperationOutcome<()> {
        let Some(entry) = self.entries.iter().find(|entry| entry.key == selected.key) else {
            return refused_key(
                ResolutionRefusalCode::InterfaceSelectionMismatch,
                &selected.key,
                missing_definition(&selected.key),
            );
        };
        let Some(revision) = entry.revision.as_ref() else {
            return refused_key(
                ResolutionRefusalCode::RevisionAbsent,
                &selected.key,
                incomplete_field(b"revision"),
            );
        };
        if entry.kind() != EntryKind::Interface {
            return refused_key(
                ResolutionRefusalCode::InterfaceSelectionMismatch,
                &selected.key,
                wrong_kind(b"interface", b"artifact"),
            );
        }
        if revision != &selected.revision {
            return refused_key(
                ResolutionRefusalCode::InterfaceSelectionMismatch,
                &selected.key,
                identity_mismatch(&selected.revision, revision),
            );
        }
        OperationOutcome::Completed(())
    }

    /// Mint an artifact capability only for the exact compiled selection.
    pub(crate) fn artifact<C>(
        &self,
        selected: &ArtifactSelection<C>,
    ) -> OperationOutcome<ResolvedArtifact<C>> {
        let Some(entry) = self.entries.iter().find(|entry| entry.key == selected.key) else {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &selected.key,
                missing_definition(&selected.key),
            );
        };
        let Some(revision) = entry.revision.as_ref() else {
            return refused_key(
                ResolutionRefusalCode::RevisionAbsent,
                &selected.key,
                incomplete_field(b"revision"),
            );
        };
        let Some(artifact) = self
            .artifacts
            .iter()
            .find(|artifact| artifact.key == selected.key)
        else {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &selected.key,
                wrong_kind(b"artifact", b"interface"),
            );
        };
        if revision != &selected.revision {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &selected.key,
                identity_mismatch(&selected.revision, revision),
            );
        }
        OperationOutcome::Completed(ResolvedArtifact {
            request: self.request.clone(),
            key: selected.key.clone(),
            revision: selected.revision.clone(),
            bytes: artifact.bytes.clone(),
            contract: PhantomData,
        })
    }

    /// Mint an interface capability only for the exact compiled selection.
    pub(crate) fn interface<C>(
        &self,
        selected: &InterfaceSelection<C>,
    ) -> OperationOutcome<ResolvedInterface<C>> {
        match self.validate_interface(selected) {
            OperationOutcome::Completed(()) => {}
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
        OperationOutcome::Completed(ResolvedInterface {
            request: self.request.clone(),
            key: selected.key.clone(),
            revision: selected.revision.clone(),
            contract: PhantomData,
        })
    }
}

/// Exact verified artifact bytes bound to one request and typed contract.
#[derive(Debug)]
pub(crate) struct ResolvedArtifact<C> {
    request: ResolutionRequestIdentity,
    key: EntryKey,
    revision: ExactBytes,
    bytes: Box<[u8]>,
    contract: PhantomData<fn() -> C>,
}

impl<C> ResolvedArtifact<C> {
    /// Release the already-verified bytes only to the bound request/selection.
    pub(crate) fn bytes_for<'a>(
        &'a self,
        request: &ResolutionRequestIdentity,
        selected: &ArtifactSelection<C>,
    ) -> OperationOutcome<&'a [u8]> {
        if request != &self.request {
            return refused_key(
                ResolutionRefusalCode::CapabilityRequestMismatch,
                &self.key,
                identity_mismatch_bytes(self.request.0.as_bytes(), request.0.as_bytes()),
            );
        }
        if selected.key != self.key {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &self.key,
                identity_mismatch_bytes(
                    self.key.identity_bytes().as_slice(),
                    selected.key.identity_bytes().as_slice(),
                ),
            );
        }
        if selected.revision != self.revision {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &self.key,
                identity_mismatch(&self.revision, &selected.revision),
            );
        }
        OperationOutcome::Completed(&self.bytes)
    }
}

/// One independently authenticated interface bound to a request and contract.
#[derive(Debug)]
pub(crate) struct ResolvedInterface<C> {
    request: ResolutionRequestIdentity,
    key: EntryKey,
    revision: ExactBytes,
    contract: PhantomData<fn() -> C>,
}

impl<C> ResolvedInterface<C> {
    /// Authorize use only for the bound request and compiled interface selection.
    pub(crate) fn authorize(
        &self,
        request: &ResolutionRequestIdentity,
        selected: &InterfaceSelection<C>,
    ) -> OperationOutcome<()> {
        if request != &self.request {
            return refused_key(
                ResolutionRefusalCode::CapabilityRequestMismatch,
                &self.key,
                identity_mismatch_bytes(self.request.0.as_bytes(), request.0.as_bytes()),
            );
        }
        if selected.key != self.key {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &self.key,
                identity_mismatch_bytes(
                    self.key.identity_bytes().as_slice(),
                    selected.key.identity_bytes().as_slice(),
                ),
            );
        }
        if selected.revision != self.revision {
            return refused_key(
                ResolutionRefusalCode::CapabilityContractMismatch,
                &self.key,
                identity_mismatch(&self.revision, &selected.revision),
            );
        }
        OperationOutcome::Completed(())
    }
}

fn missing_definition(key: &EntryKey) -> ProtocolCause {
    ProtocolCause::MissingDefinition {
        definition: RetainedBytes::new(&key.identity_bytes()),
    }
}

fn incomplete_field(field: &[u8]) -> ProtocolCause {
    ProtocolCause::IncompleteEvidence {
        evidence: RetainedBytes::new(field),
    }
}

fn wrong_kind(expected: &[u8], observed: &[u8]) -> ProtocolCause {
    ProtocolCause::WrongAnchor {
        expected: RetainedBytes::new(expected),
        observed: RetainedBytes::new(observed),
    }
}

fn identity_mismatch(expected: &ExactBytes, observed: &ExactBytes) -> ProtocolCause {
    identity_mismatch_bytes(expected.as_bytes(), observed.as_bytes())
}

fn identity_mismatch_bytes(expected: &[u8], observed: &[u8]) -> ProtocolCause {
    ProtocolCause::IdentityMismatch {
        expected: RetainedBytes::new(expected),
        observed: RetainedBytes::new(observed),
    }
}

fn refusal(
    code: ResolutionRefusalCode,
    identity: AffectedIdentity,
    cause: ProtocolCause,
) -> ProtocolRefusal {
    let registered = catalog::lookup(code.as_str())
        .expect("every resolver code is registered in the local catalog");
    ProtocolRefusal::builder(
        registered,
        CatalogSelection::local(),
        ProtocolStage::ContractResolution,
        RefusalAxis::from_registered_code(registered),
        cause.into(),
        ResourceUsage::protocol("quire.protocol/resolution-usage/1", &[]),
    )
    .expect("every resolver code belongs to the resolution domain")
    .identity(identity)
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing()
}

fn refused_key<T>(
    code: ResolutionRefusalCode,
    key: &EntryKey,
    cause: ProtocolCause,
) -> OperationOutcome<T> {
    OperationOutcome::Refused(refusal(
        code,
        AffectedIdentity::retained(&key.identity_bytes()),
        cause,
    ))
}

fn refused_member<T>(
    code: ResolutionRefusalCode,
    member: &ExactBytes,
    cause: ProtocolCause,
) -> OperationOutcome<T> {
    OperationOutcome::Refused(refusal(
        code,
        AffectedIdentity::retained(member.as_bytes()),
        cause,
    ))
}

fn encode_accounting_fault(fault: AccountingFault) -> (AffectedIdentity, ProtocolCause) {
    let mut detail = Vec::new();
    let site = match fault {
        AccountingFault::UnregisteredSite { site } => {
            detail.extend_from_slice(b"unregistered-site");
            site
        }
        AccountingFault::CounterOverflow {
            site,
            consumed,
            requested,
        } => {
            detail.extend_from_slice(b"counter-overflow");
            detail.extend_from_slice(&consumed.to_be_bytes());
            detail.extend_from_slice(&requested.to_be_bytes());
            site
        }
        AccountingFault::CounterUnderflow {
            site,
            consumed,
            requested,
        } => {
            detail.extend_from_slice(b"counter-underflow");
            detail.extend_from_slice(&consumed.to_be_bytes());
            detail.extend_from_slice(&requested.to_be_bytes());
            site
        }
        AccountingFault::ClockRegressed {
            unit,
            phase,
            previous,
        } => {
            detail.extend_from_slice(b"clock-regressed");
            detail.extend_from_slice(unit.as_str().as_bytes());
            detail.push(match phase {
                Phase::BeforeUnit => 0,
                Phase::AfterUnit => 1,
            });
            detail.extend_from_slice(&previous.as_nanos().to_be_bytes());
            RESOLUTION_DEADLINE_SOURCE.id()
        }
        AccountingFault::ElapsedTimeOverflow { observed } => {
            detail.extend_from_slice(b"elapsed-time-overflow");
            detail.extend_from_slice(&observed.as_nanos().to_be_bytes());
            RESOLUTION_DEADLINE_SOURCE.id()
        }
        AccountingFault::SiteDimensionMismatch {
            site,
            declared,
            required,
        } => {
            detail.extend_from_slice(b"site-dimension-mismatch");
            detail.extend_from_slice(declared.as_str().as_bytes());
            detail.push(0);
            detail.extend_from_slice(required.as_str().as_bytes());
            site
        }
        AccountingFault::TemporalDimensionChargedStatically { site, dimension } => {
            detail.extend_from_slice(b"temporal-dimension-charged-statically");
            detail.extend_from_slice(dimension.as_str().as_bytes());
            site
        }
    };
    (
        AffectedIdentity::retained(site.as_str().as_bytes()),
        ProtocolCause::IncompleteEvidence {
            evidence: RetainedBytes::new(&detail),
        },
    )
}

fn accounting_refusal<T>(fault: AccountingFault) -> OperationOutcome<T> {
    let (identity, cause) = encode_accounting_fault(fault);
    OperationOutcome::Refused(refusal(
        ResolutionRefusalCode::AccountingFault,
        identity,
        cause,
    ))
}

fn charge_outcome<T>(error: ChargeError) -> OperationOutcome<T> {
    match error {
        ChargeError::ResourceIncomplete(incomplete) => {
            OperationOutcome::ResourceIncomplete(incomplete)
        }
        ChargeError::UnregisteredSite { site } => {
            accounting_refusal(AccountingFault::UnregisteredSite { site })
        }
        ChargeError::CounterOverflow {
            site,
            consumed,
            requested,
        } => accounting_refusal(AccountingFault::CounterOverflow {
            site,
            consumed,
            requested,
        }),
        ChargeError::CounterUnderflow {
            site,
            consumed,
            requested,
        } => accounting_refusal(AccountingFault::CounterUnderflow {
            site,
            consumed,
            requested,
        }),
        ChargeError::ClockRegressed {
            unit,
            phase,
            previous,
        } => accounting_refusal(AccountingFault::ClockRegressed {
            unit,
            phase,
            previous,
        }),
        ChargeError::ElapsedTimeOverflow { observed } => {
            accounting_refusal(AccountingFault::ElapsedTimeOverflow { observed })
        }
        ChargeError::SiteDimensionMismatch {
            site,
            declared,
            required,
        } => accounting_refusal(AccountingFault::SiteDimensionMismatch {
            site,
            declared,
            required,
        }),
        ChargeError::TemporalDimensionChargedStatically { site, dimension } => {
            accounting_refusal(AccountingFault::TemporalDimensionChargedStatically {
                site,
                dimension,
            })
        }
    }
}

fn bounded_outcome<T>(
    outcome: inventory::OperationOutcome<T, ProtocolRefusal>,
) -> OperationOutcome<T> {
    match outcome {
        inventory::OperationOutcome::Completed(value) => OperationOutcome::Completed(value),
        inventory::OperationOutcome::DomainError(refusal) => OperationOutcome::Refused(refusal),
        inventory::OperationOutcome::ResourceIncomplete(incomplete) => {
            OperationOutcome::ResourceIncomplete(incomplete)
        }
        inventory::OperationOutcome::AccountingFault(fault) => accounting_refusal(fault),
    }
}

fn sorted_duplicate_entry(entries: &[ManifestEntry]) -> Option<EntryKey> {
    let mut keys = entries.iter().map(|entry| &entry.key).collect::<Vec<_>>();
    keys.sort();
    keys.windows(2)
        .find_map(|pair| (pair[0] == pair[1]).then(|| pair[0].clone()))
}

fn sorted_duplicate_key(keys: &[EntryKey]) -> Option<EntryKey> {
    let mut ordered = keys.iter().collect::<Vec<_>>();
    ordered.sort();
    ordered
        .windows(2)
        .find_map(|pair| (pair[0] == pair[1]).then(|| pair[0].clone()))
}

fn sorted_duplicate_member(deliveries: &[ArtifactDelivery]) -> Option<ExactBytes> {
    let mut members = deliveries
        .iter()
        .map(|delivery| &delivery.member)
        .collect::<Vec<_>>();
    members.sort();
    members
        .windows(2)
        .find_map(|pair| (pair[0] == pair[1]).then(|| pair[0].clone()))
}

fn entry_for<'a>(manifest: &'a VersionLockManifest, key: &EntryKey) -> Option<&'a ManifestEntry> {
    manifest.entries.iter().find(|entry| entry.key == *key)
}

fn outcome_refusal<T>(outcome: OperationOutcome<T>) -> ProtocolRefusal {
    let OperationOutcome::Refused(refusal) = outcome else {
        unreachable!("the helper always creates a refusal")
    };
    refusal
}

fn prevalidate_manifest(
    manifest: &VersionLockManifest,
    required: &[EntryKey],
) -> Result<Vec<EntryKey>, ProtocolRefusal> {
    if let Some(key) = sorted_duplicate_entry(&manifest.entries) {
        return Err(refusal(
            ResolutionRefusalCode::DuplicateEntryKey,
            AffectedIdentity::retained(&key.identity_bytes()),
            ProtocolCause::ConflictingDefinition {
                definition: RetainedBytes::new(&key.identity_bytes()),
                conflicting_definition: RetainedBytes::new(&key.identity_bytes()),
            },
        ));
    }
    if let Some(key) = sorted_duplicate_key(required) {
        return Err(refusal(
            ResolutionRefusalCode::DuplicateRequiredKey,
            AffectedIdentity::retained(&key.identity_bytes()),
            ProtocolCause::ConflictingDefinition {
                definition: RetainedBytes::new(&key.identity_bytes()),
                conflicting_definition: RetainedBytes::new(&key.identity_bytes()),
            },
        ));
    }

    let mut ordered = required.to_vec();
    ordered.sort();
    for key in &ordered {
        let Some(entry) = entry_for(manifest, key) else {
            return Err(outcome_refusal(refused_key::<()>(
                ResolutionRefusalCode::EntryAbsent,
                key,
                missing_definition(key),
            )));
        };
        if entry.status == EntryStatus::Unresolved {
            let Some(reason) = entry
                .unresolved_reason
                .as_ref()
                .filter(|reason| !reason.as_bytes().is_empty())
            else {
                return Err(outcome_refusal(refused_key::<()>(
                    ResolutionRefusalCode::UnresolvedReasonAbsent,
                    key,
                    incomplete_field(b"unresolved-reason"),
                )));
            };
            return Err(outcome_refusal(refused_key::<()>(
                ResolutionRefusalCode::EntryUnresolved,
                key,
                ProtocolCause::IncompleteEvidence {
                    evidence: RetainedBytes::new(reason.as_bytes()),
                },
            )));
        }
        if entry
            .revision
            .as_ref()
            .is_none_or(|revision| revision.as_bytes().is_empty())
        {
            return Err(outcome_refusal(refused_key::<()>(
                ResolutionRefusalCode::RevisionAbsent,
                key,
                incomplete_field(b"revision"),
            )));
        }
        if entry
            .immutability_assertion
            .as_ref()
            .is_none_or(|assertion| assertion.as_bytes().is_empty())
        {
            return Err(outcome_refusal(refused_key::<()>(
                ResolutionRefusalCode::ImmutabilityAssertionAbsent,
                key,
                incomplete_field(b"immutability-assertion"),
            )));
        }
        match &entry.subject {
            EntrySubject::Artifact { member, .. } if member.as_bytes().is_empty() => {
                return Err(outcome_refusal(refused_key::<()>(
                    ResolutionRefusalCode::ArtifactMemberAbsent,
                    key,
                    missing_definition(key),
                )));
            }
            EntrySubject::Interface { digest } if digest.is_some() => {
                return Err(outcome_refusal(refused_key::<()>(
                    ResolutionRefusalCode::InterfaceDigestPresent,
                    key,
                    wrong_kind(b"interface-without-digest", b"interface-with-digest"),
                )));
            }
            EntrySubject::Artifact { .. }
            | EntrySubject::UnresolvedArtifact { .. }
            | EntrySubject::Interface { .. } => {}
        }
    }
    Ok(ordered)
}

fn prevalidate_delivery(
    manifest: &VersionLockManifest,
    required: &[EntryKey],
    deliveries: &[ArtifactDelivery],
) -> Result<usize, ProtocolRefusal> {
    if let Some(member) = sorted_duplicate_member(deliveries) {
        return Err(outcome_refusal(refused_member::<()>(
            ResolutionRefusalCode::DuplicateDeliveryMember,
            &member,
            ProtocolCause::ConflictingDefinition {
                definition: RetainedBytes::new(member.as_bytes()),
                conflicting_definition: RetainedBytes::new(member.as_bytes()),
            },
        )));
    }

    let mut selected_members = Vec::new();
    let mut required_bytes = 0usize;
    for key in required {
        let entry = entry_for(manifest, key)
            .expect("manifest prevalidation established every required entry");
        if let Some(member) = entry.member() {
            selected_members.push(member);
            let Some(delivery) = deliveries.iter().find(|offer| offer.member() == member) else {
                return Err(outcome_refusal(refused_key::<()>(
                    ResolutionRefusalCode::DeliveryMemberAbsent,
                    key,
                    ProtocolCause::MissingDefinition {
                        definition: RetainedBytes::new(member.as_bytes()),
                    },
                )));
            };
            required_bytes = required_bytes
                .checked_add(delivery.bytes().len())
                .ok_or_else(|| {
                    refusal(
                        ResolutionRefusalCode::AccountingFault,
                        AffectedIdentity::retained(
                            RESOLUTION_BYTES_SOURCE.id().as_str().as_bytes(),
                        ),
                        incomplete_field(b"resolution-byte-total-overflow"),
                    )
                })?;
        }
    }

    let mut unexpected = deliveries
        .iter()
        .filter(|delivery| !selected_members.contains(&&delivery.member))
        .collect::<Vec<_>>();
    unexpected.sort_by(|left, right| left.member.cmp(&right.member));
    if let Some(delivery) = unexpected.first() {
        return Err(outcome_refusal(refused_member::<()>(
            ResolutionRefusalCode::UnexpectedDeliveryMember,
            &delivery.member,
            wrong_kind(b"selected-delivery-member", delivery.member.as_bytes()),
        )));
    }
    Ok(required_bytes)
}

/// Resolve every independently required key before returning a capability.
#[must_use]
pub(crate) fn resolve<C: Clock + 'static>(
    request: ResolutionRequestIdentity,
    manifest: &VersionLockManifest,
    required: &[EntryKey],
    deliveries: &[ArtifactDelivery],
    budget: ResolutionBudget,
    clock: C,
) -> OperationOutcome<ResolvedRequest> {
    resolve_selected(
        request,
        manifest,
        required,
        SelectedResolution {
            deliveries,
            interfaces: &[],
        },
        budget,
        clock,
        None::<(usize, SiteId, fn() -> ())>,
    )
    .map(|(resolved, _)| resolved)
}

struct SelectedResolution<'a> {
    deliveries: &'a [ArtifactDelivery],
    interfaces: &'a [InterfaceRequirement],
}

fn resolve_selected<C, T>(
    request: ResolutionRequestIdentity,
    manifest: &VersionLockManifest,
    required: &[EntryKey],
    selected: SelectedResolution<'_>,
    budget: ResolutionBudget,
    clock: C,
    verification: Option<(usize, SiteId, impl FnOnce() -> T)>,
) -> OperationOutcome<(ResolvedRequest, Option<T>)>
where
    C: Clock + 'static,
{
    let production_inventory = inventory::production_inventory()
        .expect("the compile-time Protocol resource inventory must reconcile");
    let mut resources = ResourceAccountant::with_resolution_clock(
        budget.into_shared(),
        &production_inventory,
        clock,
    );

    let entry_work = manifest.entries.len().max(required.len());
    if let Err(error) = resources.charge(RESOLUTION_ENTRY_SOURCE.id(), entry_work) {
        return charge_outcome(error);
    }

    let manifest_outcome = resources.run_resolution_unit(MANIFEST_UNIT, |_| {
        inventory::OperationOutcome::from_domain(prevalidate_manifest(manifest, required).and_then(
            |ordered| {
                for interface in selected.interfaces {
                    let entry = entry_for(manifest, &interface.key)
                        .expect("manifest prevalidation established each interface key");
                    let Some(revision) = entry.revision.as_ref() else {
                        return Err(outcome_refusal(refused_key::<()>(
                            ResolutionRefusalCode::RevisionAbsent,
                            &interface.key,
                            incomplete_field(b"revision"),
                        )));
                    };
                    if entry.kind() != EntryKind::Interface {
                        return Err(outcome_refusal(refused_key::<()>(
                            ResolutionRefusalCode::InterfaceSelectionMismatch,
                            &interface.key,
                            wrong_kind(b"interface", b"artifact"),
                        )));
                    }
                    if revision != &interface.revision {
                        return Err(outcome_refusal(refused_key::<()>(
                            ResolutionRefusalCode::InterfaceSelectionMismatch,
                            &interface.key,
                            identity_mismatch(&interface.revision, revision),
                        )));
                    }
                    let assertion = entry
                        .immutability_assertion
                        .as_ref()
                        .expect("manifest prevalidation established an owner assertion");
                    if assertion != &interface.immutability_assertion {
                        return Err(outcome_refusal(refused_key::<()>(
                            ResolutionRefusalCode::InterfaceSelectionMismatch,
                            &interface.key,
                            identity_mismatch(&interface.immutability_assertion, assertion),
                        )));
                    }
                }
                Ok(ordered)
            },
        ))
    });
    let ordered = match bounded_outcome(manifest_outcome) {
        OperationOutcome::Completed(ordered) => ordered,
        OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
        OperationOutcome::ResourceIncomplete(incomplete) => {
            return OperationOutcome::ResourceIncomplete(incomplete)
        }
    };

    let delivery_outcome = resources.run_resolution_unit(DELIVERY_UNIT, |_| {
        inventory::OperationOutcome::from_domain(prevalidate_delivery(
            manifest,
            &ordered,
            selected.deliveries,
        ))
    });
    let required_bytes = match bounded_outcome(delivery_outcome) {
        OperationOutcome::Completed(required_bytes) => required_bytes,
        OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
        OperationOutcome::ResourceIncomplete(incomplete) => {
            return OperationOutcome::ResourceIncomplete(incomplete)
        }
    };

    if let Err(error) = resources.charge(RESOLUTION_BYTES_SOURCE.id(), required_bytes) {
        return charge_outcome(error);
    }

    let mut entries = Vec::with_capacity(ordered.len());
    let mut artifacts = Vec::new();
    for key in &ordered {
        let entry =
            entry_for(manifest, key).expect("manifest prevalidation established every ordered key");
        let digest_outcome = resources.run_resolution_unit(DIGEST_UNIT, |_| {
            let result = match &entry.subject {
                EntrySubject::Artifact {
                    member,
                    digest: expected,
                } => {
                    let delivery = selected
                        .deliveries
                        .iter()
                        .find(|delivery| delivery.member == *member)
                        .expect("delivery prevalidation established the selected member");
                    let observed = ByteDigest::of(&delivery.bytes);
                    if observed == *expected {
                        Ok(Some(delivery.bytes.clone()))
                    } else {
                        let expected = format!("{expected:x}");
                        let observed = format!("{observed:x}");
                        Err(outcome_refusal(refused_key::<()>(
                            ResolutionRefusalCode::DigestMismatch,
                            key,
                            identity_mismatch_bytes(expected.as_bytes(), observed.as_bytes()),
                        )))
                    }
                }
                EntrySubject::Interface { .. } => Ok(None),
                EntrySubject::UnresolvedArtifact { .. } => {
                    unreachable!("manifest prevalidation refuses unresolved entries")
                }
            };
            inventory::OperationOutcome::from_domain(result.map(|bytes| (bytes, entry.clone())))
        });
        let (bytes, resolved_entry) = match bounded_outcome(digest_outcome) {
            OperationOutcome::Completed(resolved) => resolved,
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete)
            }
        };
        if let Some(bytes) = bytes {
            artifacts.push(ResolvedArtifactValue {
                key: key.clone(),
                bytes,
            });
        }
        entries.push(resolved_entry);
    }

    let verified = if let Some((selection_bytes, selection_source, verify)) = verification {
        if let Err(error) = resources.charge(selection_source, selection_bytes) {
            return charge_outcome(error);
        }
        let outcome = resources.run_resolution_unit(REQUEST_SELECTION_UNIT, |_| {
            inventory::OperationOutcome::from_domain(Ok::<T, ProtocolRefusal>(verify()))
        });
        match bounded_outcome(outcome) {
            OperationOutcome::Completed(value) => Some(value),
            OperationOutcome::Refused(refusal) => return OperationOutcome::Refused(refusal),
            OperationOutcome::ResourceIncomplete(incomplete) => {
                return OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
    } else {
        None
    };

    OperationOutcome::Completed((
        ResolvedRequest {
            request,
            entries,
            artifacts,
            usage: resources.usage(),
        },
        verified,
    ))
}

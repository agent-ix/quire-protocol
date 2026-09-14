// SPDX-License-Identifier: AGPL-3.0-or-later
//! B-owned version-lock manifest resolution. Owner task: **Task-013**.
//!
//! Resolution is deliberately an in-memory, capability-free operation. Its
//! complete input is a caller-authored manifest, the scenario's required keys,
//! delivered byte slices, a caller budget and a monotonic [`Clock`]. There is
//! no filesystem, version-control, process or network handle in the API.

use std::fmt;
use std::time::Duration;

use crate::bounds::{Clock, DeadlineGuard, DeadlineRefusal, UnitLabel};
use crate::ByteDigest;

const MANIFEST_UNIT: UnitLabel = UnitLabel::new("version-lock-manifest-validation");
const DELIVERY_UNIT: UnitLabel = UnitLabel::new("version-lock-delivery-validation");
const DIGEST_UNIT: UnitLabel = UnitLabel::new("version-lock-digest");

/// Bytes retained without parsing, normalization or re-encoding.
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

/// The `(owner, contract version)` identity of one manifest entry.
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

/// One B-authored manifest entry.
///
/// Optional pinned fields remain explicit because the manifest is untrusted
/// input at this boundary. Resolution, rather than a producer-side builder,
/// decides whether a pinned entry is complete and names the entry on refusal.
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
    ///
    /// `digest` is accepted as manifest input so resolution can explicitly
    /// refuse the forbidden shape; successful interface entries pass `None`.
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

    /// Declare an unresolved entry with the exact reason B records.
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

    /// Declare an unresolved interface entry with the exact reason B records.
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

    /// Entry identity.
    #[must_use]
    pub const fn key(&self) -> &EntryKey {
        &self.key
    }

    /// Subject kind.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        match &self.subject {
            EntrySubject::Artifact { .. } | EntrySubject::UnresolvedArtifact { .. } => {
                EntryKind::Artifact
            }
            EntrySubject::Interface { .. } => EntryKind::Interface,
        }
    }

    /// Resolution status.
    #[must_use]
    pub const fn status(&self) -> EntryStatus {
        self.status
    }

    /// Exact pinned revision, if present.
    #[must_use]
    pub const fn revision(&self) -> Option<&ExactBytes> {
        self.revision.as_ref()
    }

    /// Owner's exact immutability/publication assertion, if present.
    #[must_use]
    pub const fn immutability_assertion(&self) -> Option<&ExactBytes> {
        self.immutability_assertion.as_ref()
    }

    /// Exact unresolved reason, if this is unresolved.
    #[must_use]
    pub const fn unresolved_reason(&self) -> Option<&ExactBytes> {
        self.unresolved_reason.as_ref()
    }

    /// Independently selected delivery member, for an artifact.
    #[must_use]
    pub const fn member(&self) -> Option<&ExactBytes> {
        match &self.subject {
            EntrySubject::Artifact { member, .. } | EntrySubject::UnresolvedArtifact { member } => {
                Some(member)
            }
            EntrySubject::Interface { .. } => None,
        }
    }

    /// Exact expected digest, for an artifact.
    #[must_use]
    pub const fn digest(&self) -> Option<&ByteDigest> {
        match &self.subject {
            EntrySubject::Artifact { digest, .. } => Some(digest),
            EntrySubject::Interface { digest } => digest.as_ref(),
            EntrySubject::UnresolvedArtifact { .. } => None,
        }
    }
}

/// The B-authored set of producer locks. Requiredness is intentionally absent.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VersionLockManifest {
    entries: Vec<ManifestEntry>,
}

impl VersionLockManifest {
    /// Retain entries in authored order. Duplicate detection belongs to
    /// resolution so malformed input yields a typed per-key refusal.
    #[must_use]
    pub const fn new(entries: Vec<ManifestEntry>) -> Self {
        Self { entries }
    }

    /// Authored entries.
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
    /// Retain a delivery member and its exact bytes.
    #[must_use]
    pub fn new(member: &[u8], bytes: &[u8]) -> Self {
        Self {
            member: ExactBytes::new(member),
            bytes: bytes.into(),
        }
    }

    /// Exact member name.
    #[must_use]
    pub const fn member(&self) -> &ExactBytes {
        &self.member
    }

    /// Exact delivered bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// The caller's independent resolution limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolutionBudget {
    entries: usize,
    bytes: usize,
    deadline: Duration,
}

impl ResolutionBudget {
    /// Grant all three resolution dimensions.
    #[must_use]
    pub const fn new(entries: usize, bytes: usize, deadline: Duration) -> Self {
        Self {
            entries,
            bytes,
            deadline,
        }
    }

    /// Entry-count grant.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }

    /// Bytes-digested grant.
    #[must_use]
    pub const fn bytes(self) -> usize {
        self.bytes
    }

    /// Monotonic deadline grant.
    #[must_use]
    pub const fn deadline(self) -> Duration {
        self.deadline
    }
}

/// Consumption retained separately on each reproducible dimension.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ResolutionUsage {
    entries: usize,
    bytes_digested: usize,
}

impl ResolutionUsage {
    /// Entries completely resolved.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }

    /// Exact delivered bytes hashed.
    #[must_use]
    pub const fn bytes_digested(self) -> usize {
        self.bytes_digested
    }
}

/// Stable, independently assertable version-lock refusal axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ResolutionRefusalCode {
    /// A required `(owner, contract version)` key is absent.
    EntryAbsent,
    /// Two manifest entries share one key.
    DuplicateEntryKey,
    /// A scenario declared one required key twice.
    DuplicateRequiredKey,
    /// A pinned entry has no revision.
    RevisionAbsent,
    /// A pinned entry has no owner assertion.
    ImmutabilityAssertionAbsent,
    /// An artifact entry has no selected member.
    ArtifactMemberAbsent,
    /// An interface-only entry improperly carries a digest.
    InterfaceDigestPresent,
    /// The selected delivery member is absent.
    DeliveryMemberAbsent,
    /// The delivery contains a member the required entries do not select.
    UnexpectedDeliveryMember,
    /// Two delivery members share one exact name.
    DuplicateDeliveryMember,
    /// Delivered bytes do not equal the selected exact-byte digest.
    DigestMismatch,
    /// An unresolved entry carries no reason.
    UnresolvedReasonAbsent,
}

impl ResolutionRefusalCode {
    /// Every refusal axis.
    pub const ALL: [Self; 12] = [
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
    ];

    /// Stable catalog spelling.
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
        }
    }
}

impl fmt::Display for ResolutionRefusalCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A manifest refusal, including the partial non-byte resolution state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionRefusal {
    code: ResolutionRefusalCode,
    key: Option<EntryKey>,
    member: Option<ExactBytes>,
    resolved_before: Vec<ManifestEntry>,
}

impl ResolutionRefusal {
    fn for_key(
        code: ResolutionRefusalCode,
        key: &EntryKey,
        resolved_before: &[ManifestEntry],
    ) -> Self {
        Self {
            code,
            key: Some(key.clone()),
            member: None,
            resolved_before: resolved_before.to_vec(),
        }
    }

    fn for_member(code: ResolutionRefusalCode, member: &ExactBytes) -> Self {
        Self {
            code,
            key: None,
            member: Some(member.clone()),
            resolved_before: Vec::new(),
        }
    }

    /// Stable refusal axis.
    #[must_use]
    pub const fn code(&self) -> ResolutionRefusalCode {
        self.code
    }

    /// Affected manifest key, when this is an entry refusal.
    #[must_use]
    pub const fn key(&self) -> Option<&EntryKey> {
        self.key.as_ref()
    }

    /// Affected delivery member, when this is a delivery-surface refusal.
    #[must_use]
    pub fn member(&self) -> Option<&[u8]> {
        self.member.as_ref().map(ExactBytes::as_bytes)
    }

    /// Entries resolved before the causing entry, without exposing bytes.
    #[must_use]
    pub fn resolved_before(&self) -> &[ManifestEntry] {
        &self.resolved_before
    }
}

/// The dimension that made resolution incomplete.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ResolutionDimension {
    /// `resolution-entry-count`.
    EntryCount,
    /// `resolution-bytes-digested`.
    BytesDigested,
    /// `resolution-deadline`.
    Deadline,
}

/// Count/byte resource exhaustion before resolution began.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrecheckedIncomplete {
    dimension: ResolutionDimension,
    granted: usize,
    required: usize,
}

/// Deadline exhaustion after an indivisible digest operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeadlineIncomplete {
    refusal: DeadlineRefusal,
    usage: ResolutionUsage,
}

impl DeadlineIncomplete {
    /// The underlying measured monotonic deadline outcome.
    #[must_use]
    pub const fn refusal(self) -> DeadlineRefusal {
        self.refusal
    }

    /// Always the resolution deadline dimension.
    #[must_use]
    pub const fn dimension(self) -> ResolutionDimension {
        ResolutionDimension::Deadline
    }

    /// Entries resolved or fully digested before the deadline observation.
    #[must_use]
    pub const fn consumed_entries(self) -> usize {
        self.usage.entries
    }

    /// Bytes actually digested before the deadline observation.
    #[must_use]
    pub const fn consumed_bytes(self) -> usize {
        self.usage.bytes_digested
    }
}

/// A resource-incomplete outcome, deliberately not a manifest refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionIncomplete {
    /// Entry count exceeded before any entry was resolved.
    EntryCount(PrecheckedIncomplete),
    /// Byte count exceeded before any byte was digested.
    BytesDigested(PrecheckedIncomplete),
    /// The monotonic resolution deadline expired or regressed.
    Deadline(DeadlineIncomplete),
}

impl ResolutionIncomplete {
    /// Exact exhausted dimension.
    #[must_use]
    pub const fn dimension(self) -> ResolutionDimension {
        match self {
            Self::EntryCount(_) => ResolutionDimension::EntryCount,
            Self::BytesDigested(_) => ResolutionDimension::BytesDigested,
            Self::Deadline(_) => ResolutionDimension::Deadline,
        }
    }

    /// Entries consumed before the outcome.
    #[must_use]
    pub const fn consumed_entries(self) -> usize {
        match self {
            Self::EntryCount(_) | Self::BytesDigested(_) => 0,
            Self::Deadline(incomplete) => incomplete.consumed_entries(),
        }
    }

    /// Bytes consumed before the outcome.
    #[must_use]
    pub const fn consumed_bytes(self) -> usize {
        match self {
            Self::EntryCount(_) | Self::BytesDigested(_) => 0,
            Self::Deadline(incomplete) => incomplete.consumed_bytes(),
        }
    }

    /// Caller grant for a prechecked dimension.
    #[must_use]
    pub const fn granted(self) -> Option<usize> {
        match self {
            Self::EntryCount(value) | Self::BytesDigested(value) => Some(value.granted),
            Self::Deadline(_) => None,
        }
    }

    /// Required amount for a prechecked dimension.
    #[must_use]
    pub const fn required(self) -> Option<usize> {
        match self {
            Self::EntryCount(value) | Self::BytesDigested(value) => Some(value.required),
            Self::Deadline(_) => None,
        }
    }
}

/// A required entry is present but not yet pinned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnexecutableResolution {
    key: EntryKey,
    reason: ExactBytes,
    resolved_before: Vec<ManifestEntry>,
}

impl UnexecutableResolution {
    /// The unresolved required key.
    #[must_use]
    pub const fn key(&self) -> &EntryKey {
        &self.key
    }

    /// Exact B-authored reason the entry cannot be pinned.
    #[must_use]
    pub fn reason(&self) -> &[u8] {
        self.reason.as_bytes()
    }

    /// Entries resolved before reaching the unresolved key.
    #[must_use]
    pub fn resolved_before(&self) -> &[ManifestEntry] {
        &self.resolved_before
    }
}

/// A fully resolved manifest and the exact verified artifact bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedManifest {
    entries: Vec<ManifestEntry>,
    artifacts: Vec<(EntryKey, Box<[u8]>)>,
    usage: ResolutionUsage,
    elapsed: Duration,
}

impl ResolvedManifest {
    /// Entries, in scenario-required order.
    #[must_use]
    pub fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }

    /// The exact byte sequence verified for `key`, or `None` for interfaces.
    #[must_use]
    pub fn artifact_bytes(&self, key: &EntryKey) -> Option<&[u8]> {
        self.artifacts
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, bytes)| bytes.as_ref())
    }

    /// Reproducible consumption figures.
    #[must_use]
    pub const fn usage(&self) -> ResolutionUsage {
        self.usage
    }

    /// Observed elapsed time, retained but excluded from reproducible usage.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

/// Every typed resolution outcome. Unexecutable and incomplete are neither a
/// pass nor a refusal of producer bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionOutcome {
    /// Every required entry resolved.
    Resolved(ResolvedManifest),
    /// The manifest or delivery was invalid.
    Refused(ResolutionRefusal),
    /// A required entry has not been pinned.
    Unexecutable(UnexecutableResolution),
    /// The caller's resolution budget was insufficient.
    Incomplete(ResolutionIncomplete),
}

fn duplicate_key(entries: &[ManifestEntry]) -> Option<&EntryKey> {
    entries.iter().enumerate().find_map(|(index, entry)| {
        entries[index + 1..]
            .iter()
            .any(|other| other.key == entry.key)
            .then_some(&entry.key)
    })
}

fn duplicate_member(deliveries: &[ArtifactDelivery]) -> Option<&ExactBytes> {
    deliveries.iter().enumerate().find_map(|(index, delivery)| {
        deliveries[index + 1..]
            .iter()
            .any(|other| other.member == delivery.member)
            .then_some(&delivery.member)
    })
}

fn duplicate_required_key(required: &[EntryKey]) -> Option<&EntryKey> {
    required.iter().enumerate().find_map(|(index, key)| {
        required[index + 1..]
            .iter()
            .any(|other| other == key)
            .then_some(key)
    })
}

fn entry_for<'a>(manifest: &'a VersionLockManifest, key: &EntryKey) -> Option<&'a ManifestEntry> {
    manifest.entries.iter().find(|entry| entry.key == *key)
}

fn required_artifact_member(
    manifest: &VersionLockManifest,
    required: &[EntryKey],
    member: &ExactBytes,
) -> bool {
    required.iter().any(|key| {
        entry_for(manifest, key)
            .and_then(ManifestEntry::member)
            .is_some_and(|selected| selected == member)
    })
}

fn prevalidate_manifest(
    manifest: &VersionLockManifest,
    required: &[EntryKey],
) -> Option<ResolutionOutcome> {
    if let Some(key) = duplicate_key(&manifest.entries) {
        return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
            ResolutionRefusalCode::DuplicateEntryKey,
            key,
            &[],
        )));
    }
    if let Some(key) = duplicate_required_key(required) {
        return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
            ResolutionRefusalCode::DuplicateRequiredKey,
            key,
            &[],
        )));
    }

    for key in required {
        let Some(entry) = entry_for(manifest, key) else {
            return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                ResolutionRefusalCode::EntryAbsent,
                key,
                &[],
            )));
        };
        if entry.status == EntryStatus::Unresolved {
            let Some(reason) = entry
                .unresolved_reason
                .as_ref()
                .filter(|reason| !reason.as_bytes().is_empty())
            else {
                return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                    ResolutionRefusalCode::UnresolvedReasonAbsent,
                    key,
                    &[],
                )));
            };
            return Some(ResolutionOutcome::Unexecutable(UnexecutableResolution {
                key: key.clone(),
                reason: reason.clone(),
                resolved_before: Vec::new(),
            }));
        }
        if entry
            .revision
            .as_ref()
            .is_none_or(|revision| revision.as_bytes().is_empty())
        {
            return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                ResolutionRefusalCode::RevisionAbsent,
                key,
                &[],
            )));
        }
        if entry
            .immutability_assertion
            .as_ref()
            .is_none_or(|assertion| assertion.as_bytes().is_empty())
        {
            return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                ResolutionRefusalCode::ImmutabilityAssertionAbsent,
                key,
                &[],
            )));
        }
        match &entry.subject {
            EntrySubject::Artifact { member, .. } if member.as_bytes().is_empty() => {
                return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                    ResolutionRefusalCode::ArtifactMemberAbsent,
                    key,
                    &[],
                )));
            }
            EntrySubject::Interface { digest } if digest.is_some() => {
                return Some(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                    ResolutionRefusalCode::InterfaceDigestPresent,
                    key,
                    &[],
                )));
            }
            EntrySubject::Artifact { .. }
            | EntrySubject::UnresolvedArtifact { .. }
            | EntrySubject::Interface { .. } => {}
        }
    }
    None
}

fn prevalidate_delivery(
    manifest: &VersionLockManifest,
    required: &[EntryKey],
    deliveries: &[ArtifactDelivery],
    budget: ResolutionBudget,
) -> Result<usize, ResolutionOutcome> {
    if let Some(member) = duplicate_member(deliveries) {
        return Err(ResolutionOutcome::Refused(ResolutionRefusal::for_member(
            ResolutionRefusalCode::DuplicateDeliveryMember,
            member,
        )));
    }
    if let Some(delivery) = deliveries
        .iter()
        .find(|delivery| !required_artifact_member(manifest, required, &delivery.member))
    {
        return Err(ResolutionOutcome::Refused(ResolutionRefusal::for_member(
            ResolutionRefusalCode::UnexpectedDeliveryMember,
            &delivery.member,
        )));
    }

    let mut required_bytes = 0usize;
    for key in required {
        let Some(entry) = entry_for(manifest, key) else {
            return Err(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                ResolutionRefusalCode::EntryAbsent,
                key,
                &[],
            )));
        };
        if let Some(member) = entry.member() {
            let Some(delivery) = deliveries.iter().find(|offer| offer.member() == member) else {
                return Err(ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                    ResolutionRefusalCode::DeliveryMemberAbsent,
                    key,
                    &[],
                )));
            };
            let Some(next) = required_bytes.checked_add(delivery.bytes().len()) else {
                return Err(ResolutionOutcome::Incomplete(
                    ResolutionIncomplete::BytesDigested(PrecheckedIncomplete {
                        dimension: ResolutionDimension::BytesDigested,
                        granted: budget.bytes,
                        required: usize::MAX,
                    }),
                ));
            };
            required_bytes = next;
        }
    }
    if required_bytes > budget.bytes {
        return Err(ResolutionOutcome::Incomplete(
            ResolutionIncomplete::BytesDigested(PrecheckedIncomplete {
                dimension: ResolutionDimension::BytesDigested,
                granted: budget.bytes,
                required: required_bytes,
            }),
        ));
    }
    Ok(required_bytes)
}

/// Resolve every key a scenario independently declares before returning any
/// producer byte to its consumer.
#[must_use]
pub fn resolve<C: Clock + ?Sized>(
    manifest: &VersionLockManifest,
    required: &[EntryKey],
    deliveries: &[ArtifactDelivery],
    budget: ResolutionBudget,
    clock: &C,
) -> ResolutionOutcome {
    let entry_work = manifest.entries.len().max(required.len());
    if entry_work > budget.entries {
        return ResolutionOutcome::Incomplete(ResolutionIncomplete::EntryCount(
            PrecheckedIncomplete {
                dimension: ResolutionDimension::EntryCount,
                granted: budget.entries,
                required: entry_work,
            },
        ));
    }

    let mut deadline = DeadlineGuard::start(clock, budget.deadline);
    let mut usage = ResolutionUsage::default();

    let manifest_token = match deadline.begin_unit(clock, MANIFEST_UNIT) {
        Ok(token) => token,
        Err(refusal) => {
            return ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(
                DeadlineIncomplete { refusal, usage },
            ));
        }
    };
    let manifest_outcome = prevalidate_manifest(manifest, required);
    if let Err(refusal) = deadline.end_unit(clock, manifest_token) {
        return ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(DeadlineIncomplete {
            refusal,
            usage,
        }));
    }
    if let Some(outcome) = manifest_outcome {
        return outcome;
    }

    let delivery_token = match deadline.begin_unit(clock, DELIVERY_UNIT) {
        Ok(token) => token,
        Err(refusal) => {
            return ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(
                DeadlineIncomplete { refusal, usage },
            ));
        }
    };
    let delivery_outcome = prevalidate_delivery(manifest, required, deliveries, budget);
    if let Err(refusal) = deadline.end_unit(clock, delivery_token) {
        return ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(DeadlineIncomplete {
            refusal,
            usage,
        }));
    }
    if let Err(outcome) = delivery_outcome {
        return outcome;
    }

    let mut resolved_entries = Vec::with_capacity(required.len());
    let mut resolved_artifacts = Vec::new();

    for key in required {
        let Some(entry) = entry_for(manifest, key) else {
            return ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                ResolutionRefusalCode::EntryAbsent,
                key,
                &resolved_entries,
            ));
        };
        let artifact = match &entry.subject {
            EntrySubject::Interface { .. } => None,
            EntrySubject::Artifact {
                member,
                digest: expected_digest,
            } => {
                let Some(delivery) = deliveries.iter().find(|offer| offer.member == *member) else {
                    return ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                        ResolutionRefusalCode::DeliveryMemberAbsent,
                        key,
                        &resolved_entries,
                    ));
                };
                Some((expected_digest, delivery))
            }
            EntrySubject::UnresolvedArtifact { .. } => {
                let Some(reason) = entry
                    .unresolved_reason
                    .as_ref()
                    .filter(|reason| !reason.as_bytes().is_empty())
                else {
                    return ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                        ResolutionRefusalCode::UnresolvedReasonAbsent,
                        key,
                        &resolved_entries,
                    ));
                };
                return ResolutionOutcome::Unexecutable(UnexecutableResolution {
                    key: key.clone(),
                    reason: reason.clone(),
                    resolved_before: resolved_entries,
                });
            }
        };

        let token = match deadline.begin_unit(clock, DIGEST_UNIT) {
            Ok(token) => token,
            Err(refusal) => {
                return ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(
                    DeadlineIncomplete { refusal, usage },
                ));
            }
        };

        let digest_matches = artifact
            .map(|(expected, delivery)| {
                usage.bytes_digested += delivery.bytes.len();
                ByteDigest::of(&delivery.bytes) == *expected
            })
            .unwrap_or(true);
        usage.entries += 1;

        if let Err(refusal) = deadline.end_unit(clock, token) {
            return ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(
                DeadlineIncomplete { refusal, usage },
            ));
        }
        if !digest_matches {
            return ResolutionOutcome::Refused(ResolutionRefusal::for_key(
                ResolutionRefusalCode::DigestMismatch,
                key,
                &resolved_entries,
            ));
        }
        if let Some((_, delivery)) = artifact {
            resolved_artifacts.push((key.clone(), delivery.bytes.clone()));
        }
        resolved_entries.push(entry.clone());
    }

    ResolutionOutcome::Resolved(ResolvedManifest {
        entries: resolved_entries,
        artifacts: resolved_artifacts,
        usage,
        elapsed: deadline.observed(),
    })
}

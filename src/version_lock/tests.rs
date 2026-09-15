// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-013: deterministic request-bound version-lock resolution.

use std::cell::Cell;
use std::time::{Duration, Instant};

use ix_trace_rs::trace;

use super::*;
use crate::bounds::inventory::BudgetDimension;
use crate::bounds::Clock;
use crate::outcome::OperationOutcome;
use crate::refusal::{AffectedIdentity, PartialOutput, ProtocolStage, RefusalCause};

const ARTIFACT_BYTES: &[u8] = b"{\n  \"profile\": \"exact-delivery\"\n}\n";
const COMPILED_MEMBER: &str = concat!("compiled", ".", "json");
const V1_MEMBER: &str = concat!("v1", ".", "json");
const V2_MEMBER: &str = concat!("v2", ".", "json");
const A_MEMBER: &str = concat!("a", ".", "json");
const Z_MEMBER: &str = concat!("z", ".", "json");
const EXTRA_MEMBER: &str = concat!("extra", ".", "bin");

#[derive(Debug)]
struct ArtifactContract;

#[derive(Debug)]
struct InterfaceContract;

#[derive(Debug)]
struct ScriptedClock {
    origin: Instant,
    nanos: Vec<u64>,
    next: Cell<usize>,
}

impl ScriptedClock {
    fn new(nanos: &[u64]) -> Self {
        Self {
            origin: Instant::now(),
            nanos: nanos.to_vec(),
            next: Cell::new(0),
        }
    }
}

impl Clock for ScriptedClock {
    fn monotonic(&self) -> Instant {
        let index = self.next.get();
        self.next.set(index.saturating_add(1));
        self.origin + Duration::from_nanos(*self.nanos.get(index).unwrap_or(&0))
    }
}

fn key(owner: &str, version: &str) -> EntryKey {
    EntryKey::new(owner.as_bytes(), version.as_bytes()).expect("valid fixture key")
}

fn request(identity: &str) -> ResolutionRequestIdentity {
    ResolutionRequestIdentity::new(identity.as_bytes()).expect("valid request identity")
}

fn artifact_entry(owner: &str, version: &str, member: &str, bytes: &[u8]) -> ManifestEntry {
    ManifestEntry::pinned_artifact(
        key(owner, version),
        Some(format!("revision-{owner}-{version}").as_bytes()),
        Some(b"owner asserts immutable and published"),
        member.as_bytes(),
        ByteDigest::of(bytes),
    )
}

fn interface_entry(owner: &str, version: &str, revision: &[u8]) -> ManifestEntry {
    ManifestEntry::pinned_interface(
        key(owner, version),
        Some(revision),
        Some(b"owner asserts immutable and published"),
        None,
    )
}

fn delivery(member: &str, bytes: &[u8]) -> ArtifactDelivery {
    ArtifactDelivery::new(member.as_bytes(), bytes)
}

fn budget(entries: usize, bytes: usize, deadline_nanos: u64) -> ResolutionBudget {
    ResolutionBudget::new(entries, bytes, Duration::from_nanos(deadline_nanos))
        .expect("valid positive resolution budget")
}

fn completed(outcome: OperationOutcome<ResolvedRequest>) -> ResolvedRequest {
    match outcome {
        OperationOutcome::Completed(resolved) => resolved,
        other => panic!("fixture should resolve, got {other:?}"),
    }
}

fn take_refusal(outcome: OperationOutcome<ResolvedRequest>) -> ProtocolRefusal {
    let OperationOutcome::Refused(refusal) = outcome else {
        panic!("fixture should refuse");
    };
    assert_eq!(refusal.stage(), ProtocolStage::ContractResolution);
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
    refusal
}

fn assert_retained_identity(refusal: &ProtocolRefusal, expected: &[u8]) {
    let AffectedIdentity::Retained(actual) = refusal.identity() else {
        panic!("a resolver refusal must retain its exact affected identity");
    };
    assert_eq!(actual.as_bytes(), expected);
}

fn generous_clock() -> ScriptedClock {
    ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3, 3, 4])
}

fn assert_entry_exact(actual: &ManifestEntry, expected: &ManifestEntry) {
    assert_eq!(actual.key(), expected.key());
    assert_eq!(actual.kind(), expected.kind());
    assert_eq!(actual.status(), expected.status());
    assert_eq!(actual.revision(), expected.revision());
    assert_eq!(
        actual.immutability_assertion(),
        expected.immutability_assertion()
    );
    assert_eq!(actual.unresolved_reason(), expected.unresolved_reason());
    assert_eq!(actual.member(), expected.member());
    assert_eq!(actual.digest(), expected.digest());
}

fn only_protocol_cause(refusal: &ProtocolRefusal) -> &ProtocolCause {
    let [RefusalCause::Protocol(cause)] = refusal.causes().declared_order() else {
        panic!("resolver refusals must retain exactly one Protocol-owned cause");
    };
    cause
}

#[test]
#[trace("TC-013", "FR-009-AC-1", "FR-009-AC-8", "FR-012-AC-3", "FR-012-AC-4")]
fn tc_013_resolution_mints_only_exact_request_bound_capabilities() {
    let artifact = artifact_entry("A", "compiled/2", COMPILED_MEMBER, ARTIFACT_BYTES);
    let interface = interface_entry("EA", "producer-execution/v1", b"accepted-ea-revision");
    let manifest = VersionLockManifest::new(vec![interface.clone(), artifact.clone()]);
    let request = request("assessment-7");

    let resolved = completed(resolve(
        request.clone(),
        &manifest,
        &[interface.key().clone(), artifact.key().clone()],
        &[delivery(COMPILED_MEMBER, ARTIFACT_BYTES)],
        budget(2, ARTIFACT_BYTES.len(), 100),
        ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3, 3, 4]),
    ));

    assert_eq!(resolved.request_identity(), &request);
    assert_eq!(resolved.entries().len(), 2);
    assert_entry_exact(&resolved.entries()[0], &artifact);
    assert_entry_exact(&resolved.entries()[1], &interface);
    assert_eq!(
        resolved
            .usage()
            .consumed(BudgetDimension::ResolutionEntryCount),
        2
    );
    assert_eq!(
        resolved
            .usage()
            .consumed(BudgetDimension::ResolutionBytesDigested),
        ARTIFACT_BYTES.len()
    );

    let selected_artifact = ArtifactSelection::<ArtifactContract>::new(
        artifact.key().clone(),
        artifact.revision().expect("pinned revision").as_bytes(),
    );
    let OperationOutcome::Completed(capability) = resolved.artifact(&selected_artifact) else {
        panic!("the exact artifact selection should mint a capability");
    };
    let OperationOutcome::Completed(bytes) = capability.bytes_for(&request, &selected_artifact)
    else {
        panic!("the bound request should receive the verified bytes");
    };
    assert_eq!(bytes, ARTIFACT_BYTES);

    let selected_interface = InterfaceSelection::<InterfaceContract>::new(
        interface.key().clone(),
        b"accepted-ea-revision",
    );
    let OperationOutcome::Completed(capability) = resolved.interface(&selected_interface) else {
        panic!("the independently compiled selection should mint a capability");
    };
    assert!(matches!(
        capability.authorize(&request, &selected_interface),
        OperationOutcome::Completed(())
    ));

    let artifact_as_interface = InterfaceSelection::<InterfaceContract>::new(
        artifact.key().clone(),
        artifact.revision().expect("pinned revision").as_bytes(),
    );
    let OperationOutcome::Refused(wrong_subject) = resolved.interface(&artifact_as_interface)
    else {
        panic!("artifact state cannot authenticate an interface selection");
    };
    assert_eq!(
        wrong_subject.code().as_str(),
        ResolutionRefusalCode::InterfaceSelectionMismatch.as_str()
    );
    let ProtocolCause::WrongAnchor { expected, observed } = only_protocol_cause(&wrong_subject)
    else {
        panic!("a subject-kind mismatch must retain both exact kinds");
    };
    assert_eq!(expected.as_bytes(), b"interface");
    assert_eq!(observed.as_bytes(), b"artifact");
}

#[test]
#[trace("TC-013", "FR-009-AC-2", "FR-009-AC-3", "FR-012-AC-6")]
fn tc_013_absent_unpinned_and_unresolved_entries_refuse_exactly() {
    let required = key("A", "compiled/2");
    let clock = || ScriptedClock::new(&[0, 0, 1, 1]);
    let absent = take_refusal(resolve(
        request("r-absent"),
        &VersionLockManifest::new(vec![]),
        std::slice::from_ref(&required),
        &[],
        budget(1, 1, 100),
        clock(),
    ));
    assert_eq!(
        absent.code().as_str(),
        ResolutionRefusalCode::EntryAbsent.as_str()
    );
    assert_retained_identity(&absent, &required.identity_bytes());

    let unpinned = ManifestEntry::pinned_artifact(
        required.clone(),
        None,
        Some(b"asserted"),
        COMPILED_MEMBER.as_bytes(),
        ByteDigest::of(ARTIFACT_BYTES),
    );
    let unpinned = take_refusal(resolve(
        request("r-unpinned"),
        &VersionLockManifest::new(vec![unpinned]),
        std::slice::from_ref(&required),
        &[delivery(COMPILED_MEMBER, ARTIFACT_BYTES)],
        budget(1, ARTIFACT_BYTES.len(), 100),
        clock(),
    ));
    assert_eq!(
        unpinned.code().as_str(),
        ResolutionRefusalCode::RevisionAbsent.as_str()
    );
    assert_retained_identity(&unpinned, &required.identity_bytes());

    let unresolved = ManifestEntry::unresolved_artifact(
        required.clone(),
        COMPILED_MEMBER.as_bytes(),
        b"owner contract not accepted",
    );
    let other_version = artifact_entry("A", "compiled/1", V1_MEMBER, b"v1");
    let unresolved = take_refusal(resolve(
        request("r-unresolved"),
        &VersionLockManifest::new(vec![other_version, unresolved]),
        std::slice::from_ref(&required),
        &[],
        budget(2, 1, 100),
        clock(),
    ));
    assert_eq!(
        unresolved.code().as_str(),
        ResolutionRefusalCode::EntryUnresolved.as_str()
    );
    assert_retained_identity(&unresolved, &required.identity_bytes());
    let ProtocolCause::IncompleteEvidence { evidence } = only_protocol_cause(&unresolved) else {
        panic!("an unresolved entry must retain its exact reason");
    };
    assert_eq!(evidence.as_bytes(), b"owner contract not accepted");
}

#[test]
#[trace(
    "TC-013",
    "FR-009-AC-5",
    "FR-009-AC-9",
    "FR-012-AC-1",
    "FR-012-AC-2",
    "FR-012-AC-7"
)]
fn tc_013_manifest_order_changes_neither_first_cause_nor_partial_state() {
    let a = key("A", "compiled/2");
    let z = key("Z", "runtime/1");
    let bad_a = ManifestEntry::pinned_artifact(
        a.clone(),
        None,
        Some(b"asserted"),
        A_MEMBER.as_bytes(),
        ByteDigest::of(b"a"),
    );
    let bad_z = ManifestEntry::pinned_artifact(
        z.clone(),
        None,
        Some(b"asserted"),
        Z_MEMBER.as_bytes(),
        ByteDigest::of(b"z"),
    );

    let outcomes = [
        VersionLockManifest::new(vec![bad_z.clone(), bad_a.clone()]),
        VersionLockManifest::new(vec![bad_a.clone(), bad_z.clone()]),
    ]
    .map(|manifest| {
        resolve(
            request("ordered-request"),
            &manifest,
            &[z.clone(), a.clone()],
            &[delivery(Z_MEMBER, b"z"), delivery(A_MEMBER, b"a")],
            budget(2, 2, 100),
            ScriptedClock::new(&[0, 0, 1, 1]),
        )
    });
    let [OperationOutcome::Refused(first_refusal), OperationOutcome::Refused(second_refusal)] =
        &outcomes
    else {
        panic!("every manifest permutation must refuse");
    };
    assert_eq!(first_refusal, second_refusal);
    let refusal = first_refusal;
    assert_eq!(
        refusal.code().as_str(),
        ResolutionRefusalCode::RevisionAbsent.as_str()
    );
    let AffectedIdentity::Retained(identity) = refusal.identity() else {
        panic!("an entry refusal must retain its exact key identity");
    };
    assert_eq!(identity.as_bytes(), a.identity_bytes());
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);

    let duplicate = artifact_entry("A", "compiled/2", A_MEMBER, b"a");
    for entries in [
        vec![bad_z.clone(), duplicate.clone(), duplicate.clone()],
        vec![duplicate.clone(), bad_z.clone(), duplicate.clone()],
        vec![duplicate.clone(), duplicate.clone(), bad_z.clone()],
    ] {
        let refusal = take_refusal(resolve(
            request("duplicate-request"),
            &VersionLockManifest::new(entries),
            std::slice::from_ref(&z),
            &[delivery(Z_MEMBER, b"z")],
            budget(3, 1, 100),
            ScriptedClock::new(&[0, 0, 1, 1]),
        ));
        assert_eq!(
            refusal.code().as_str(),
            ResolutionRefusalCode::DuplicateEntryKey.as_str()
        );
        assert_retained_identity(&refusal, &duplicate.key().identity_bytes());
    }

    let pinned_a = interface_entry("A", "interface/1", b"a-revision");
    let pinned_z = interface_entry("Z", "interface/1", b"z-revision");
    let required = vec![pinned_z.key().clone(), pinned_a.key().clone()];
    let first = completed(resolve(
        request("permutation-success"),
        &VersionLockManifest::new(vec![pinned_z.clone(), pinned_a.clone()]),
        &required,
        &[],
        budget(2, 1, 100),
        generous_clock(),
    ));
    let second = completed(resolve(
        request("permutation-success"),
        &VersionLockManifest::new(vec![pinned_a, pinned_z]),
        &required,
        &[],
        budget(2, 1, 100),
        generous_clock(),
    ));
    assert_eq!(first.entries().len(), second.entries().len());
    for (left, right) in first.entries().iter().zip(second.entries()) {
        assert_entry_exact(left, right);
    }

    let a_v1 = interface_entry("A", "interface/1", b"a-v1-revision");
    let a_v2 = interface_entry("A", "interface/2", b"a-v2-revision");
    let original = completed(resolve(
        request("independent-versions"),
        &VersionLockManifest::new(vec![a_v1.clone(), a_v2.clone()]),
        &[a_v1.key().clone(), a_v2.key().clone()],
        &[],
        budget(2, 1, 100),
        generous_clock(),
    ));
    let repinned_v1 = ManifestEntry::pinned_interface(
        a_v1.key().clone(),
        Some(b"repinned-v1-revision"),
        Some(b"new owner assertion"),
        None,
    );
    assert_eq!(a_v1, repinned_v1, "revision is state, not entry identity");
    assert_ne!(a_v1, a_v2, "contract version is an identity component");
    assert_ne!(
        a_v1,
        interface_entry("B", "interface/1", b"a-v1-revision"),
        "owner is an identity component"
    );
    let repinned = completed(resolve(
        request("independent-versions"),
        &VersionLockManifest::new(vec![a_v2.clone(), repinned_v1]),
        &[a_v2.key().clone(), a_v1.key().clone()],
        &[],
        budget(2, 1, 100),
        generous_clock(),
    ));
    assert_ne!(
        original.entries()[0].revision(),
        repinned.entries()[0].revision()
    );
    assert_entry_exact(&original.entries()[1], &a_v2);
    assert_entry_exact(&repinned.entries()[1], &a_v2);

    let removed_v1 = completed(resolve(
        request("independent-versions"),
        &VersionLockManifest::new(vec![a_v2.clone()]),
        &[a_v2.key().clone()],
        &[],
        budget(1, 1, 100),
        generous_clock(),
    ));
    assert_eq!(removed_v1.entries().len(), 1);
    assert_entry_exact(&removed_v1.entries()[0], &a_v2);

    let unrelated = interface_entry("B", "interface/1", b"b-revision");
    let added_unrequired = completed(resolve(
        request("independent-versions"),
        &VersionLockManifest::new(vec![unrelated, a_v2.clone()]),
        &[a_v2.key().clone()],
        &[],
        budget(2, 1, 100),
        generous_clock(),
    ));
    assert_eq!(added_unrequired.entries().len(), 1);
    assert_entry_exact(&added_unrequired.entries()[0], &a_v2);
}

#[test]
#[trace("TC-013", "FR-009-AC-2", "FR-012-AC-2", "FR-012-AC-6", "NFR-006-AC-6")]
fn tc_013_every_manifest_and_delivery_refusal_axis_is_distinct() {
    let required = key("A", "compiled/2");
    let cases = [
        (
            VersionLockManifest::new(vec![ManifestEntry::pinned_artifact(
                required.clone(),
                Some(b"revision"),
                None,
                COMPILED_MEMBER.as_bytes(),
                ByteDigest::of(ARTIFACT_BYTES),
            )]),
            vec![delivery(COMPILED_MEMBER, ARTIFACT_BYTES)],
            ResolutionRefusalCode::ImmutabilityAssertionAbsent,
        ),
        (
            VersionLockManifest::new(vec![ManifestEntry::pinned_interface(
                required.clone(),
                Some(b"revision"),
                Some(b"asserted"),
                Some(ByteDigest::of(ARTIFACT_BYTES)),
            )]),
            vec![],
            ResolutionRefusalCode::InterfaceDigestPresent,
        ),
        (
            VersionLockManifest::new(vec![ManifestEntry::pinned_artifact(
                required.clone(),
                Some(b"revision"),
                Some(b"asserted"),
                b"",
                ByteDigest::of(ARTIFACT_BYTES),
            )]),
            vec![],
            ResolutionRefusalCode::ArtifactMemberAbsent,
        ),
        (
            VersionLockManifest::new(vec![artifact_entry(
                "A",
                "compiled/2",
                COMPILED_MEMBER,
                ARTIFACT_BYTES,
            )]),
            vec![],
            ResolutionRefusalCode::DeliveryMemberAbsent,
        ),
        (
            VersionLockManifest::new(vec![ManifestEntry::unresolved_interface(
                required.clone(),
                b"",
            )]),
            vec![],
            ResolutionRefusalCode::UnresolvedReasonAbsent,
        ),
    ];

    for (manifest, deliveries, expected) in cases {
        let refusal = take_refusal(resolve(
            request(expected.as_str()),
            &manifest,
            std::slice::from_ref(&required),
            &deliveries,
            budget(1, ARTIFACT_BYTES.len(), 100),
            generous_clock(),
        ));
        assert_eq!(refusal.code().as_str(), expected.as_str());
        assert_retained_identity(&refusal, &required.identity_bytes());
    }

    let selected = artifact_entry("A", "compiled/2", COMPILED_MEMBER, ARTIFACT_BYTES);
    let manifest = VersionLockManifest::new(vec![selected.clone()]);
    for (deliveries, expected) in [
        (
            vec![
                delivery(COMPILED_MEMBER, ARTIFACT_BYTES),
                delivery(EXTRA_MEMBER, b"extra"),
            ],
            ResolutionRefusalCode::UnexpectedDeliveryMember,
        ),
        (
            vec![
                delivery(COMPILED_MEMBER, ARTIFACT_BYTES),
                delivery(COMPILED_MEMBER, ARTIFACT_BYTES),
            ],
            ResolutionRefusalCode::DuplicateDeliveryMember,
        ),
    ] {
        let refusal = take_refusal(resolve(
            request(expected.as_str()),
            &manifest,
            std::slice::from_ref(&required),
            &deliveries,
            budget(1, ARTIFACT_BYTES.len(), 100),
            generous_clock(),
        ));
        assert_eq!(refusal.code().as_str(), expected.as_str());
        let expected_member = match expected {
            ResolutionRefusalCode::UnexpectedDeliveryMember => EXTRA_MEMBER,
            ResolutionRefusalCode::DuplicateDeliveryMember => COMPILED_MEMBER,
            _ => unreachable!("the table contains only delivery-member refusals"),
        };
        assert_retained_identity(&refusal, expected_member.as_bytes());
    }

    let duplicate_required = take_refusal(resolve(
        request("duplicate-required"),
        &manifest,
        &[required.clone(), required.clone()],
        &[delivery(COMPILED_MEMBER, ARTIFACT_BYTES)],
        budget(2, ARTIFACT_BYTES.len(), 100),
        generous_clock(),
    ));
    assert_eq!(
        duplicate_required.code().as_str(),
        ResolutionRefusalCode::DuplicateRequiredKey.as_str()
    );
    assert_retained_identity(&duplicate_required, &required.identity_bytes());
}

#[test]
#[trace(
    "TC-013",
    "FR-009-AC-1",
    "FR-009-AC-2",
    "FR-009-AC-7",
    "FR-012-AC-8",
    "NFR-006-AC-7"
)]
fn tc_013_only_the_exact_delivered_bytes_are_verified_and_released() {
    let canonicalized = br#"{"profile":"exact-delivery"}"#;
    let entry = artifact_entry("A", "compiled/2", COMPILED_MEMBER, ARTIFACT_BYTES);
    let manifest = VersionLockManifest::new(vec![entry.clone()]);

    let digest_mismatch = take_refusal(resolve(
        request("changed-bytes"),
        &manifest,
        &[entry.key().clone()],
        &[delivery(COMPILED_MEMBER, canonicalized)],
        budget(1, canonicalized.len(), 100),
        ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3]),
    ));
    assert_eq!(
        digest_mismatch.code().as_str(),
        ResolutionRefusalCode::DigestMismatch.as_str()
    );
    assert_retained_identity(&digest_mismatch, &entry.key().identity_bytes());
    assert_eq!(entry.digest(), Some(&ByteDigest::of(ARTIFACT_BYTES)));
    let absent = take_refusal(resolve(
        request("missing-delivery"),
        &manifest,
        &[entry.key().clone()],
        &[],
        budget(1, canonicalized.len(), 100),
        ScriptedClock::new(&[0, 0, 1, 1, 2]),
    ));
    assert_eq!(
        absent.code().as_str(),
        ResolutionRefusalCode::DeliveryMemberAbsent.as_str()
    );
    assert_retained_identity(&absent, &entry.key().identity_bytes());
    assert_ne!(
        digest_mismatch.code(),
        absent.code(),
        "substituted bytes and an absent member must remain distinct outcomes"
    );
}

#[test]
#[trace("TC-013", "FR-009-AC-6", "FR-016-AC-8")]
fn tc_013_each_resolution_budget_is_inclusive_one_short_and_fresh() {
    let first = artifact_entry("A", "compiled/1", V1_MEMBER, b"v1");
    let second = artifact_entry("A", "compiled/2", V2_MEMBER, b"v2");
    let manifest = VersionLockManifest::new(vec![first.clone(), second.clone()]);
    let required = vec![first.key().clone(), second.key().clone()];
    let deliveries = vec![delivery(V1_MEMBER, b"v1"), delivery(V2_MEMBER, b"v2")];

    let exact = completed(resolve(
        request("exact-budget"),
        &manifest,
        &required,
        &deliveries,
        budget(2, 4, 6),
        ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3, 3, 6]),
    ));
    assert_eq!(exact.entries().len(), 2);
    assert_entry_exact(&exact.entries()[0], &first);
    assert_entry_exact(&exact.entries()[1], &second);

    for (short, expected) in [
        (budget(1, 4, 100), BudgetDimension::ResolutionEntryCount),
        (budget(2, 3, 100), BudgetDimension::ResolutionBytesDigested),
        (budget(2, 4, 5), BudgetDimension::ResolutionDeadline),
    ] {
        let OperationOutcome::ResourceIncomplete(incomplete) = resolve(
            request("one-short"),
            &manifest,
            &required,
            &deliveries,
            short,
            ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3, 3, 6]),
        ) else {
            panic!("one-short budget should be resource-incomplete");
        };
        assert_eq!(incomplete.dimension(), expected);
    }

    assert!(matches!(
        resolve(
            request("fresh-budget"),
            &manifest,
            &required,
            &deliveries,
            budget(2, 4, 100),
            ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3, 3, 4]),
        ),
        OperationOutcome::Completed(_)
    ));
}

#[test]
#[trace("TC-013", "FR-009-AC-6", "FR-016-AC-8")]
fn tc_013_resolution_budget_constructor_rejects_unrepresentable_grants() {
    for (result, expected_dimension) in [
        (
            ResolutionBudget::new(0, 1, Duration::from_nanos(1)),
            BudgetDimension::ResolutionEntryCount,
        ),
        (
            ResolutionBudget::new(1, 0, Duration::from_nanos(1)),
            BudgetDimension::ResolutionBytesDigested,
        ),
        (
            ResolutionBudget::new(1, 1, Duration::ZERO),
            BudgetDimension::ResolutionDeadline,
        ),
    ] {
        assert!(matches!(
            result,
            Err(ResolutionBudgetError::InvalidGrant(
                BudgetError::NonPositiveGrant { dimension }
            )) if dimension == expected_dimension
        ));
    }
    assert!(matches!(
        ResolutionBudget::new(1, 1, Duration::MAX),
        Err(ResolutionBudgetError::DeadlineOutOfRange { .. })
    ));
}

#[test]
#[trace("TC-013", "FR-009-AC-8", "FR-012-AC-1")]
fn tc_013_identity_constructors_reject_absent_components() {
    assert_eq!(
        EntryKey::new(b"", b"compiled/2"),
        Err(EntryKeyError::EmptyOwner)
    );
    assert_eq!(
        EntryKey::new(b"A", b""),
        Err(EntryKeyError::EmptyContractVersion)
    );
    assert_eq!(
        ResolutionRequestIdentity::new(b""),
        Err(EmptyResolutionRequestIdentity)
    );
}

#[test]
#[trace("TC-013", "FR-009-AC-8")]
fn tc_013_manifest_revision_and_capability_reuse_cannot_authorize() {
    let entry = interface_entry("EA", "producer-execution/v1", b"accepted-revision");
    let manifest = VersionLockManifest::new(vec![entry.clone()]);
    let first_request = request("first-request");
    let resolved = completed(resolve(
        first_request.clone(),
        &manifest,
        &[entry.key().clone()],
        &[],
        budget(1, 1, 100),
        ScriptedClock::new(&[0, 0, 1, 1, 2, 2, 3]),
    ));

    let changed_revision = InterfaceSelection::<InterfaceContract>::new(
        entry.key().clone(),
        b"manifest-only-revision",
    );
    let OperationOutcome::Refused(mismatch) = resolved.interface(&changed_revision) else {
        panic!("a manifest-only revision cannot authenticate an interface");
    };
    assert_eq!(
        mismatch.code().as_str(),
        ResolutionRefusalCode::InterfaceSelectionMismatch.as_str()
    );
    let ProtocolCause::IdentityMismatch { expected, observed } = only_protocol_cause(&mismatch)
    else {
        panic!("a selection mismatch must retain both exact revisions");
    };
    assert_eq!(expected.as_bytes(), b"manifest-only-revision");
    assert_eq!(observed.as_bytes(), b"accepted-revision");

    let accepted =
        InterfaceSelection::<InterfaceContract>::new(entry.key().clone(), b"accepted-revision");
    let OperationOutcome::Completed(capability) = resolved.interface(&accepted) else {
        panic!("accepted interface should mint a capability");
    };
    let OperationOutcome::Refused(reused) =
        capability.authorize(&request("other-request"), &accepted)
    else {
        panic!("a capability must not authorize another request");
    };
    assert_eq!(
        reused.code().as_str(),
        ResolutionRefusalCode::CapabilityRequestMismatch.as_str()
    );

    let artifact = artifact_entry("A", "compiled/2", COMPILED_MEMBER, ARTIFACT_BYTES);
    let artifact_manifest = VersionLockManifest::new(vec![artifact.clone()]);
    let artifact_request = request("artifact-request");
    let artifact_resolved = completed(resolve(
        artifact_request.clone(),
        &artifact_manifest,
        &[artifact.key().clone()],
        &[delivery(COMPILED_MEMBER, ARTIFACT_BYTES)],
        budget(1, ARTIFACT_BYTES.len(), 100),
        generous_clock(),
    ));
    let accepted_artifact = ArtifactSelection::<ArtifactContract>::new(
        artifact.key().clone(),
        artifact.revision().expect("revision").as_bytes(),
    );
    let OperationOutcome::Completed(artifact_capability) =
        artifact_resolved.artifact(&accepted_artifact)
    else {
        panic!("accepted artifact should mint a capability");
    };
    let changed_selection =
        ArtifactSelection::<ArtifactContract>::new(artifact.key().clone(), b"different-revision");
    let OperationOutcome::Refused(changed_contract) =
        artifact_capability.bytes_for(&artifact_request, &changed_selection)
    else {
        panic!("a capability cannot authorize a changed artifact selection");
    };
    assert_eq!(
        changed_contract.code().as_str(),
        ResolutionRefusalCode::CapabilityContractMismatch.as_str()
    );
    let ProtocolCause::IdentityMismatch { expected, observed } =
        only_protocol_cause(&changed_contract)
    else {
        panic!("a changed capability selection must retain both exact revisions");
    };
    assert_eq!(expected.as_bytes(), b"revision-A-compiled/2");
    assert_eq!(observed.as_bytes(), b"different-revision");

    let changed_key = ArtifactSelection::<ArtifactContract>::new(
        key("B", "compiled/2"),
        artifact.revision().expect("revision").as_bytes(),
    );
    let OperationOutcome::Refused(changed_contract) =
        artifact_capability.bytes_for(&artifact_request, &changed_key)
    else {
        panic!("a capability cannot authorize another owner/version key");
    };
    assert_eq!(
        changed_contract.code().as_str(),
        ResolutionRefusalCode::CapabilityContractMismatch.as_str()
    );
    let ProtocolCause::IdentityMismatch { expected, observed } =
        only_protocol_cause(&changed_contract)
    else {
        panic!("a changed capability key must retain both exact keys");
    };
    assert_eq!(expected.as_bytes(), artifact.key().identity_bytes());
    assert_eq!(observed.as_bytes(), changed_key.key.identity_bytes());
}

#[test]
#[trace("TC-013", "FR-009-AC-2")]
fn tc_013_refusal_axes_are_unique_and_registered() {
    for (index, code) in ResolutionRefusalCode::ALL.iter().enumerate() {
        assert_eq!(
            catalog::lookup(code.as_str()).map(|registered| registered.as_str()),
            Some(code.as_str()),
            "every emitted resolver code must be registered"
        );
        for (other_index, other) in ResolutionRefusalCode::ALL.iter().enumerate() {
            assert_eq!(
                index == other_index,
                code.as_str() == other.as_str(),
                "distinct refusal axes cannot share an identity"
            );
        }
    }
}

#[test]
#[trace("TC-013", "FR-009-AC-4", "FR-009-AC-7", "FR-012-AC-5")]
fn tc_013_resolution_has_no_discovery_or_self_authorizing_input() {
    let source = include_str!("../version_lock.rs");
    for forbidden in [
        "std::fs",
        "std::net",
        "std::process",
        "Command::new",
        "git2",
        "sidecar:",
        "delivery_index:",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden resolver surface: {forbidden}"
        );
    }
}

/// A delivery index is consumer evidence only: the resolver's complete source
/// surface has no index parameter, import, or fallback. The positive resolver
/// tests above exercise the same delivered bytes and manifest without an index;
/// this companion check prevents a later repair path from adding one.
#[test]
#[trace("TC-017", "FR-018-AC-1")]
fn tc_017_delivery_index_cannot_supply_or_repair_version_lock_resolution() {
    let source = include_str!("../version_lock.rs");
    for forbidden in ["DeliveryIndex", "IndexRead", "delivery_index"] {
        assert!(
            !source.contains(forbidden),
            "a consumer delivery index must not enter resolver authority: {forbidden}"
        );
    }
    assert!(
        source.contains("pub(crate) fn resolve<C: Clock + 'static>("),
        "the resolver remains the explicit manifest/delivery-only authority boundary"
    );
}

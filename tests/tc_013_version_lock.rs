// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-013: B-owned version-lock resolution, with no external delivery.

use std::cell::Cell;
use std::time::{Duration, Instant};

use ix_trace_rs::trace;
use quire_protocol::bounds::Clock;
use quire_protocol::refusal::catalog::lookup;
use quire_protocol::version_lock::{
    resolve, ArtifactDelivery, EntryKey, EntryStatus, ExactBytes, ManifestEntry, ResolutionBudget,
    ResolutionDimension, ResolutionIncomplete, ResolutionOutcome, ResolutionRefusalCode,
    VersionLockManifest,
};
use quire_protocol::ByteDigest;

const ARTIFACT_BYTES: &[u8] = b"{\n  \"profile\": \"exact-delivery\"\n}\n";

fn key(owner: &str, version: &str) -> EntryKey {
    EntryKey::new(owner.as_bytes(), version.as_bytes()).expect("non-empty test key")
}

fn artifact_entry(owner: &str, version: &str, member: &str, bytes: &[u8]) -> ManifestEntry {
    ManifestEntry::pinned_artifact(
        key(owner, version),
        Some(b"0123456789abcdef"),
        Some(b"owner asserts immutable and published"),
        member.as_bytes(),
        ByteDigest::of(bytes),
    )
}

fn delivery(member: &str, bytes: &[u8]) -> ArtifactDelivery {
    ArtifactDelivery::new(member.as_bytes(), bytes)
}

#[derive(Debug)]
struct ScriptedClock {
    origin: Instant,
    offsets: Vec<Duration>,
    next: Cell<usize>,
}

impl ScriptedClock {
    fn new(offsets: &[Duration]) -> Self {
        Self {
            origin: Instant::now(),
            offsets: offsets.to_vec(),
            next: Cell::new(0),
        }
    }
}

impl Clock for ScriptedClock {
    fn monotonic(&self) -> Instant {
        let index = self.next.get();
        self.next.set(index + 1);
        self.origin + self.offsets[index.min(self.offsets.len() - 1)]
    }
}

fn generous_budget(entries: usize, bytes: usize) -> ResolutionBudget {
    ResolutionBudget::new(entries, bytes, Duration::from_secs(1))
}

#[test]
#[trace("TC-013", "FR-009-AC-1", "FR-009-AC-5", "FR-012-AC-3", "FR-012-AC-4")]
fn tc_013_resolves_artifacts_and_interfaces_without_changing_either() {
    let a_v1 = artifact_entry("A", "1", "compiled-v1.json", ARTIFACT_BYTES);
    let a_v2 = artifact_entry("A", "2", "compiled-v2.json", b"version two");
    let quoin = ManifestEntry::pinned_interface(
        key("Quoin", "evidence/1"),
        Some(b"fedcba9876543210"),
        Some(b"owner asserts immutable and published"),
        None,
    );
    let authored_entries = vec![a_v1.clone(), a_v2.clone(), quoin.clone()];
    let manifest = VersionLockManifest::new(authored_entries.clone());
    let deliveries = vec![
        delivery("compiled-v1.json", ARTIFACT_BYTES),
        delivery("compiled-v2.json", b"version two"),
    ];
    let required = vec![a_v1.key().clone(), a_v2.key().clone(), quoin.key().clone()];
    let clock = ScriptedClock::new(&[Duration::ZERO]);

    let ResolutionOutcome::Resolved(resolved) = resolve(
        &manifest,
        &required,
        &deliveries,
        generous_budget(3, ARTIFACT_BYTES.len() + b"version two".len()),
        &clock,
    ) else {
        panic!("a fully pinned manifest should resolve");
    };

    assert_eq!(resolved.entries(), authored_entries);
    for (resolved_entry, authored_entry) in resolved.entries().iter().zip(authored_entries.iter()) {
        assert_eq!(resolved_entry.key(), authored_entry.key());
        assert_eq!(resolved_entry.kind(), authored_entry.kind());
        assert_eq!(resolved_entry.status(), authored_entry.status());
        assert_eq!(resolved_entry.revision(), authored_entry.revision());
        assert_eq!(
            resolved_entry.immutability_assertion(),
            authored_entry.immutability_assertion()
        );
        assert_eq!(resolved_entry.member(), authored_entry.member());
        assert_eq!(resolved_entry.digest(), authored_entry.digest());
    }
    assert_eq!(
        resolved
            .artifact_bytes(a_v1.key())
            .expect("artifact bytes retained"),
        ARTIFACT_BYTES,
        "the consumer receives the exact bytes whose digest was checked"
    );
    assert_eq!(
        resolved.artifact_bytes(a_v2.key()),
        Some(&b"version two"[..])
    );
    assert_eq!(resolved.artifact_bytes(&key("Quoin", "evidence/1")), None);
    assert_eq!(resolved.usage().entries(), 3);
    assert_eq!(
        resolved.usage().bytes_digested(),
        ARTIFACT_BYTES.len() + b"version two".len()
    );

    let repinned_v1 = ManifestEntry::pinned_artifact(
        a_v1.key().clone(),
        Some(b"new-revision"),
        Some(b"new owner assertion"),
        b"compiled-v1.json",
        ByteDigest::of(ARTIFACT_BYTES),
    );
    assert_eq!(
        a_v1.key(),
        repinned_v1.key(),
        "revision and assertion are state, not entry identity"
    );
    assert_eq!(a_v2, resolved.entries()[1], "repinning v1 cannot alter v2");
}

#[test]
#[trace("TC-013", "FR-009-AC-2", "FR-012-AC-2", "FR-012-AC-6")]
fn tc_013_each_invalid_entry_axis_has_its_own_typed_outcome() {
    let required = key("A", "1");
    let clock = ScriptedClock::new(&[Duration::ZERO]);
    let budget = generous_budget(2, ARTIFACT_BYTES.len());

    let cases = [
        (
            VersionLockManifest::new(vec![]),
            vec![],
            ResolutionRefusalCode::EntryAbsent,
        ),
        (
            VersionLockManifest::new(vec![ManifestEntry::pinned_artifact(
                required.clone(),
                None,
                Some(b"asserted"),
                b"compiled.json",
                ByteDigest::of(ARTIFACT_BYTES),
            )]),
            vec![delivery("compiled.json", ARTIFACT_BYTES)],
            ResolutionRefusalCode::RevisionAbsent,
        ),
        (
            VersionLockManifest::new(vec![ManifestEntry::pinned_artifact(
                required.clone(),
                Some(b"revision"),
                None,
                b"compiled.json",
                ByteDigest::of(ARTIFACT_BYTES),
            )]),
            vec![delivery("compiled.json", ARTIFACT_BYTES)],
            ResolutionRefusalCode::ImmutabilityAssertionAbsent,
        ),
        (
            VersionLockManifest::new(vec![artifact_entry("A", "1", "compiled.json", b"other")]),
            vec![delivery("compiled.json", ARTIFACT_BYTES)],
            ResolutionRefusalCode::DigestMismatch,
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
            VersionLockManifest::new(vec![artifact_entry("A", "1", "", ARTIFACT_BYTES)]),
            vec![],
            ResolutionRefusalCode::ArtifactMemberAbsent,
        ),
        (
            VersionLockManifest::new(vec![artifact_entry(
                "A",
                "1",
                "compiled.json",
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

    for (manifest, deliveries, expected_code) in cases {
        let ResolutionOutcome::Refused(refusal) = resolve(
            &manifest,
            std::slice::from_ref(&required),
            &deliveries,
            budget,
            &clock,
        ) else {
            panic!("{expected_code:?} case should refuse");
        };
        assert_eq!(refusal.code(), expected_code);
        assert_eq!(refusal.key(), Some(&required));
        assert!(refusal.resolved_before().is_empty());
    }

    let duplicate = artifact_entry("A", "1", "compiled.json", ARTIFACT_BYTES);
    let manifest = VersionLockManifest::new(vec![duplicate.clone(), duplicate]);
    let ResolutionOutcome::Refused(refusal) = resolve(
        &manifest,
        std::slice::from_ref(&required),
        &[delivery("compiled.json", ARTIFACT_BYTES)],
        budget,
        &clock,
    ) else {
        panic!("a duplicate manifest key should refuse");
    };
    assert_eq!(refusal.code(), ResolutionRefusalCode::DuplicateEntryKey);
    assert_eq!(refusal.key(), Some(&required));

    let single = artifact_entry("A", "1", "compiled.json", ARTIFACT_BYTES);
    let manifest = VersionLockManifest::new(vec![single]);
    let ResolutionOutcome::Refused(refusal) = resolve(
        &manifest,
        &[required.clone(), required.clone()],
        &[delivery("compiled.json", ARTIFACT_BYTES)],
        budget,
        &clock,
    ) else {
        panic!("a duplicate required key should refuse");
    };
    assert_eq!(refusal.code(), ResolutionRefusalCode::DuplicateRequiredKey);
    assert_eq!(refusal.key(), Some(&required));

    let ResolutionOutcome::Refused(refusal) = resolve(
        &manifest,
        std::slice::from_ref(&required),
        &[
            delivery("compiled.json", ARTIFACT_BYTES),
            delivery("compiled.json", ARTIFACT_BYTES),
        ],
        budget,
        &clock,
    ) else {
        panic!("a duplicate delivery member should refuse");
    };
    assert_eq!(
        refusal.code(),
        ResolutionRefusalCode::DuplicateDeliveryMember
    );
    assert_eq!(refusal.member(), Some(b"compiled.json".as_slice()));
}

#[test]
#[trace("TC-013", "FR-009-AC-3", "FR-012-AC-6")]
fn tc_013_unresolved_is_unexecutable_and_cannot_substitute_another_version() {
    let unresolved = ManifestEntry::unresolved_artifact(
        key("A", "2"),
        b"compiled-v2.json",
        b"owner has not published an immutable revision",
    );
    let manifest = VersionLockManifest::new(vec![
        artifact_entry("A", "1", "compiled-v1.json", ARTIFACT_BYTES),
        unresolved.clone(),
    ]);
    let clock = ScriptedClock::new(&[Duration::ZERO]);

    let ResolutionOutcome::Unexecutable(outcome) = resolve(
        &manifest,
        &[key("A", "2")],
        &[],
        generous_budget(2, ARTIFACT_BYTES.len()),
        &clock,
    ) else {
        panic!("an unresolved required entry is neither pass nor fail");
    };
    assert_eq!(outcome.key(), unresolved.key());
    assert_eq!(
        outcome.reason(),
        b"owner has not published an immutable revision"
    );
    assert!(outcome.resolved_before().is_empty());
}

#[test]
#[trace("TC-013", "FR-009-AC-1", "FR-009-AC-2", "NFR-006-AC-6", "NFR-006-AC-7")]
fn tc_013_hashes_delivered_bytes_and_rejects_unselected_delivery_members() {
    let canonical = br#"{\"profile\":\"exact-delivery\"}"#;
    assert_ne!(ARTIFACT_BYTES, canonical);
    let entry = artifact_entry("A", "1", "compiled.json", ARTIFACT_BYTES);
    let manifest = VersionLockManifest::new(vec![entry.clone()]);
    let clock = ScriptedClock::new(&[Duration::ZERO]);
    let budget = generous_budget(1, ARTIFACT_BYTES.len());

    assert!(matches!(
        resolve(
            &manifest,
            &[entry.key().clone()],
            &[delivery("compiled.json", ARTIFACT_BYTES)],
            budget,
            &clock,
        ),
        ResolutionOutcome::Resolved(_)
    ));
    let ResolutionOutcome::Refused(refusal) = resolve(
        &manifest,
        &[entry.key().clone()],
        &[delivery("compiled.json", canonical)],
        budget,
        &clock,
    ) else {
        panic!("a canonical re-encoding is different delivered bytes");
    };
    assert_eq!(refusal.code(), ResolutionRefusalCode::DigestMismatch);
    assert_eq!(entry.digest(), Some(&ByteDigest::of(ARTIFACT_BYTES)));

    let ResolutionOutcome::Refused(extra) = resolve(
        &manifest,
        &[entry.key().clone()],
        &[
            delivery("compiled.json", ARTIFACT_BYTES),
            delivery("unselected.bin", b"not covered by a pinned entry"),
        ],
        generous_budget(1, ARTIFACT_BYTES.len()),
        &clock,
    ) else {
        panic!("an unselected delivery member cannot widen the accepted delivery");
    };
    assert_eq!(
        extra.code(),
        ResolutionRefusalCode::UnexpectedDeliveryMember
    );
    assert_eq!(extra.member(), Some(b"unselected.bin".as_slice()));
}

#[test]
#[trace("TC-013", "FR-009-AC-6", "FR-016-AC-8")]
fn tc_013_resolution_budgets_are_inclusive_typed_and_fresh_per_attempt() {
    let entry = artifact_entry("A", "1", "compiled.json", ARTIFACT_BYTES);
    let manifest = VersionLockManifest::new(vec![entry.clone()]);
    let deliveries = vec![delivery("compiled.json", ARTIFACT_BYTES)];
    let required = vec![entry.key().clone()];

    let exact_clock =
        ScriptedClock::new(&[Duration::ZERO, Duration::ZERO, Duration::from_millis(10)]);
    assert!(matches!(
        resolve(
            &manifest,
            &required,
            &deliveries,
            ResolutionBudget::new(1, ARTIFACT_BYTES.len(), Duration::from_millis(10)),
            &exact_clock,
        ),
        ResolutionOutcome::Resolved(_)
    ));

    for (budget, expected) in [
        (
            ResolutionBudget::new(0, ARTIFACT_BYTES.len(), Duration::from_millis(10)),
            ResolutionDimension::EntryCount,
        ),
        (
            ResolutionBudget::new(1, ARTIFACT_BYTES.len() - 1, Duration::from_millis(10)),
            ResolutionDimension::BytesDigested,
        ),
    ] {
        let clock = ScriptedClock::new(&[Duration::ZERO]);
        let ResolutionOutcome::Incomplete(incomplete) =
            resolve(&manifest, &required, &deliveries, budget, &clock)
        else {
            panic!("one-short budget should be incomplete");
        };
        assert_eq!(incomplete.dimension(), expected);
        assert_eq!(incomplete.consumed_entries(), 0);
        assert_eq!(incomplete.consumed_bytes(), 0);
    }

    let expired_clock =
        ScriptedClock::new(&[Duration::ZERO, Duration::ZERO, Duration::from_millis(10)]);
    let ResolutionOutcome::Incomplete(ResolutionIncomplete::Deadline(incomplete)) = resolve(
        &manifest,
        &required,
        &deliveries,
        ResolutionBudget::new(1, ARTIFACT_BYTES.len(), Duration::from_millis(9)),
        &expired_clock,
    ) else {
        panic!("one-short deadline should be incomplete");
    };
    assert_eq!(incomplete.dimension(), ResolutionDimension::Deadline);
    assert_eq!(incomplete.consumed_entries(), 1);
    assert_eq!(incomplete.consumed_bytes(), ARTIFACT_BYTES.len());

    let fresh_clock = ScriptedClock::new(&[Duration::ZERO]);
    assert!(matches!(
        resolve(
            &manifest,
            &required,
            &deliveries,
            generous_budget(1, ARTIFACT_BYTES.len()),
            &fresh_clock,
        ),
        ResolutionOutcome::Resolved(_)
    ));
}

#[test]
#[trace("TC-013", "FR-012-AC-1", "FR-012-AC-8")]
fn tc_013_entry_identity_is_only_owner_and_contract_version() {
    let original = artifact_entry("A", "1", "compiled.json", ARTIFACT_BYTES);
    let changed = ManifestEntry::pinned_artifact(
        original.key().clone(),
        Some(b"different-revision"),
        Some(b"different assertion"),
        b"renamed.json",
        ByteDigest::of(b"different bytes"),
    );
    assert_eq!(original.key(), changed.key());
    assert_ne!(original.revision(), changed.revision());
    assert_ne!(original.digest(), changed.digest());
    assert_eq!(original.status(), EntryStatus::Pinned);
    assert_eq!(changed.status(), EntryStatus::Pinned);
    assert_eq!(ExactBytes::new(b"A"), *original.key().owner());
}

#[test]
#[trace("TC-013", "FR-009-AC-2", "FR-017-AC-4")]
fn tc_013_refusal_axes_are_distinct_and_registered() {
    for (index, code) in ResolutionRefusalCode::ALL.iter().enumerate() {
        assert_eq!(
            lookup(code.as_str()).map(|registered| registered.as_str()),
            Some(code.as_str()),
            "every emitted version-lock code is in the crate-wide catalog"
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
#[trace("TC-013", "FR-009-AC-4", "FR-009-AC-7", "FR-018-AC-1", "NFR-006-AC-8")]
fn tc_013_resolution_has_no_external_or_self_authorizing_capability() {
    let source = include_str!("../src/version_lock.rs");
    for forbidden in [
        "std::fs",
        "std::process",
        "std::net",
        "Command::new",
        "TcpStream",
        "UdpSocket",
        "DeliveryIndex",
        "sidecar",
    ] {
        assert!(
            !source.contains(forbidden),
            "the resolution module gained forbidden capability {forbidden}"
        );
    }

    let _capability_free_boundary = resolve::<ScriptedClock>;
}

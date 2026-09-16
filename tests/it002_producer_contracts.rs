// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-002 SC-08: exact QObs and QSL producer-contract resolution.

use std::cell::Cell;
use std::time::{Duration, Instant};

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::BudgetDimension;
use quire_protocol::bounds::Clock;
use quire_protocol::claims::Claim;
use quire_protocol::outcome::OperationOutcome;
use quire_protocol::producer_contracts::{
    resolve_producer_contracts, ProducerContract, ProducerRequestError, ProducerResolution,
    ProducerResolutionRequest, ProducerValueSelection, UnsupportedObservationCapability,
    QOBS_REVISION,
};
use quire_protocol::refusal::{AffectedIdentity, ProtocolCause, ProtocolRefusal, RefusalCause};
use quire_protocol::version_lock::{
    EntryKey, EntryStatus, ManifestEntry, ResolutionBudget, ResolutionRefusalCode,
    VersionLockManifest,
};
use quire_protocol::ByteDigest;

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

fn generous_budget() -> ResolutionBudget {
    ResolutionBudget::new(ProducerContract::ALL.len() + 2, 1, Duration::from_secs(1))
        .expect("positive exact-contract resolution budget")
}

fn exact_budget() -> ResolutionBudget {
    ResolutionBudget::new(ProducerContract::ALL.len(), 1, Duration::from_nanos(1))
        .expect("positive exact-contract resolution budget")
}

fn generous_clock() -> ScriptedClock {
    ScriptedClock::new(&[0; 64])
}

fn manifest() -> VersionLockManifest {
    VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .map(ProducerContract::manifest_entry)
            .collect(),
    )
}

fn refusal(outcome: OperationOutcome<ProducerResolution>) -> ProtocolRefusal {
    let OperationOutcome::Refused(refusal) = outcome else {
        panic!("mutated producer-contract resolution must refuse")
    };
    refusal
}

fn key_identity(key: &EntryKey) -> AffectedIdentity {
    let mut identity = Vec::new();
    identity.extend_from_slice(&(key.owner().as_bytes().len() as u128).to_be_bytes());
    identity.extend_from_slice(key.owner().as_bytes());
    identity.extend_from_slice(&(key.contract_version().as_bytes().len() as u128).to_be_bytes());
    identity.extend_from_slice(key.contract_version().as_bytes());
    AffectedIdentity::retained(&identity)
}

fn assert_refusal(
    outcome: OperationOutcome<ProducerResolution>,
    code: ResolutionRefusalCode,
    affected: &EntryKey,
) {
    let refusal = refusal(outcome);
    assert_eq!(refusal.code().as_str(), code.as_str());
    assert_eq!(refusal.identity(), &key_identity(affected));
}

fn same_entry_state(left: &ManifestEntry, right: &ManifestEntry) -> bool {
    left.key() == right.key()
        && left.status() == right.status()
        && left.revision() == right.revision()
        && left.immutability_assertion() == right.immutability_assertion()
        && left.unresolved_reason() == right.unresolved_reason()
        && left.kind() == right.kind()
        && left.member() == right.member()
        && left.digest() == right.digest()
}

#[test]
#[trace(
    "IT-002",
    "FR-009-AC-1",
    "FR-009-AC-2",
    "FR-009-AC-3",
    "FR-009-AC-4",
    "FR-009-AC-5"
)]
fn it_002_sc_08_resolves_only_the_exact_scenario_contract_inventory() {
    const EXPECTED: [(&str, &str, &str); 21] = [
        (
            "agent-ix/quire-observation",
            "quire.observation.record/v2",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.population/v2",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.explicit-members/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.position-ledger/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.clock-binding/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.partial-fact/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.capture-environment/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.progress-assertion/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.activation-scope-authority/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.closure-assertion/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.completeness-assertion/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation.result-availability/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation-authority/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation-authority/v2",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-observation",
            "quire.observation-authority-lineage/v1",
            QOBS_REVISION,
        ),
        (
            "agent-ix/quire-spec-language",
            "quire.native-temporal-request/v1",
            "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07",
        ),
        (
            "agent-ix/quire-spec-language",
            "quire.native-temporal-result/v1",
            "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07",
        ),
        (
            "agent-ix/quire-spec-language",
            "quire.native-temporal-request/v2",
            "0c2f13e261c0ffeaf89f0638931b573ec038e99b",
        ),
        (
            "agent-ix/quire-spec-language",
            "quire.native-temporal-result/v2",
            "0c2f13e261c0ffeaf89f0638931b573ec038e99b",
        ),
        (
            "agent-ix/quire-spec-language",
            "quire.compiled-protocol/2",
            "0c2f13e261c0ffeaf89f0638931b573ec038e99b",
        ),
        (
            "agent-ix/quire-spec-language",
            "quire.compiled-protocol/3",
            "0c2f13e261c0ffeaf89f0638931b573ec038e99b",
        ),
    ];
    assert_eq!(ProducerContract::ALL.len(), EXPECTED.len());
    assert_eq!(
        ProducerContract::ALL.map(|contract| (
            contract.owner(),
            contract.contract_version(),
            contract.expected_revision(),
        )),
        EXPECTED,
    );
    assert_eq!(QOBS_REVISION, "924006300f45b38483be1cbdf99b68f899b7d368");
    let request = ProducerResolutionRequest::for_assessment(
        ByteDigest::of(b"it002:scenario-package"),
        [7; 32],
        &[Claim::Conformance],
        ProducerValueSelection::new([0x23; 32], 1),
    )
    .expect("the closed claim selection is representable");
    assert_eq!(
        ProducerResolutionRequest::for_assessment(
            ByteDigest::of(b"package:too-many-claims"),
            [7; 32],
            &[Claim::Conformance; 7],
            ProducerValueSelection::new([0x23; 32], 1),
        ),
        Err(ProducerRequestError::TooManyClaims),
    );
    let outcome =
        resolve_producer_contracts(&request, &manifest(), exact_budget(), generous_clock());
    let OperationOutcome::Completed(resolved) = outcome else {
        panic!("the exact accepted producer contracts must resolve")
    };
    assert_eq!(resolved.request(), &request);
    assert_eq!(
        resolved.manifest_entries().len(),
        ProducerContract::ALL.len()
    );
    assert!(resolved.manifest_entries().iter().all(|entry| {
        entry.status() == EntryStatus::Pinned
            && entry.immutability_assertion().is_some_and(|assertion| {
                assertion.as_bytes() == b"owner asserts immutable and published"
            })
    }));
    assert_eq!(
        resolved
            .usage()
            .consumed(BudgetDimension::ResolutionEntryCount),
        ProducerContract::ALL.len()
    );
    assert_eq!(
        resolved
            .contracts()
            .iter()
            .map(|contract| contract.contract())
            .collect::<Vec<_>>(),
        ProducerContract::ALL
    );
    for contract in resolved.contracts() {
        assert_eq!(
            contract.revision(),
            contract.contract().expected_revision().as_bytes()
        );
    }

    let qobs_schemas = ProducerContract::ALL
        .iter()
        .copied()
        .filter_map(|contract| contract.owner_schema().map(|schema| (contract, schema)))
        .collect::<Vec<_>>();
    assert_eq!(qobs_schemas.len(), 15);
    for (contract, schema) in &qobs_schemas {
        assert_eq!(contract.expected_revision(), QOBS_REVISION);
        assert_eq!(
            format!("{:x}", ByteDigest::of(schema.bytes())),
            schema.sha256()
        );
    }
    assert_eq!(
        qobs_schemas
            .iter()
            .filter(|(contract, _)| {
                matches!(
                    contract,
                    ProducerContract::ObservationPartialFact
                        | ProducerContract::ObservationActivationAuthority
                        | ProducerContract::ObservationAuthorityBundleV1
                        | ProducerContract::ObservationAuthorityBundleV2
                        | ProducerContract::ObservationAuthorityLineage
                )
            })
            .map(|(contract, schema)| (contract.contract_version(), schema.sha256()))
            .collect::<Vec<_>>(),
        vec![
            (
                "quire.observation.partial-fact/v1",
                "9dc068376a7db5a7509810910c36e2efff6b9c3701d83c474701ec572f13f23c",
            ),
            (
                "quire.observation.activation-scope-authority/v1",
                "c17ad8b3ec10c48e535bc3a762738b979206c01af20c31775d8a9a9e0a97fab6",
            ),
            (
                "quire.observation-authority/v1",
                "9614d8077bd1e667507b2107eab08e0f7934337115ce3e10c8c55b5d5538cdab",
            ),
            (
                "quire.observation-authority/v2",
                "f92335c5e5e1e3434f1c2484557ca1f512c22b496eb97319c94fcac3491f116a",
            ),
            (
                "quire.observation-authority-lineage/v1",
                "d58c70516e53bde40ab2d276df143fffb05740348b98d09a71e89857a82a22ef",
            ),
        ]
    );
    assert_eq!(
        UnsupportedObservationCapability::ALL,
        [
            UnsupportedObservationCapability::Repair,
            UnsupportedObservationCapability::AggregateQuery,
        ]
    );

    let absent = VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .filter(|contract| *contract != ProducerContract::ObservationRecord)
            .map(ProducerContract::manifest_entry)
            .collect(),
    );
    assert_refusal(
        resolve_producer_contracts(&request, &absent, generous_budget(), generous_clock()),
        ResolutionRefusalCode::EntryAbsent,
        &ProducerContract::ObservationRecord.key(),
    );

    let mut unpinned_entries = manifest().entries().to_vec();
    unpinned_entries[0] = ManifestEntry::pinned_interface(
        ProducerContract::ALL[0].key(),
        None,
        Some(b"owner asserts immutable and published"),
        None,
    );
    assert_refusal(
        resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(unpinned_entries),
            generous_budget(),
            generous_clock(),
        ),
        ResolutionRefusalCode::RevisionAbsent,
        &ProducerContract::ALL[0].key(),
    );

    let selected = ProducerContract::ALL[0];
    let mut wrong_assertion_entries = manifest().entries().to_vec();
    wrong_assertion_entries[0] = ManifestEntry::pinned_interface(
        selected.key(),
        Some(selected.expected_revision().as_bytes()),
        Some(b"a different owner assertion"),
        None,
    );
    let assertion_refusal = refusal(resolve_producer_contracts(
        &request,
        &VersionLockManifest::new(wrong_assertion_entries),
        generous_budget(),
        generous_clock(),
    ));
    assert_eq!(
        assertion_refusal.code().as_str(),
        ResolutionRefusalCode::InterfaceSelectionMismatch.as_str()
    );
    assert_eq!(assertion_refusal.identity(), &key_identity(&selected.key()));
    assert!(matches!(
        assertion_refusal.causes().declared_order(),
        [RefusalCause::Protocol(ProtocolCause::IdentityMismatch { expected, observed })]
            if expected.as_bytes() == b"owner asserts immutable and published"
                && observed.as_bytes() == b"a different owner assertion"
    ));

    let mut wrong_kind_entries = manifest().entries().to_vec();
    wrong_kind_entries[0] = ManifestEntry::pinned_artifact(
        selected.key(),
        Some(b"also-the-wrong-revision"),
        Some(b"owner asserts immutable and published"),
        b"not-an-interface",
        ByteDigest::of(b"not-an-interface"),
    );
    let kind_refusal = refusal(resolve_producer_contracts(
        &request,
        &VersionLockManifest::new(wrong_kind_entries),
        generous_budget(),
        generous_clock(),
    ));
    assert_eq!(
        kind_refusal.code().as_str(),
        ResolutionRefusalCode::InterfaceSelectionMismatch.as_str()
    );
    assert_eq!(kind_refusal.identity(), &key_identity(&selected.key()));
    assert!(matches!(
        kind_refusal.causes().declared_order(),
        [RefusalCause::Protocol(ProtocolCause::WrongAnchor { expected, observed })]
            if expected.as_bytes() == b"interface" && observed.as_bytes() == b"artifact"
    ));

    let mut duplicate_entries = manifest().entries().to_vec();
    duplicate_entries.push(ProducerContract::ObservationRecord.manifest_entry());
    assert_refusal(
        resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(duplicate_entries),
            generous_budget(),
            generous_clock(),
        ),
        ResolutionRefusalCode::DuplicateEntryKey,
        &ProducerContract::ObservationRecord.key(),
    );

    let mut unresolved_entries = manifest().entries().to_vec();
    unresolved_entries[0] = ManifestEntry::unresolved_interface(
        ProducerContract::ALL[0].key(),
        b"owner acceptance withdrawn",
    );
    assert_refusal(
        resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(unresolved_entries),
            generous_budget(),
            generous_clock(),
        ),
        ResolutionRefusalCode::EntryUnresolved,
        &ProducerContract::ALL[0].key(),
    );

    let baseline_entries = manifest().entries().to_vec();
    let mut repointed_entries = baseline_entries.clone();
    let selected = ProducerContract::NativeTemporalRequestV2;
    let selected_index = ProducerContract::ALL
        .iter()
        .position(|contract| *contract == selected)
        .expect("selected contract belongs to the exact inventory");
    repointed_entries[selected_index] = ManifestEntry::pinned_interface(
        selected.key(),
        Some(b"foreign-revision"),
        Some(b"owner asserts immutable and published"),
        None,
    );
    assert_eq!(
        repointed_entries
            .iter()
            .zip(&baseline_entries)
            .filter(|(left, right)| !same_entry_state(left, right))
            .count(),
        1,
        "repinning one contract must not alter a sibling manifest entry",
    );
    assert_refusal(
        resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(repointed_entries),
            generous_budget(),
            generous_clock(),
        ),
        ResolutionRefusalCode::InterfaceSelectionMismatch,
        &selected.key(),
    );

    for (changed, sibling) in [
        (
            ProducerContract::ObservationAuthorityBundleV1,
            ProducerContract::ObservationAuthorityBundleV2,
        ),
        (
            ProducerContract::ObservationAuthorityBundleV2,
            ProducerContract::ObservationAuthorityBundleV1,
        ),
    ] {
        let changed_index = ProducerContract::ALL
            .iter()
            .position(|contract| *contract == changed)
            .expect("changed contract belongs to the inventory");
        let sibling_index = ProducerContract::ALL
            .iter()
            .position(|contract| *contract == sibling)
            .expect("sibling contract belongs to the inventory");
        let mut entries = baseline_entries.clone();
        entries[changed_index] = ManifestEntry::pinned_interface(
            changed.key(),
            Some(b"foreign-revision"),
            Some(b"owner asserts immutable and published"),
            None,
        );
        assert!(same_entry_state(
            &entries[sibling_index],
            &baseline_entries[sibling_index],
        ));
        assert_refusal(
            resolve_producer_contracts(
                &request,
                &VersionLockManifest::new(entries),
                generous_budget(),
                generous_clock(),
            ),
            ResolutionRefusalCode::InterfaceSelectionMismatch,
            &changed.key(),
        );
    }

    let sidecar_key = EntryKey::new(b"local/diagnostics", b"sidecar/v1")
        .expect("nonempty sidecar key components");
    let resolve_with_sidecar = |revision: &'static [u8]| {
        let mut entries = manifest().entries().to_vec();
        entries.push(ManifestEntry::pinned_interface(
            sidecar_key.clone(),
            Some(revision),
            Some(b"local immutable sidecar"),
            None,
        ));
        resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(entries),
            generous_budget(),
            generous_clock(),
        )
    };
    let OperationOutcome::Completed(first_sidecar) = resolve_with_sidecar(b"sidecar-revision:one")
    else {
        panic!("a non-authoritative sidecar must not affect producer resolution")
    };
    let OperationOutcome::Completed(second_sidecar) = resolve_with_sidecar(b"sidecar-revision:two")
    else {
        panic!("repinning a non-authoritative sidecar must remain irrelevant")
    };
    assert_eq!(first_sidecar.contracts(), second_sidecar.contracts());
    assert_eq!(
        first_sidecar.manifest_entries(),
        second_sidecar.manifest_entries()
    );
    assert!(first_sidecar
        .manifest_entries()
        .iter()
        .all(|entry| entry.key() != &sidecar_key));

    let one_short =
        ResolutionBudget::new(ProducerContract::ALL.len() - 1, 1, Duration::from_nanos(1))
            .expect("positive one-short entry budget");
    assert!(matches!(
        resolve_producer_contracts(&request, &manifest(), one_short, generous_clock()),
        OperationOutcome::ResourceIncomplete(incomplete)
            if incomplete.dimension() == BudgetDimension::ResolutionEntryCount
    ));
    assert!(matches!(
        resolve_producer_contracts(&request, &manifest(), exact_budget(), generous_clock()),
        OperationOutcome::Completed(_)
    ));
}

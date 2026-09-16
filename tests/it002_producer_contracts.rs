// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-002 SC-08: exact QObs and QSL producer-contract resolution.

use std::cell::Cell;
use std::time::{Duration, Instant};

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::BudgetDimension;
use quire_protocol::bounds::Clock;
use quire_protocol::outcome::OperationOutcome;
use quire_protocol::producer_contracts::{
    resolve_producer_contracts, ProducerContract, ProducerResolution, ProducerResolutionRequest,
    UnsupportedObservationCapability, QOBS_REVISION,
};
use quire_protocol::version_lock::{
    EntryStatus, ManifestEntry, ResolutionBudget, ResolutionRefusalCode, VersionLockManifest,
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

fn refusal_code(outcome: OperationOutcome<ProducerResolution>) -> String {
    let OperationOutcome::Refused(refusal) = outcome else {
        panic!("mutated producer-contract resolution must refuse")
    };
    refusal.code().as_str().to_owned()
}

#[test]
#[trace(
    "IT-002",
    "FR-009-AC-1",
    "FR-009-AC-2",
    "FR-009-AC-3",
    "FR-009-AC-4",
    "FR-009-AC-5",
    "FR-009-AC-6"
)]
fn it_002_sc_08_resolves_only_the_exact_scenario_contract_inventory() {
    assert_eq!(ProducerContract::ALL.len(), 19);
    assert_eq!(QOBS_REVISION, "581d98f1ac9f1467cc5355785d67b242f8d5d150");
    let request = ProducerResolutionRequest::new(b"it002:scenario-resolution")
        .expect("nonempty scenario resolution identity");
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
                "6d8b3b41a7fdc9b7d4dba085809b396bc41e41c89f9c512ff1580108d43663e8",
            ),
            (
                "quire.observation-authority/v1",
                "6b6f4e3b08a4b16476b55ae6cdfdff4440b8c053f056ffd7b9bdcde47e200296",
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
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &absent,
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::EntryAbsent.as_str()
    );

    let mut unpinned_entries = manifest().entries().to_vec();
    unpinned_entries[0] = ManifestEntry::pinned_interface(
        ProducerContract::ALL[0].key(),
        None,
        Some(b"owner asserts immutable and published"),
        None,
    );
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(unpinned_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::RevisionAbsent.as_str()
    );

    let mut duplicate_entries = manifest().entries().to_vec();
    duplicate_entries.push(ProducerContract::ObservationRecord.manifest_entry());
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(duplicate_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::DuplicateEntryKey.as_str()
    );

    let mut unresolved_entries = manifest().entries().to_vec();
    unresolved_entries[0] = ManifestEntry::unresolved_interface(
        ProducerContract::ALL[0].key(),
        b"owner acceptance withdrawn",
    );
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(unresolved_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::EntryUnresolved.as_str()
    );

    let mut repointed_entries = manifest().entries().to_vec();
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
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(repointed_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::InterfaceSelectionMismatch.as_str()
    );

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

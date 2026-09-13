// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-013's B-owned scenario-evidence record and refusal-axis invariant.

use std::cell::Cell;
use std::time::{Duration, Instant};

use ix_trace_rs::trace;
use quire_protocol::bounds::Clock;
use quire_protocol::version_lock::evidence::{
    validate_refusal_axis_distinctness, BudgetDimension, BudgetFigure, EvidenceOutcome,
    IntegrationId, ResolutionEvidence, ScenarioEvidenceRecord, ScenarioKey, TraceBinding,
    ValueOwner,
};
use quire_protocol::version_lock::{
    resolve, ArtifactDelivery, EntryKey, ExactBytes, ManifestEntry, ResolutionBudget,
    ResolutionOutcome, ResolutionRefusalCode, VersionLockManifest,
};
use quire_protocol::ByteDigest;

#[derive(Debug)]
struct FixedClock {
    now: Instant,
    calls: Cell<usize>,
}

impl FixedClock {
    fn new() -> Self {
        Self {
            now: Instant::now(),
            calls: Cell::new(0),
        }
    }
}

impl Clock for FixedClock {
    fn monotonic(&self) -> Instant {
        self.calls.set(self.calls.get() + 1);
        self.now
    }
}

fn key(owner: &str, version: &str) -> EntryKey {
    EntryKey::new(owner.as_bytes(), version.as_bytes()).expect("valid test key")
}

fn resolved_manifest() -> quire_protocol::version_lock::ResolvedManifest {
    let bytes = b"exact producer bytes";
    let entry = ManifestEntry::pinned_artifact(
        key("F", "observation/1"),
        Some(b"revision-f"),
        Some(b"F asserts immutable and published"),
        b"observation.bin",
        ByteDigest::of(bytes),
    );
    let outcome = resolve(
        &VersionLockManifest::new(vec![entry.clone()]),
        &[entry.key().clone()],
        &[ArtifactDelivery::new(b"observation.bin", bytes)],
        ResolutionBudget::new(1, bytes.len(), Duration::from_secs(1)),
        &FixedClock::new(),
    );
    let ResolutionOutcome::Resolved(resolved) = outcome else {
        panic!("fixture should resolve");
    };
    resolved
}

fn budget() -> Vec<BudgetFigure> {
    vec![
        BudgetFigure::new(BudgetDimension::ResolutionEntryCount, 1, 1),
        BudgetFigure::new(BudgetDimension::ResolutionBytesDigested, 20, 20),
        BudgetFigure::new(BudgetDimension::ResolutionDeadline, 1_000_000, 0),
    ]
}

fn refusal_record(
    scenario: &str,
    axis: &str,
    code: ResolutionRefusalCode,
) -> ScenarioEvidenceRecord {
    let missing = key("A", "2");
    let ResolutionOutcome::Refused(refusal) = resolve(
        &VersionLockManifest::new(vec![]),
        std::slice::from_ref(&missing),
        &[],
        ResolutionBudget::new(1, 0, Duration::from_secs(1)),
        &FixedClock::new(),
    ) else {
        panic!("missing entry fixture should refuse");
    };
    assert_eq!(refusal.code(), ResolutionRefusalCode::EntryAbsent);

    ScenarioEvidenceRecord::new(
        ScenarioKey::new(IntegrationId::It002, scenario.as_bytes()).expect("scenario key"),
        ResolutionEvidence::from_refusal(&refusal).expect("entry refusal has a partial state"),
        vec![],
        EvidenceOutcome::Refusal(ExactBytes::new(code.as_str().as_bytes())),
        Some(ExactBytes::new(axis.as_bytes())),
        budget(),
        TraceBinding::new(b"FR-009-AC-2").expect("trace binding"),
    )
    .expect("complete refusal evidence")
}

#[test]
#[trace("TC-013", "FR-017-AC-5")]
fn tc_013_scenario_record_carries_resolution_owner_outcome_budget_and_trace() {
    let resolved = resolved_manifest();
    let record = ScenarioEvidenceRecord::new(
        ScenarioKey::new(IntegrationId::It003, b"IT-003-SC-08b").expect("scenario key"),
        ResolutionEvidence::complete(&resolved),
        vec![ValueOwner::new(b"observation-value-7", b"F").expect("value owner")],
        EvidenceOutcome::CanonicalResult(ExactBytes::new(b"sha256-jcs:result")),
        None,
        budget(),
        TraceBinding::new(b"NFR-005-AC-1").expect("trace binding"),
    )
    .expect("every required component is present");

    assert_eq!(record.key().integration(), IntegrationId::It003);
    assert_eq!(record.key().scenario().as_bytes(), b"IT-003-SC-08b");
    assert!(matches!(
        record.resolution(),
        ResolutionEvidence::Complete(_)
    ));
    assert_eq!(
        record.value_owners()[0].value().as_bytes(),
        b"observation-value-7"
    );
    assert_eq!(record.value_owners()[0].owner().as_bytes(), b"F");
    assert!(matches!(
        record.outcome(),
        EvidenceOutcome::CanonicalResult(_)
    ));
    assert_eq!(record.budget().len(), 3);
    assert_eq!(record.trace().criterion().as_bytes(), b"NFR-005-AC-1");
}

#[test]
#[trace("TC-013")]
fn tc_013_distinct_refusal_axes_must_retain_distinct_outcomes() {
    let first = refusal_record(
        "IT-002-SC-07a",
        "absent-entry",
        ResolutionRefusalCode::EntryAbsent,
    );
    let same_axis = refusal_record(
        "IT-002-SC-07b",
        "absent-entry",
        ResolutionRefusalCode::EntryAbsent,
    );
    validate_refusal_axis_distinctness(&[first.clone(), same_axis])
        .expect("two scenarios may exercise one refusal axis");

    let colliding_axis = refusal_record(
        "IT-002-SC-08",
        "digest-mismatch",
        ResolutionRefusalCode::EntryAbsent,
    );
    let collision = validate_refusal_axis_distinctness(&[first.clone(), colliding_axis])
        .expect_err("different axes cannot share one typed outcome");
    assert_eq!(collision.integration(), IntegrationId::It002);
    assert_eq!(collision.first_axis().as_bytes(), b"absent-entry");
    assert_eq!(collision.second_axis().as_bytes(), b"digest-mismatch");

    let distinct = refusal_record(
        "IT-002-SC-08",
        "digest-mismatch",
        ResolutionRefusalCode::DigestMismatch,
    );
    validate_refusal_axis_distinctness(&[first, distinct])
        .expect("different axes with different typed outcomes are valid");
}

#[test]
#[trace("TC-013", "FR-017-AC-2")]
fn tc_013_entry_refusal_retains_partial_resolution_and_causing_entry() {
    let first = ManifestEntry::pinned_interface(
        key("Quoin", "evidence/1"),
        Some(b"revision-q"),
        Some(b"Quoin asserts immutable and published"),
        None,
    );
    let bad_bytes = b"different producer bytes";
    let second = ManifestEntry::pinned_artifact(
        key("F", "observation/1"),
        Some(b"revision-f"),
        Some(b"F asserts immutable and published"),
        b"observation.bin",
        ByteDigest::of(b"selected producer bytes"),
    );
    let ResolutionOutcome::Refused(refusal) = resolve(
        &VersionLockManifest::new(vec![first.clone(), second.clone()]),
        &[first.key().clone(), second.key().clone()],
        &[ArtifactDelivery::new(b"observation.bin", bad_bytes)],
        ResolutionBudget::new(2, bad_bytes.len(), Duration::from_secs(1)),
        &FixedClock::new(),
    ) else {
        panic!("the second entry's digest should refuse");
    };
    let record = ScenarioEvidenceRecord::new(
        ScenarioKey::new(IntegrationId::It002, b"IT-002-SC-07a").expect("scenario key"),
        ResolutionEvidence::from_refusal(&refusal).expect("entry-caused refusal"),
        vec![],
        EvidenceOutcome::Refusal(ExactBytes::new(refusal.code().as_str().as_bytes())),
        Some(ExactBytes::new(b"digest-mismatch")),
        budget(),
        TraceBinding::new(b"FR-017-AC-2").expect("trace binding"),
    )
    .expect("complete partial evidence");
    let ResolutionEvidence::Partial {
        resolved_before,
        causing_entry,
    } = record.resolution()
    else {
        panic!("an entry refusal must carry partial resolution evidence");
    };
    assert_eq!(resolved_before, std::slice::from_ref(&first));
    assert_eq!(causing_entry, second.key());
}

#[test]
#[trace("TC-013")]
fn tc_013_scenario_record_identity_is_only_integration_and_scenario() {
    let first = refusal_record(
        "IT-002-SC-07a",
        "absent-entry",
        ResolutionRefusalCode::EntryAbsent,
    );
    let changed_state = ScenarioEvidenceRecord::new(
        first.key().clone(),
        ResolutionEvidence::complete(&resolved_manifest()),
        vec![ValueOwner::new(b"different-value", b"different-owner").expect("value owner")],
        EvidenceOutcome::CanonicalResult(ExactBytes::new(b"different-result")),
        None,
        vec![BudgetFigure::new(BudgetDimension::Operation, 99, 12)],
        TraceBinding::new(b"FR-017-AC-6").expect("trace binding"),
    )
    .expect("complete alternate state");
    assert_eq!(first, changed_state);

    let other_scenario = refusal_record(
        "IT-002-SC-07b",
        "absent-entry",
        ResolutionRefusalCode::EntryAbsent,
    );
    assert_ne!(first, other_scenario);
}

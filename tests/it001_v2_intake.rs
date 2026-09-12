// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-06..SC-09/Task-009: the version 2 consumer lane.
//!
//! The version 1 lane in `it001_native_handoff.rs` is unchanged and remains the
//! compatibility precedent. This file owns only version 2.
//!
//! A's `protocol_artifact::v2` boundary is not published yet, so no test here
//! calls it. What is exercised today is B-owned: the evidence loader, its
//! absence reporting, and the scenario corpus the lane owes. The single
//! `#[ignore]` lane test fails loudly rather than passing vacuously.

use std::{
    fs,
    path::{Path, PathBuf},
};

use ix_trace_rs::trace;

#[path = "support/v2/mod.rs"]
mod v2;

use v2::{
    Absence, CrossVersion, DigestDomain, Disposition, ExpectedRefusal, Fixture, IndexedLimit,
    LimitBound, OfferedAxis, ProducerInput, Scenario, ScenarioKind, SelectionAxis,
    TemporalBindingMutation, REQUIRED_A_DELIVERABLES,
};

// ---------------------------------------------------------------------------
// Corpus obligations
// ---------------------------------------------------------------------------

#[test]
#[trace("Task-009")]
fn static_corpus_ids_are_unique() {
    let scenarios = v2::static_scenarios();
    let mut ids: Vec<&str> = scenarios.iter().map(|row| row.id.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "scenario ids collide: {ids:?}");
}

#[test]
#[trace("Task-009")]
fn static_corpus_covers_every_changed_selection_axis() {
    let scenarios = v2::static_scenarios();
    for axis in SelectionAxis::ALL {
        assert!(
            scenarios
                .iter()
                .any(|row| row.kind == ScenarioKind::ChangedSelection(axis)),
            "no changed-selection scenario for {}",
            axis.slug()
        );
    }
    assert_eq!(
        count(&scenarios, |kind| matches!(
            kind,
            ScenarioKind::ChangedSelection(_)
        )),
        SelectionAxis::ALL.len()
    );
}

#[test]
#[trace("Task-009")]
fn static_corpus_covers_both_cross_version_submissions() {
    let scenarios = v2::static_scenarios();
    for cross in CrossVersion::ALL {
        assert!(
            scenarios
                .iter()
                .any(|row| row.kind == ScenarioKind::CrossVersion(cross)),
            "no cross-version scenario for {}",
            cross.slug()
        );
    }
}

#[test]
#[trace("Task-009")]
fn static_corpus_substitutes_every_ordered_digest_domain_pair() {
    let scenarios = v2::static_scenarios();
    let mut pairs = 0;
    for offered in DigestDomain::ALL {
        for slot in DigestDomain::ALL {
            let present = scenarios
                .iter()
                .any(|row| row.kind == ScenarioKind::DigestDomainSubstitution { offered, slot });
            if offered == slot {
                assert!(
                    !present,
                    "{} substituted for itself is not a substitution",
                    offered.as_str()
                );
            } else {
                assert!(
                    present,
                    "no substitution of {} into the {} slot",
                    offered.as_str(),
                    slot.as_str()
                );
                pairs += 1;
            }
        }
    }
    // Two selected members (FR-015 selects from FR-201's nine), so two
    // ordered pairs. An earlier revision asserted twelve, over a local
    // four-member vocabulary FR-015 withdrew.
    assert_eq!(pairs, 2);
}

#[test]
#[trace("Task-009")]
fn static_corpus_recanonicalizes_every_producer_input() {
    let scenarios = v2::static_scenarios();
    for input in ProducerInput::ALL {
        assert!(
            scenarios
                .iter()
                .any(|row| row.kind == ScenarioKind::Recanonicalized(input)),
            "no recanonicalized scenario for {}",
            input.slug()
        );
    }
}

#[test]
#[trace("Task-009")]
fn static_corpus_covers_every_temporal_binding_mutation() {
    let scenarios = v2::static_scenarios();
    for mutation in TemporalBindingMutation::ALL {
        assert!(
            scenarios
                .iter()
                .any(|row| row.kind == ScenarioKind::TemporalBinding(mutation)),
            "no temporal-binding scenario for {}",
            mutation.slug()
        );
    }
}

/// IT-001 Inputs declares six offered-artifact mutations for *both* lanes. They
/// change what A offered, not what B selected, and need their own version 2 rows.
#[test]
#[trace("Task-009")]
fn static_corpus_covers_every_offered_artifact_axis() {
    let scenarios = v2::static_scenarios();
    for axis in OfferedAxis::ALL {
        assert!(
            scenarios
                .iter()
                .any(|row| row.kind == ScenarioKind::ChangedOffer(axis)),
            "no offered-artifact scenario for {}",
            axis.slug()
        );
    }
    assert_eq!(
        count(&scenarios, |kind| matches!(
            kind,
            ScenarioKind::ChangedOffer(_)
        )),
        OfferedAxis::ALL.len()
    );
}

/// A two-valued admits/refuses oracle is satisfied by one shared error code
/// across every negative row. Each refusing scenario must expect a distinct
/// refusal identity, and each admitting scenario must expect none.
#[test]
#[trace("Task-009")]
fn every_refusing_scenario_expects_a_distinct_refusal_identity() {
    let limits = vec![
        IndexedLimit {
            name: "declaration-count".to_owned(),
            exact: 10_000,
        },
        IndexedLimit {
            name: "artifact-bytes".to_owned(),
            exact: 1_048_576,
        },
    ];
    let mut scenarios = v2::static_scenarios();
    scenarios.extend(v2::resource_scenarios(&limits));

    let mut identities: Vec<String> = Vec::new();
    for row in &scenarios {
        match (row.disposition, &row.expected_refusal) {
            (Disposition::Admits, None) => {}
            (Disposition::RefusesBeforeLinking, Some(expected)) => {
                identities.push(format!("{expected:?}"));
            }
            (disposition, expected) => panic!("{} pairs {disposition:?} with {expected:?}", row.id),
        }
    }

    let before = identities.len();
    identities.sort();
    identities.dedup();
    assert_eq!(
        before,
        identities.len(),
        "two refusing scenarios share one expected refusal identity"
    );
    assert_eq!(
        before,
        scenarios
            .iter()
            .filter(|row| row.disposition == Disposition::RefusesBeforeLinking)
            .count()
    );
}

/// The budget probe, stated the way SC-09 states it: the input is held
/// constant and only the budget moves.
#[test]
#[trace("Task-009")]
fn the_one_short_budget_is_the_only_refusing_limit_probe() {
    let limits = vec![IndexedLimit {
        name: "declaration-count".to_owned(),
        exact: 10_000,
    }];
    for row in v2::resource_scenarios(&limits) {
        let ScenarioKind::ResourceLimit { bound, .. } = row.kind else {
            panic!("{} is not a resource probe", row.id);
        };
        match bound {
            LimitBound::OneShort => {
                assert_eq!(row.disposition, Disposition::RefusesBeforeLinking);
                assert_eq!(
                    row.expected_refusal,
                    Some(ExpectedRefusal::ExhaustedBudget {
                        limit: "declaration-count".to_owned()
                    })
                );
            }
            LimitBound::Exact | LimitBound::FreshRetry => {
                assert_eq!(row.disposition, Disposition::Admits);
                assert_eq!(row.expected_refusal, None);
            }
        }
    }
}

#[test]
#[trace("Task-009")]
fn only_the_valid_offer_admits_in_the_static_corpus() {
    for row in v2::static_scenarios() {
        let expected = match row.kind {
            ScenarioKind::Valid => Disposition::Admits,
            _ => Disposition::RefusesBeforeLinking,
        };
        assert_eq!(row.disposition, expected, "{} disposition", row.id);
    }
}

#[test]
#[trace("Task-009")]
fn every_declared_limit_gets_an_exact_a_one_short_and_a_fresh_retry_probe() {
    let limits = vec![
        IndexedLimit {
            name: "declaration-count".to_owned(),
            exact: 10_000,
        },
        IndexedLimit {
            name: "artifact-bytes".to_owned(),
            exact: 1_048_576,
        },
    ];
    let scenarios = v2::resource_scenarios(&limits);
    assert_eq!(scenarios.len(), limits.len() * LimitBound::ALL.len());
    for limit in &limits {
        for bound in LimitBound::ALL {
            let row = find(&scenarios, &limit.name, bound);
            let expected = match bound {
                LimitBound::OneShort => Disposition::RefusesBeforeLinking,
                LimitBound::Exact | LimitBound::FreshRetry => Disposition::Admits,
            };
            assert_eq!(row.disposition, expected, "{} disposition", row.id);
            assert_eq!(row.criterion, "IT-001-SC-09");
        }
    }
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

#[test]
#[trace("Task-009")]
fn a_missing_fixture_root_is_reported_not_skipped() {
    let absent = scratch("absent-root").join("never-created");
    match Fixture::load_from(&absent) {
        Err(Absence::MissingRoot(path)) => assert_eq!(path, absent),
        other => panic!("expected a MissingRoot report, got {other:?}"),
    }
}

#[test]
#[trace("Task-009")]
fn a_delivery_without_b_s_evidence_index_is_reported() {
    let root = scratch("no-delivery_index");
    fs::create_dir_all(&root).expect("scratch root");
    match Fixture::load_from(&root) {
        Err(Absence::MissingDeliveryIndex(path)) => {
            assert_eq!(path, root.join(v2::DELIVERY_INDEX_FILE));
        }
        other => panic!("expected a MissingDeliveryIndex report, got {other:?}"),
    }
}

#[test]
#[trace("Task-009")]
fn an_indexed_member_that_is_absent_is_reported() {
    let root = scratch("missing-member");
    write_delivery(&root);
    fs::remove_file(root.join("inputs/source.native")).expect("remove indexed source");
    match Fixture::load_from(&root) {
        Err(Absence::MissingMember { role, path }) => {
            assert_eq!(role, "Source");
            assert_eq!(path, root.join("inputs/source.native"));
        }
        other => panic!("expected a MissingMember report, got {other:?}"),
    }
}

/// The loader must carry the offered bytes through unchanged and copy A's
/// artifact reference verbatim.
///
/// The bytes written here are opaque loader input, not a version 2 artifact, and no
/// test submits them to an admission API. Hand-authored emitted bytes are never
/// admissible evidence for SC-06.
#[test]
#[trace("Task-009")]
fn the_loader_preserves_the_offered_bytes_and_reference() {
    let root = scratch("preserves-bytes");
    write_delivery(&root);
    let fixture = Fixture::load_from(&root).expect("assembled delivery");
    let on_disk = fs::read(root.join("compiled-protocol.v2.bin")).expect("offer bytes");
    assert_eq!(fixture.offer().bytes(), on_disk.as_slice());
    assert_eq!(fixture.offer().reference(), "ix://example/offer@v2");
    assert_eq!(fixture.delivery_index().producer_revision, "0".repeat(40));
    assert_eq!(
        fixture
            .input_bytes(v2::SelectionRole::Source)
            .expect("indexed source is readable")
            .as_deref(),
        Some(b"original-source-bytes".as_slice())
    );
}

/// Every part of B's evidence index is read, and every indexed mutation names a
/// scenario the corpus actually owes. An unrecognised mutation is a delivery
/// defect, not a row to skip.
#[test]
#[trace("Task-009")]
fn the_loader_indexes_the_temporal_and_mutation_evidence() {
    let root = scratch("indexes-evidence");
    write_delivery(&root);
    let fixture = Fixture::load_from(&root).expect("assembled delivery");
    let delivery_index = fixture.delivery_index();

    assert_eq!(delivery_index.temporal.clock_alternative, "wall");
    assert_eq!(delivery_index.temporal.clock_parameters, ["resolution"]);
    let binding = delivery_index
        .temporal
        .bindings
        .first()
        .expect("at least one temporal binding");
    assert_eq!(binding.owner, "Workflow");
    assert_eq!(binding.declaration, 0);

    let limit = delivery_index
        .limits
        .first()
        .expect("at least one declared limit");
    assert_eq!(limit.exact, 10_000);

    let mut mutation_ids: Vec<&str> = delivery_index
        .mutations
        .iter()
        .map(|mutation| mutation.id.as_str())
        .collect();
    mutation_ids.sort_unstable();
    let before = mutation_ids.len();
    mutation_ids.dedup();
    assert_eq!(before, mutation_ids.len(), "mutation ids collide");

    let owed: Vec<String> = v2::scenarios_for(delivery_index)
        .into_iter()
        .map(|row| row.id)
        .collect();
    for mutation in &delivery_index.mutations {
        assert!(
            owed.contains(&mutation.scenario),
            "mutation {} names scenario {}, which this lane does not owe",
            mutation.id,
            mutation.scenario
        );
        assert!(
            fixture
                .mutation_bytes(&mutation.scenario)
                .expect("indexed mutation is readable and unambiguous")
                .is_some(),
            "mutation {} has no readable bytes",
            mutation.id
        );
    }
}

#[test]
#[trace("Task-009")]
fn a_delivery_owes_a_scenario_row_for_every_declared_limit() {
    let root = scratch("scenarios-for");
    write_delivery(&root);
    let fixture = Fixture::load_from(&root).expect("assembled delivery");
    let scenarios = v2::scenarios_for(fixture.delivery_index());
    assert_eq!(
        scenarios.len(),
        v2::static_scenarios().len()
            + fixture.delivery_index().limits.len() * LimitBound::ALL.len()
    );
}

// ---------------------------------------------------------------------------
// The lane itself
// ---------------------------------------------------------------------------

/// SC-06 through SC-09 cannot run until A publishes its final version 2 revision.
///
/// A's `protocol_artifact::v2` is implemented and under final integration, and
/// B's spec and corpus are built against its exact published names. Execution
/// stays dependency-gated: B pins no intermediate commit, so until the final
/// revision lands there is nothing to submit.
///
/// This test is deliberately NOT `#[ignore]`d. The declared gate in AGENTS.md
/// runs `cargo test --locked`, which never executes ignored tests, so an
/// ignored lane test could not report the block to the gate at all. Here the
/// blocked state is what the gate sees: with no delivery configured the lane
/// reports exactly why it cannot run, and the moment a delivery appears the
/// test fails until `admit_and_link_v2` is written against a pinned revision.
///
/// It carries no `IT-001` trace tag. It exercises no IT-001 scenario, and
/// tagging it would count it among IT-001's backing symbols while SC-06
/// through SC-09 are unexecuted.
#[test]
#[trace("Task-009")]
fn the_v2_lane_reports_its_blocked_state_to_the_gate() {
    let owed = REQUIRED_A_DELIVERABLES.join("\n  - ");
    match Fixture::load() {
        Err(Absence::EnvUnset) => {
            // The expected state today: no delivery, lane unexecuted, nothing
            // claimed about A. Recorded, not skipped.
        }
        Err(absence) => panic!(
            "a /2 delivery was configured but could not be assembled: {absence}.\n\
             quire-spec-language#40 owes:\n  - {owed}"
        ),
        Ok(fixture) => {
            let undelivered = fixture.undelivered_scenarios();
            panic!(
                "a /2 delivery is present at {} (producer revision {}), but \
                 quire_protocol::admit_and_link_v2 is not implemented. If that \
                 is A's final revision, pin it in Cargo.toml over the \
                 provisional /1 pin, rederive B's selection, and run the {} \
                 scenarios this delivery owes. Refusal scenarios with no \
                 delivered mutation fixture: {}.",
                fixture.root().display(),
                fixture.delivery_index().producer_revision,
                v2::scenarios_for(fixture.delivery_index()).len(),
                if undelivered.is_empty() {
                    "none".to_owned()
                } else {
                    undelivered.join(", ")
                }
            )
        }
    }
}

/// An under-delivery must not read as an assembled lane.
#[test]
#[trace("Task-009")]
fn a_delivery_missing_mutation_fixtures_names_every_scenario_it_owes() {
    let root = scratch("under-delivery");
    write_delivery(&root);
    let fixture = Fixture::load_from(&root).expect("assembled delivery");
    let undelivered = fixture.undelivered_scenarios();
    let owed_refusals = v2::scenarios_for(fixture.delivery_index())
        .into_iter()
        .filter(|row| row.disposition == Disposition::RefusesBeforeLinking)
        .count();
    // The sample delivery registers exactly one mutation fixture.
    assert_eq!(undelivered.len(), owed_refusals - 1);
    assert!(!undelivered.contains(&"v2-changed-source".to_owned()));
    assert!(undelivered.contains(&"v2-changed-configuration".to_owned()));
}

/// A scenario with two registered mutation fixtures is a delivery defect, not a
/// first-match lookup.
#[test]
#[trace("Task-009")]
fn two_mutation_fixtures_for_one_scenario_are_refused() {
    let root = scratch("ambiguous-mutation");
    write_delivery(&root);
    let index_path = root.join(v2::DELIVERY_INDEX_FILE);
    let delivery_index = fs::read_to_string(&index_path).expect("delivery_index");
    fs::write(
        root.join("mutations/changed-source-2.bin"),
        b"changed-again",
    )
    .expect("second");
    fs::write(
        &index_path,
        delivery_index.replace(
            r#"{ "id": "m-1", "scenario": "v2-changed-source", "path": "mutations/changed-source.bin" }"#,
            r#"{ "id": "m-1", "scenario": "v2-changed-source", "path": "mutations/changed-source.bin" },
    { "id": "m-2", "scenario": "v2-changed-source", "path": "mutations/changed-source-2.bin" }"#,
        ),
    )
    .expect("rewrite delivery_index");
    let fixture = Fixture::load_from(&root).expect("assembled delivery");
    match fixture.mutation_bytes("v2-changed-source") {
        Err(Absence::AmbiguousMutation { scenario }) => {
            assert_eq!(scenario, "v2-changed-source");
        }
        other => panic!("expected an AmbiguousMutation report, got {other:?}"),
    }
}

/// A's declaration-index key must be unique. Without that, SC-08's "duplicate"
/// mutation is indistinguishable from a legitimate binding set.
#[test]
#[trace("Task-009")]
fn a_repeated_binding_declaration_is_refused() {
    let root = scratch("duplicate-binding-key");
    write_delivery(&root);
    rewrite_bindings(
        &root,
        r#"{ "owner": "Workflow", "declaration": 0, "path": "inputs/binding-0.json" },
      { "owner": "Workflow", "declaration": 0, "path": "inputs/binding-3.json" }"#,
    );
    match Fixture::load_from(&root) {
        Err(Absence::RepeatedBindingDeclaration { declaration }) => {
            assert_eq!(declaration, 0);
        }
        other => panic!("expected a RepeatedBindingDeclaration report, got {other:?}"),
    }
}

/// A's rule says bindings "sort by `declaration`". Without that holding, SC-08's
/// "reordered" mutation is judged against file position rather than A's rule.
#[test]
#[trace("Task-009")]
fn bindings_not_sorted_by_declaration_are_refused() {
    let root = scratch("unordered-bindings");
    write_delivery(&root);
    rewrite_bindings(
        &root,
        r#"{ "owner": "Workflow", "declaration": 3, "path": "inputs/binding-3.json" },
      { "owner": "Workflow", "declaration": 0, "path": "inputs/binding-0.json" }"#,
    );
    match Fixture::load_from(&root) {
        Err(Absence::UnsortedBindings { at, after }) => {
            assert_eq!((at, after), (0, 3));
        }
        other => panic!("expected an UnsortedBindings report, got {other:?}"),
    }
}

/// A sparse ascending declaration set is legitimate: A's rule says bindings sort
/// by `declaration`, not that the declarations are contiguous.
#[test]
#[trace("Task-009")]
fn a_sparse_ascending_declaration_set_is_accepted() {
    let root = scratch("sparse-bindings");
    write_delivery(&root);
    let fixture = Fixture::load_from(&root).expect("assembled delivery");
    let keys: Vec<u32> = fixture
        .delivery_index()
        .temporal
        .bindings
        .iter()
        .map(|binding| binding.declaration)
        .collect();
    assert_eq!(keys, [0, 3]);
}

/// An index B cannot deserialise is reported with its cause, never skipped.
#[test]
#[trace("Task-009")]
fn an_unreadable_evidence_index_is_reported_with_its_cause() {
    let root = scratch("unreadable-delivery_index");
    write_delivery(&root);
    fs::write(root.join(v2::DELIVERY_INDEX_FILE), b"{ not json").expect("corrupt delivery_index");
    match Fixture::load_from(&root) {
        Err(Absence::UnreadableDeliveryIndex { path, cause }) => {
            assert_eq!(path, root.join(v2::DELIVERY_INDEX_FILE));
            assert!(!cause.is_empty(), "the serde cause must be reported");
        }
        other => panic!("expected an UnreadableDeliveryIndex report, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn count(scenarios: &[Scenario], predicate: impl Fn(&ScenarioKind) -> bool) -> usize {
    scenarios.iter().filter(|row| predicate(&row.kind)).count()
}

fn find<'a>(scenarios: &'a [Scenario], limit: &str, bound: LimitBound) -> &'a Scenario {
    scenarios
        .iter()
        .find(|row| {
            row.kind
                == ScenarioKind::ResourceLimit {
                    limit: limit.to_owned(),
                    bound,
                }
        })
        .unwrap_or_else(|| panic!("no {} probe for {limit}", bound.slug()))
}

/// Replace the delivery_index's binding array with `bindings`, leaving the rest of
/// B's evidence index untouched.
fn rewrite_bindings(root: &Path, bindings: &str) {
    let index_path = root.join(v2::DELIVERY_INDEX_FILE);
    let delivery_index = fs::read_to_string(&index_path).expect("delivery_index");
    let start = delivery_index
        .find("\"bindings\": [")
        .expect("bindings key");
    let open = delivery_index[start..].find('[').expect("bindings open") + start;
    let close = delivery_index[open..].find(']').expect("bindings close") + open;
    let rewritten = format!(
        "{}[\n      {bindings}\n    {}",
        &delivery_index[..open],
        &delivery_index[close..]
    );
    fs::write(&index_path, rewritten).expect("rewrite bindings");
}

fn scratch(name: &str) -> PathBuf {
    // Unique per process and per call: a fixed shared path under `temp_dir()`
    // lets a concurrent run on this workstation delete another run's fixtures.
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after the epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "quire-protocol-v2-{name}-{}-{unique}",
        std::process::id()
    ))
}

/// Write a minimally shaped delivery directory for the loader's own tests.
fn write_delivery(root: &Path) {
    fs::create_dir_all(root.join("inputs")).expect("inputs dir");
    fs::create_dir_all(root.join("mutations")).expect("mutations dir");
    fs::write(root.join("compiled-protocol.v2.bin"), b"opaque-offer-bytes").expect("offer");
    fs::write(root.join("inputs/source.native"), b"original-source-bytes").expect("source");
    fs::write(root.join("inputs/model.json"), b"original-model-bytes").expect("model");
    fs::write(root.join("inputs/profile.json"), b"original-profile-bytes").expect("profile");
    fs::write(root.join("inputs/binding-0.json"), b"binding-zero").expect("binding");
    fs::write(root.join("inputs/binding-3.json"), b"binding-three").expect("binding");
    fs::write(root.join("mutations/changed-source.bin"), b"changed").expect("mutation");
    fs::write(
        root.join(v2::DELIVERY_INDEX_FILE),
        br#"{
  "producer_revision": "0000000000000000000000000000000000000000",
  "offer": {
    "path": "compiled-protocol.v2.bin",
    "reference": "ix://example/offer@v2"
  },
  "inputs": [
    { "role": "source", "path": "inputs/source.native" },
    { "role": "model", "path": "inputs/model.json" }
  ],
  "temporal": {
    "profile": "inputs/profile.json",
    "clock_alternative": "wall",
    "clock_parameters": ["resolution"],
    "bindings": [
      { "owner": "Workflow", "declaration": 0, "path": "inputs/binding-0.json" },
      { "owner": "Workflow", "declaration": 3, "path": "inputs/binding-3.json" }
    ]
  },
  "limits": [{ "name": "declaration-count", "exact": 10000 }],
  "mutations": [
    { "id": "m-1", "scenario": "v2-changed-source", "path": "mutations/changed-source.bin" }
  ]
}
"#,
    )
    .expect("delivery_index");
}

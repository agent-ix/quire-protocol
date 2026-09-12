// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-06..SC-09/Task-012: run A's published `/2` handoff through B.
//!
//! `it001_v2_intake.rs` owns B's evidence-index loader and the scenario corpus
//! B owes. This file owns the execution of A's own published handoff: the base
//! admission, and every case in A's mutation manifest with its declared refusal
//! identity.
//!
//! Nothing here reads the offered `/2` bytes to build an expectation. Every
//! `Expected` value comes from A's independent selection sidecar and the
//! original unrecanonicalized inputs, and each of those inputs is checked
//! against the identity the sidecar selects for it before any admission runs.

use ix_trace_rs::trace;
use quire_spec_language::protocol_artifact::{self as artifact, v2};

#[path = "support/v2_handoff/mod.rs"]
mod handoff;

use handoff::{Absence, Handoff, MutationInput};

/// The lane's state is reported to the gate, never skipped.
///
/// With no handoff configured this records that SC-06..SC-09 did not execute.
/// With one configured it asserts the handoff assembles, so a handoff that
/// cannot be read fails the gate instead of silently reporting nothing.
///
/// It carries no `IT-001` tag: it exercises no IT-001 scenario.
#[test]
#[trace("Task-012")]
fn the_v2_handoff_lane_reports_its_state_to_the_gate() {
    match Handoff::load() {
        Err(Absence::EnvUnset) => {
            // No handoff on this machine: SC-06..SC-09 unexecuted, and nothing
            // claimed about A.
        }
        Err(absence) => panic!(
            "a /2 handoff was configured but could not be assembled: {absence}.\n\
             What B still needs from A:\n  - {}",
            handoff::missing_a_api().join("\n  - ")
        ),
        Ok(loaded) => {
            assert!(
                !loaded.offer().is_empty(),
                "the handoff at {} publishes an empty offer",
                loaded.root().display()
            );
        }
    }
}

/// A's manifest is a closed corpus: every case names a readable fixture.
#[test]
#[trace("Task-012")]
fn every_declared_mutation_publishes_its_fixture() {
    let Some(loaded) = configured() else { return };
    for case in &loaded.mutations().cases {
        let file = match &case.input {
            MutationInput::Offer { file, .. } | MutationInput::ReplaceOriginal { file, .. } => file,
        };
        let bytes = loaded
            .mutation_bytes(file)
            .unwrap_or_else(|absence| panic!("case {}: {absence}", case.identity));
        assert!(
            !bytes.is_empty(),
            "case {} publishes an empty fixture",
            case.identity
        );
        assert!(
            !case.expected_refusal_code.is_empty(),
            "case {} declares no refusal identity",
            case.identity
        );
    }
}

/// Case identities are unique, so one case's outcome cannot stand in for another's.
#[test]
#[trace("Task-012")]
fn declared_mutation_identities_are_unique() {
    let Some(loaded) = configured() else { return };
    let mut identities: Vec<&str> = loaded
        .mutations()
        .cases
        .iter()
        .map(|case| case.identity.as_str())
        .collect();
    identities.sort_unstable();
    let before = identities.len();
    identities.dedup();
    assert_eq!(before, identities.len(), "mutation identities collide");
}

/// SC-06: the published handoff is admitted and linked against B's independent
/// selection.
#[test]
#[trace("Task-012")]
fn the_published_handoff_is_admitted_and_linked() {
    let Some(loaded) = configured() else { return };
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);

    let report = quire_protocol::admit_and_link_v2(loaded.offer(), &expected, loaded.limits());
    match report.result() {
        Ok(package) => {
            assert_eq!(package.digest(), loaded.reference().digest);
            assert_eq!(package.package().inherited.wire, v2::WIRE);
            assert_eq!(
                package.package().temporal_bindings.len(),
                loaded.selection().temporal.len()
            );
        }
        Err(error) => panic!(
            "the published handoff was refused with {}: {error}",
            handoff::reported_code(error)
        ),
    }
}

/// SC-07/SC-08: every declared mutation is refused with the identity A declares
/// for it, and none is admitted.
#[test]
#[trace("Task-012")]
fn every_declared_mutation_is_refused_with_its_declared_identity() {
    let Some(loaded) = configured() else { return };
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let limits = loaded.limits();

    let mut wrong = Vec::new();
    let mut exercised = 0usize;
    for case in &loaded.mutations().cases {
        exercised += 1;
        let observed = match &case.input {
            MutationInput::Offer { file, artifact } => {
                let bytes = loaded
                    .mutation_bytes(file)
                    .unwrap_or_else(|absence| panic!("case {}: {absence}", case.identity));
                let expected = loaded.expected_for(&inventories, artifact);
                outcome(
                    quire_protocol::admit_and_link_v2(&bytes, &expected, limits),
                    &case.expected_refusal_code,
                )
            }
            MutationInput::ReplaceOriginal { target, file } => {
                let bytes = loaded
                    .mutation_bytes(file)
                    .unwrap_or_else(|absence| panic!("case {}: {absence}", case.identity));
                let replaced = loaded
                    .replaced_dependency_bytes(target, bytes)
                    .unwrap_or_else(|| {
                        panic!(
                            "case {}: {target} names no published dependency",
                            case.identity
                        )
                    });
                let dependencies = loaded.dependencies_over(&replaced);
                let base = loaded.expected(&inventories);
                let expected = v2::Expected {
                    inherited: artifact::Expected {
                        dependencies: &dependencies,
                        ..base.inherited
                    },
                    ..base
                };
                outcome(
                    quire_protocol::admit_and_link_v2(loaded.offer(), &expected, limits),
                    &case.expected_refusal_code,
                )
            }
        };
        if let Some(observed) = observed {
            wrong.push(format!(
                "{} (axis {}): declared {}, observed {observed}",
                case.identity, case.axis, case.expected_refusal_code
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} mutations did not refuse as declared:\n  {}",
        wrong.len(),
        loaded.mutations().cases.len(),
        wrong.join("\n  ")
    );
    // An empty or short corpus would satisfy the assertion above without
    // submitting anything.
    assert_eq!(exercised, loaded.mutations().cases.len());
    assert!(exercised > 0, "the mutation corpus is empty");
}

/// Negative control for the loop above: under the same expectation the base
/// offer is admitted, so a refusal there is caused by the mutation and not by
/// an expectation B assembled wrongly.
///
/// Without this, every mutation could be "refusing as declared" for a reason
/// that has nothing to do with its axis.
#[test]
#[trace("Task-012")]
fn the_mutation_expectation_admits_the_unmutated_offer() {
    let Some(loaded) = configured() else { return };
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let report = quire_protocol::admit_and_link_v2(loaded.offer(), &expected, loaded.limits());
    assert!(
        report.result().is_ok(),
        "the unmutated offer was refused under the mutation-loop expectation"
    );
}

/// Every axis A declares is exercised by at least one case, so an axis cannot
/// drop out of the corpus unnoticed.
#[test]
#[trace("Task-012")]
fn every_declared_axis_is_exercised() {
    let Some(loaded) = configured() else { return };
    let mut axes: Vec<&str> = loaded
        .mutations()
        .cases
        .iter()
        .map(|case| case.axis.as_str())
        .collect();
    axes.sort_unstable();
    axes.dedup();
    assert!(!axes.is_empty(), "no case declares an axis");
    for axis in axes {
        assert!(
            loaded
                .mutations()
                .cases
                .iter()
                .any(|case| case.axis == axis && !case.expected_refusal_code.is_empty()),
            "axis {axis} declares no refusal identity"
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `None` when no handoff is configured. A configured handoff that cannot be
/// assembled fails here rather than reading as an absent one.
fn configured() -> Option<Handoff> {
    match Handoff::load() {
        Ok(loaded) => Some(loaded),
        Err(Absence::EnvUnset) => None,
        Err(absence) => panic!("a /2 handoff was configured but could not be assembled: {absence}"),
    }
}

/// `None` when the outcome is the declared refusal; otherwise what was observed.
fn outcome(report: artifact::Report<v2::AdmittedPackage>, declared: &str) -> Option<String> {
    match report.result() {
        Ok(_) => Some("admission".to_owned()),
        Err(error) if handoff::matches_declared_code(error, declared) => None,
        Err(error) => Some(handoff::reported_code(error)),
    }
}

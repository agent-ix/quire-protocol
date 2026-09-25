// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-06..SC-09/Task-012: run A's published `/2` handoff through B.
//!
//! This file owns B's integrity checks and the execution of A's published
//! handoff: base admission, cross-version refusal, every case in A's mutation
//! manifest, and every declared resource dimension.
//!
//! Nothing here reads the offered `/2` bytes to build an expectation. Every
//! `Expected` value comes from A's independent selection sidecar and the
//! original unrecanonicalized inputs, and each of those inputs is checked
//! against the identity the sidecar selects for it before any admission runs.

use ix_trace_rs::trace;
use qsl_foundation::ByteDigest;
use quire_spec_language::protocol_artifact::{self as artifact, v2};

#[path = "support/v2_handoff/mod.rs"]
mod handoff;

use handoff::{Absence, Handoff, MutationInput};

/// A's mutation corpus as its producer emits it at the pinned revision, sorted.
///
/// Frozen so the corpus cannot shrink unnoticed: every assertion over "every
/// declared case" is only as strong as the number of cases A declares.
const DECLARED_CASES: usize = 28;

/// Every selection axis those cases mutate, sorted and deduplicated.
const DECLARED_AXES: [&str; 21] = [
    "clock.epoch",
    "clock.kind",
    "clock.period",
    "clock.sequence_authority",
    "clock.timestamp_unit",
    "clock.unit",
    "declaration.clause",
    "declaration.execution",
    "declaration.name",
    "declaration.requirement",
    "definition.artifact.digest",
    "definition.artifact.wire_identity",
    "definition.identity",
    "definition.original_bytes",
    "definition.revision",
    "header.version",
    "temporal_binding.declaration",
    "temporal_binding.definition",
    "temporal_bindings.inventory",
    "temporal_bindings.order",
    "temporal_bindings.uniqueness",
];

/// The pinned producer handoff is reachable and non-empty on every gate run.
#[test]
#[trace("IT-001", "Task-012")]
fn the_pinned_v2_handoff_is_reachable() {
    let loaded = configured();
    assert!(
        !loaded.offer().is_empty(),
        "the handoff at {} publishes an empty offer",
        loaded.root().display()
    );
}

/// A's manifest is a closed corpus: every case names a readable fixture.
#[test]
#[trace("Task-012")]
fn every_declared_mutation_publishes_its_fixture() {
    let loaded = configured();
    assert_eq!(
        loaded.mutations().cases.len(),
        DECLARED_CASES,
        "A's corpus is no longer the one frozen at the pin"
    );
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
    let loaded = configured();
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
#[trace("IT-001", "TC-001", "FR-001-AC-1", "Task-001", "Task-012")]
fn the_published_handoff_is_admitted_and_linked() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);

    let report = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits());
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

/// TC-001: the accepted semantic baseline is selected independently of the
/// offer. Identity, revision and digest are separate axes, and changing any one
/// refuses before an admitted package is exposed.
#[test]
#[trace("TC-001", "FR-001-AC-3", "Task-001")]
fn unaccepted_baseline_selections_refuse_without_offer_authority() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let base = loaded.expected(&inventories);
    base_admits(&loaded, &base);

    let mut wrong_identity = loaded.selection().inherited.baseline.clone();
    wrong_identity.identity.push_str("-unknown");
    let mut wrong_revision = loaded.selection().inherited.baseline.clone();
    wrong_revision.revision.value.push_str("-superseded");
    let mut wrong_digest = loaded.selection().inherited.baseline.clone();
    wrong_digest.digest = ByteDigest::of(b"unaccepted draft baseline");

    for (axis, baseline) in [
        ("identity", wrong_identity),
        ("revision", wrong_revision),
        ("digest", wrong_digest),
    ] {
        let expected = v2::Expected {
            inherited: artifact::Expected {
                baseline: &baseline,
                ..base.inherited
            },
            ..base
        };
        let error = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
            .into_result()
            .unwrap_err();
        assert_eq!(
            error.code(),
            "invalid.selection",
            "baseline {axis} returned the wrong typed refusal: {error}"
        );
    }

    base_admits(&loaded, &base);
}

/// TC-001: the current owner-published package carries and round-trips every
/// selected numeric edge, while one-axis invalid representations and values
/// refuse through A's typed reader rather than being repaired.
#[test]
#[trace("TC-001", "FR-001-AC-4", "Task-001")]
fn numeric_edges_round_trip_and_invalid_forms_refuse_without_repair() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let admitted = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("the published numeric control must admit");
    let encoded = v2::encode_candidate(admitted.package(), loaded.limits())
        .into_result()
        .expect("the admitted package must encode under the same limits");
    assert_eq!(encoded.bytes(), loaded.offer());

    for required in [
        br#""decimal":"0""#.as_slice(),
        br#""decimal":"1""#.as_slice(),
        br#""decimal":"5""#.as_slice(),
        br#""decimal":"-9223372036854775808""#.as_slice(),
        br#""decimal":"9223372036854775807""#.as_slice(),
        br#""numerator":"1","denominator":"1""#.as_slice(),
        br#""numerator":"1","denominator":"2""#.as_slice(),
        br#""numerator":"9223372036854775807","denominator":"9223372036854775806""#.as_slice(),
        br#""numerator":"-9223372036854775808","denominator":"9223372036854775807""#.as_slice(),
    ] {
        assert!(
            contains(loaded.offer(), required),
            "the published control dropped numeric edge {}",
            String::from_utf8_lossy(required)
        );
    }

    let integer_anchor = br#""original_expression":12,"locus":{"source":1"#;
    let minimum_anchor = br#""original_expression":33,"locus":{"source":1"#;
    let rational_anchor = br#""original_expression":1,"locus":{"source":0"#;
    let wide_anchor = br#""original_expression":8,"locus":{"source":0"#;
    let cases = [
        (
            "leading zero",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#""decimal":"5""#,
                br#""decimal":"05""#,
            ),
            "numeric.non-canonical-decimal",
        ),
        (
            "plus prefix",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#""decimal":"5""#,
                br#""decimal":"+5""#,
            ),
            "numeric.non-canonical-decimal",
        ),
        (
            "negative zero",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#""decimal":"5""#,
                br#""decimal":"-0""#,
            ),
            "numeric.non-canonical-decimal",
        ),
        (
            "decimal float",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#""decimal":"5""#,
                br#""decimal":"5.0""#,
            ),
            "numeric.non-canonical-decimal",
        ),
        (
            "unsafe bare integer",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#"{"kind":"integer","decimal":"5"}"#,
                b"9007199254740992",
            ),
            "invalid.structural-integer",
        ),
        (
            "signed-64 overflow",
            mutate_after(
                loaded.offer(),
                wide_anchor,
                br#""decimal":"9223372036854775807""#,
                br#""decimal":"9223372036854775808""#,
            ),
            "numeric.component-out-of-range",
        ),
        (
            "non-positive denominator",
            mutate_after(
                loaded.offer(),
                rational_anchor,
                br#""denominator":"1""#,
                br#""denominator":"0""#,
            ),
            "numeric.non-positive-denominator",
        ),
        (
            "negative denominator",
            mutate_after(
                loaded.offer(),
                rational_anchor,
                br#""denominator":"1""#,
                br#""denominator":"-1""#,
            ),
            "numeric.non-positive-denominator",
        ),
        (
            "unreduced rational",
            mutate_after(
                loaded.offer(),
                rational_anchor,
                br#""numerator":"1","denominator":"1""#,
                br#""numerator":"2","denominator":"2""#,
            ),
            "numeric.unreduced-rational",
        ),
        (
            "rational outside the integer field domain",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#"{"kind":"integer","decimal":"5"}"#,
                br#"{"kind":"rational","numerator":"1","denominator":"1"}"#,
            ),
            "invalid.numeric-domain",
        ),
        (
            "below field minimum",
            mutate_after(
                loaded.offer(),
                minimum_anchor,
                br#""decimal":"1""#,
                br#""decimal":"0""#,
            ),
            "invalid.numeric-domain",
        ),
        (
            "above field maximum",
            mutate_after(
                loaded.offer(),
                integer_anchor,
                br#""decimal":"5""#,
                br#""decimal":"6""#,
            ),
            "invalid.numeric-domain",
        ),
    ];

    for (case, bytes, expected_code) in cases {
        let mut seal = loaded.reference().clone();
        seal.digest = ByteDigest::of(&bytes);
        let expected = loaded.expected_for(&inventories, &seal);
        let error = quire_protocol::intake::v2::admit(&bytes, &expected, loaded.limits())
            .into_result()
            .unwrap_err();
        assert_eq!(
            error.code(),
            expected_code,
            "{case} returned the wrong typed refusal: {error}"
        );
    }
}

/// SC-06: each contract version refuses the other version's bytes through its
/// explicit public entry point; neither entry point detects or translates it.
#[test]
#[trace("IT-001", "FR-001-AC-5", "NFR-007-AC-3", "NFR-007-AC-4")]
fn contract_versions_mutually_refuse_without_translation() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected_v2 = loaded.expected(&inventories);

    let (candidate_v1, reference_v1) = inherited_v1_control(&loaded, &expected_v2);
    let expected_v1 = artifact::Expected {
        artifact: &reference_v1,
        ..expected_v2.inherited
    };

    quire_protocol::admit_and_link(candidate_v1.bytes(), &expected_v1, loaded.limits())
        .into_result()
        .expect("the derived /1 control must admit at the /1 entry point");

    let mut cross_reference_v1 = reference_v1.clone();
    cross_reference_v1.digest = loaded.reference().digest;
    let cross_expected_v1 = artifact::Expected {
        artifact: &cross_reference_v1,
        ..expected_v1
    };
    let v2_at_v1 =
        quire_protocol::admit_and_link(loaded.offer(), &cross_expected_v1, loaded.limits())
            .into_result()
            .expect_err("/2 bytes must refuse at the /1 entry point");
    assert_eq!(v2_at_v1.code(), "json");

    let mut cross_reference_v2 = loaded.reference().clone();
    cross_reference_v2.digest = candidate_v1.digest();
    let cross_expected_v2 = v2::Expected {
        inherited: artifact::Expected {
            artifact: &cross_reference_v2,
            ..expected_v2.inherited
        },
        ..expected_v2
    };
    let v1_at_v2 = quire_protocol::intake::v2::admit(
        candidate_v1.bytes(),
        &cross_expected_v2,
        loaded.limits(),
    )
    .into_result()
    .expect_err("/1 bytes must refuse at the /2 entry point");
    assert_eq!(v1_at_v2.code(), "json");
    assert_ne!(
        v2_at_v1, v1_at_v2,
        "the two cross-version directions must retain distinct typed causes"
    );

    for case in &loaded.mutations().cases {
        let error = match &case.input {
            MutationInput::Offer { file, artifact } => {
                let bytes = loaded
                    .mutation_bytes(file)
                    .unwrap_or_else(|absence| panic!("case {}: {absence}", case.identity));
                let expected = loaded.expected_for(&inventories, artifact);
                quire_protocol::intake::v2::admit(&bytes, &expected, loaded.limits())
                    .into_result()
                    .unwrap_err()
            }
            MutationInput::ReplaceOriginal { target, file } => {
                let bytes = loaded
                    .mutation_bytes(file)
                    .unwrap_or_else(|absence| panic!("case {}: {absence}", case.identity));
                let replaced = loaded
                    .replaced_dependency_bytes(target, bytes)
                    .unwrap_or_else(|| panic!("case {}: missing {target}", case.identity));
                let dependencies = loaded.dependencies_over(&replaced);
                let base = loaded.expected(&inventories);
                let expected = v2::Expected {
                    inherited: artifact::Expected {
                        dependencies: &dependencies,
                        ..base.inherited
                    },
                    ..base
                };
                quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
                    .into_result()
                    .unwrap_err()
            }
        };
        assert_ne!(
            v2_at_v1, error,
            "/2 at /1 collided with mutation {}",
            case.identity
        );
        assert_ne!(
            v1_at_v2, error,
            "/1 at /2 collided with mutation {}",
            case.identity
        );
    }
}

/// SC-06a/AC-7: the version-2 expectation embeds the complete version-1
/// expectation. Mutating each of those eight inherited selection fields must
/// therefore refuse at both explicit readers; the temporal selection remains a
/// separate version-2 axis and is exercised by A's mutation manifest below.
#[test]
#[trace("IT-001", "NFR-007-AC-7", "Task-009")]
fn every_inherited_selection_axis_refuses_at_both_versions() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected_v2 = loaded.expected(&inventories);

    let (candidate_v1, reference_v1) = inherited_v1_control(&loaded, &expected_v2);
    let expected_v1 = artifact::Expected {
        artifact: &reference_v1,
        ..expected_v2.inherited
    };
    quire_protocol::intake::v1::admit(candidate_v1.bytes(), &expected_v1, loaded.limits())
        .into_result()
        .expect("the derived /1 control must admit before selection mutations");

    let mut artifact_v1 = reference_v1.clone();
    artifact_v1.digest = ByteDigest::of(b"unselected version-1 artifact");
    let mut artifact_v2 = loaded.reference().clone();
    artifact_v2.digest = ByteDigest::of(b"unselected version-2 artifact");
    let mut contract = expected_v2.inherited.contract.clone();
    contract.identity.push_str("-unselected");
    let mut baseline = expected_v2.inherited.baseline.clone();
    baseline.revision.value.push_str("-unselected");
    let mut producer = expected_v2.inherited.producer.clone();
    producer.implementation.push_str("-unselected");
    let mut language = expected_v2.inherited.language.clone();
    language.edition.push_str("-unselected");
    let no_sources = [];
    let no_dependencies = [];
    let no_models = [];

    let cases = [
        (
            "artifact",
            artifact::Expected {
                artifact: &artifact_v1,
                ..expected_v1
            },
            artifact::Expected {
                artifact: &artifact_v2,
                ..expected_v2.inherited
            },
        ),
        (
            "contract",
            artifact::Expected {
                contract: &contract,
                ..expected_v1
            },
            artifact::Expected {
                contract: &contract,
                ..expected_v2.inherited
            },
        ),
        (
            "baseline",
            artifact::Expected {
                baseline: &baseline,
                ..expected_v1
            },
            artifact::Expected {
                baseline: &baseline,
                ..expected_v2.inherited
            },
        ),
        (
            "producer",
            artifact::Expected {
                producer: &producer,
                ..expected_v1
            },
            artifact::Expected {
                producer: &producer,
                ..expected_v2.inherited
            },
        ),
        (
            "language",
            artifact::Expected {
                language: &language,
                ..expected_v1
            },
            artifact::Expected {
                language: &language,
                ..expected_v2.inherited
            },
        ),
        (
            "sources",
            artifact::Expected {
                sources: &no_sources,
                ..expected_v1
            },
            artifact::Expected {
                sources: &no_sources,
                ..expected_v2.inherited
            },
        ),
        (
            "dependencies",
            artifact::Expected {
                dependencies: &no_dependencies,
                ..expected_v1
            },
            artifact::Expected {
                dependencies: &no_dependencies,
                ..expected_v2.inherited
            },
        ),
        (
            "models",
            artifact::Expected {
                models: &no_models,
                ..expected_v1
            },
            artifact::Expected {
                models: &no_models,
                ..expected_v2.inherited
            },
        ),
    ];

    for (axis, inherited_v1, inherited_v2) in cases {
        let error_v1 =
            quire_protocol::intake::v1::admit(candidate_v1.bytes(), &inherited_v1, loaded.limits())
                .into_result()
                .unwrap_err();
        let expected_v2 = v2::Expected {
            inherited: inherited_v2,
            ..expected_v2
        };
        let error_v2 =
            quire_protocol::intake::v2::admit(loaded.offer(), &expected_v2, loaded.limits())
                .into_result()
                .unwrap_err();
        assert_eq!(
            error_v1, error_v2,
            "the inherited {axis} selection changed typed refusal across versions"
        );
        assert!(
            !error_v2.code().starts_with("v2."),
            "the inherited {axis} selection was relabeled as a version-2-only axis"
        );
    }
}

/// SC-09: every published accounting dimension admits at exact measured usage,
/// refuses one unit short with that exact dimension, and admits the unchanged
/// bytes again under a fresh exact budget.
#[test]
#[trace("IT-001", "Task-012")]
fn every_dimension_refuses_one_short_and_admits_fresh_exact_usage() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let baseline = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits());
    baseline
        .result()
        .expect("the base handoff must admit before measuring its usage");
    let usage = baseline.usage();

    for dimension in artifact::Dimension::ALL {
        let exact = usage_for(usage, dimension);
        assert!(exact > 0, "the base handoff exercises {dimension:?}");

        let mut exact_limits = loaded.limits();
        set_limit(&mut exact_limits, dimension, exact);
        let exact_report =
            quire_protocol::intake::v2::admit(loaded.offer(), &expected, exact_limits);
        assert_eq!(
            exact_report.usage(),
            usage,
            "{dimension:?} changed measured usage under the exact budget"
        );
        exact_report
            .into_result()
            .unwrap_or_else(|error| panic!("{dimension:?} refused at exact usage: {error}"));

        let mut one_short_limits = loaded.limits();
        set_limit(&mut one_short_limits, dimension, exact - 1);
        let error = quire_protocol::intake::v2::admit(loaded.offer(), &expected, one_short_limits)
            .into_result()
            .expect_err("a one-short budget must refuse before exposing a package");
        match error {
            artifact::Error::Incomplete(exhaustion) => {
                assert_eq!(exhaustion.dimension, dimension);
                assert_eq!(exhaustion.limit, exact - 1);
            }
            other => panic!("{dimension:?} one-short returned {}: {other}", other.code()),
        }

        let fresh_report =
            quire_protocol::intake::v2::admit(loaded.offer(), &expected, exact_limits);
        assert_eq!(
            fresh_report.usage(),
            usage,
            "{dimension:?} retained accounting state across a refused attempt"
        );
        fresh_report.into_result().unwrap_or_else(|error| {
            panic!("{dimension:?} retained work across a refused attempt: {error}")
        });
    }
}

/// SC-07/SC-08: every declared mutation is refused with the identity A declares
/// for it, and none is admitted.
#[test]
#[trace("IT-001", "TC-001", "FR-001-AC-2", "Task-001", "Task-012")]
fn every_declared_mutation_is_refused_with_its_declared_identity() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let limits = loaded.limits();

    let mut wrong = Vec::new();
    let mut submitted = Vec::new();
    let mut errors = Vec::new();
    for case in &loaded.mutations().cases {
        let observed = match &case.input {
            MutationInput::Offer { file, artifact } => {
                let bytes = loaded
                    .mutation_bytes(file)
                    .unwrap_or_else(|absence| panic!("case {}: {absence}", case.identity));
                let expected = loaded.expected_for(&inventories, artifact);
                outcome(
                    quire_protocol::intake::v2::admit(&bytes, &expected, limits),
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
                    quire_protocol::intake::v2::admit(loaded.offer(), &expected, limits),
                    &case.expected_refusal_code,
                )
            }
        };
        submitted.push(case.identity.as_str());
        match observed {
            Ok(error) => errors.push((case.axis.as_str(), error)),
            Err(observed) => {
                wrong.push(format!(
                    "{} (axis {}): declared {}, observed {observed}",
                    case.identity, case.axis, case.expected_refusal_code
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} mutations did not refuse as declared:\n  {}",
        wrong.len(),
        loaded.mutations().cases.len(),
        wrong.join("\n  ")
    );
    // `wrong` being empty is also what an empty corpus produces, so the corpus
    // is checked against the inventory A declared at the pinned revision. A case
    // that A stops publishing is a finding here, not a silent shrink.
    assert_eq!(
        submitted.len(),
        DECLARED_CASES,
        "{} of {DECLARED_CASES} declared cases were submitted",
        submitted.len()
    );
    for (index, (axis, error)) in errors.iter().enumerate() {
        for (other_axis, other_error) in &errors[index + 1..] {
            if axis != other_axis {
                assert_ne!(
                    error, other_error,
                    "distinct mutation axes {axis} and {other_axis} share one typed cause"
                );
            }
        }
    }
}

/// Negative control for the loop above, at the strength it actually has.
///
/// It submits the *unmutated* offer under the *base* expectation and requires
/// admission. That rules out the whole-lane failure mode — a B-assembled
/// expectation, limit set or inventory that refuses everything, which would
/// make all 28 cases read as "refusing as declared" for a reason unrelated to
/// their axes.
///
/// It does not control the per-case expectations. A `ReplaceMember` case is
/// scored against `expected_for(..)` and a `ReplaceOriginal` case against an
/// `Expected` rebuilt over the replaced dependency; neither of those is the
/// value submitted here, so a defect confined to one case's expectation is
/// invisible to this control. What bounds that instead is the refusal identity:
/// each case must refuse with the code A declared for it, and an expectation
/// assembled wrongly refuses with a different code.
#[test]
#[trace("IT-001", "Task-012")]
fn the_mutation_expectation_admits_the_unmutated_offer() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let report = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits());
    assert!(
        report.result().is_ok(),
        "the unmutated offer was refused under the mutation-loop expectation"
    );
}

/// Every axis A declared at the pinned revision is still exercised, so an axis
/// cannot drop out of the corpus unnoticed.
///
/// The corpus cannot be its own witness: an axis list derived from the cases
/// always covers the cases. `DECLARED_AXES` is what A publishes at `f1700a9`,
/// frozen here, so a dropped axis fails and an added one is reported rather
/// than absorbed.
#[test]
#[trace("Task-012")]
fn every_declared_axis_is_exercised() {
    let loaded = configured();
    let mut axes: Vec<&str> = loaded
        .mutations()
        .cases
        .iter()
        .filter(|case| !case.expected_refusal_code.is_empty())
        .map(|case| case.axis.as_str())
        .collect();
    axes.sort_unstable();
    axes.dedup();
    assert_eq!(
        axes, DECLARED_AXES,
        "A's declared mutation axes differ from the inventory frozen at the pin"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The configured handoff, or a failure.
///
/// Absence or corruption fails the default lane rather than returning a value
/// that would make the caller's assertions vacuous.
fn configured() -> Handoff {
    Handoff::load()
        .unwrap_or_else(|absence| panic!("the pinned /2 handoff could not be assembled: {absence}"))
}

/// Derive the valid `/1` control shared by the version-isolation checks from
/// the `/2` package's inherited graph. Callers still exercise each explicit
/// reader independently.
fn inherited_v1_control(
    loaded: &Handoff,
    expected_v2: &v2::Expected<'_>,
) -> (artifact::Candidate, artifact::wire::ArtifactRef) {
    let admitted_v2 =
        quire_protocol::intake::v2::admit(loaded.offer(), expected_v2, loaded.limits())
            .into_result()
            .expect("the base /2 handoff must admit before deriving the /1 control");
    let mut package_v1 = admitted_v2.inherited().clone();
    package_v1.wire = artifact::WIRE.into();
    package_v1.media = artifact::MEDIA.into();
    package_v1.schema = artifact::SCHEMA.into();
    let candidate_v1 = artifact::encode_candidate(&package_v1, loaded.limits())
        .into_result()
        .expect("the inherited package must encode under the /1 contract");
    let mut reference_v1 = loaded.reference().clone();
    reference_v1.digest = candidate_v1.digest();
    reference_v1.wire.identity = "quire.compiled-protocol".into();
    reference_v1.wire.version = "1".into();
    (candidate_v1, reference_v1)
}

fn base_admits(loaded: &Handoff, expected: &v2::Expected<'_>) {
    quire_protocol::intake::v2::admit(loaded.offer(), expected, loaded.limits())
        .into_result()
        .expect("the unchanged offer must admit under the selected baseline");
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn mutate_after(bytes: &[u8], anchor: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    let anchor_positions: Vec<_> = bytes
        .windows(anchor.len())
        .enumerate()
        .filter_map(|(index, window)| (window == anchor).then_some(index))
        .collect();
    assert_eq!(
        anchor_positions.len(),
        1,
        "mutation anchor must occur exactly once"
    );
    let start = anchor_positions[0];
    let following = &bytes[start + anchor.len()..];
    let record_end = [
        br#"},{"original_expression":"#.as_slice(),
        br#"}],"temporal":"#.as_slice(),
    ]
    .into_iter()
    .filter_map(|boundary| {
        following
            .windows(boundary.len())
            .position(|window| window == boundary)
    })
    .min()
    .map(|relative| start + anchor.len() + relative)
    .expect("anchored expression must have a record boundary");
    let target_positions: Vec<_> = bytes[start..record_end]
        .windows(before.len())
        .enumerate()
        .filter_map(|(index, window)| (window == before).then_some(index))
        .collect();
    assert_eq!(
        target_positions.len(),
        1,
        "mutation target must occur exactly once in its anchored expression"
    );
    let target = start + target_positions[0];
    let mut changed = Vec::with_capacity(bytes.len() - before.len() + after.len());
    changed.extend_from_slice(&bytes[..target]);
    changed.extend_from_slice(after);
    changed.extend_from_slice(&bytes[target + before.len()..]);
    changed
}

/// The typed error when the outcome is the declared refusal; otherwise what was observed.
fn outcome(
    report: artifact::Report<v2::AdmittedPackage>,
    declared: &str,
) -> Result<artifact::Error, &'static str> {
    match report.into_result() {
        Ok(_) => Err("admission"),
        Err(error) if handoff::matches_declared_code(&error, declared) => Ok(error),
        Err(error) => Err(handoff::reported_code(&error)),
    }
}

fn usage_for(usage: artifact::Usage, dimension: artifact::Dimension) -> usize {
    match dimension {
        artifact::Dimension::PayloadBytes => usage.payload_bytes,
        artifact::Dimension::OutputBytes => usage.output_bytes,
        artifact::Dimension::SourceBytes => usage.source_bytes,
        artifact::Dimension::ContentBytes => usage.content_bytes,
        artifact::Dimension::Sources => usage.sources,
        artifact::Dimension::Dependencies => usage.dependencies,
        artifact::Dimension::Definitions => usage.definitions,
        artifact::Dimension::Models => usage.models,
        artifact::Dimension::Declarations => usage.declarations,
        artifact::Dimension::Entries => usage.entries,
        artifact::Dimension::References => usage.references,
        artifact::Dimension::ByteWork => usage.byte_work,
        artifact::Dimension::Depth => usage.depth,
    }
}

fn set_limit(limits: &mut artifact::Limits, dimension: artifact::Dimension, value: usize) {
    match dimension {
        artifact::Dimension::PayloadBytes => limits.payload_bytes = value,
        artifact::Dimension::OutputBytes => limits.output_bytes = value,
        artifact::Dimension::SourceBytes => limits.source_bytes = value,
        artifact::Dimension::ContentBytes => limits.content_bytes = value,
        artifact::Dimension::Sources => limits.sources = value,
        artifact::Dimension::Dependencies => limits.dependencies = value,
        artifact::Dimension::Definitions => limits.definitions = value,
        artifact::Dimension::Models => limits.models = value,
        artifact::Dimension::Declarations => limits.declarations = value,
        artifact::Dimension::Entries => limits.entries = value,
        artifact::Dimension::References => limits.references = value,
        artifact::Dimension::ByteWork => limits.byte_work = value,
        artifact::Dimension::Depth => limits.depth = value,
    }
}

// ---------------------------------------------------------------------------
// The integrity mechanism, under the gate
// ---------------------------------------------------------------------------
//
// The refusals below stop a handoff B does not trust from reaching the reader.
// Each case builds a small synthetic handoff under `CARGO_TARGET_TMPDIR` and
// asserts the refusal independently of A's committed bundle.

/// A scratch directory holding one synthetic handoff.
fn scratch(name: &str) -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create the scratch handoff root");
    root
}

/// Write one member and return its `SHA256SUMS` line.
fn member(root: &std::path::Path, relative: &str, bytes: &[u8]) -> String {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create the member's directory");
    }
    std::fs::write(&path, bytes).expect("write the member");
    format!("{:x}  ./{relative}\n", ByteDigest::of(bytes))
}

/// A checksum line B cannot read refuses the handoff.
#[test]
#[trace("Task-012")]
fn a_malformed_checksum_line_is_refused() {
    let root = scratch("checksum-line");
    let sums = member(&root, handoff::PUBLISHED_OFFER_FILE, b"{}");
    std::fs::write(
        root.join(handoff::PUBLISHED_CHECKSUMS_FILE),
        format!("{sums}not-a-checksum-line\n"),
    )
    .expect("write the manifest");
    assert!(
        matches!(Handoff::load_from(&root), Err(Absence::ChecksumLine { .. })),
        "a malformed checksum line was not refused"
    );
}

/// A member whose bytes are not the ones A published refuses the handoff.
#[test]
#[trace("Task-012")]
fn a_member_that_does_not_hash_to_its_listed_digest_is_refused() {
    let root = scratch("checksum-mismatch");
    let sums = member(&root, handoff::PUBLISHED_OFFER_FILE, b"{}");
    std::fs::write(
        root.join(handoff::PUBLISHED_OFFER_FILE),
        b"{\"changed\": true}",
    )
    .expect("replace the member's bytes");
    std::fs::write(root.join(handoff::PUBLISHED_CHECKSUMS_FILE), sums).expect("write the manifest");
    assert!(
        matches!(Handoff::load_from(&root), Err(Absence::Checksum { .. })),
        "a member that does not hash to its listed digest was not refused"
    );
}

/// A file the manifest does not list carries no published integrity claim.
#[test]
#[trace("Task-012")]
fn a_file_the_manifest_does_not_list_is_refused() {
    let root = scratch("unlisted");
    let sums = member(&root, handoff::PUBLISHED_OFFER_FILE, b"{}");
    std::fs::write(root.join("mutations/smuggled.json"), b"{}").ok();
    std::fs::create_dir_all(root.join("mutations")).expect("create the mutation directory");
    std::fs::write(root.join("mutations/smuggled.json"), b"{}").expect("write the unlisted file");
    std::fs::write(root.join(handoff::PUBLISHED_CHECKSUMS_FILE), sums).expect("write the manifest");
    assert!(
        matches!(Handoff::load_from(&root), Err(Absence::Unlisted { .. })),
        "a file absent from the manifest was not refused"
    );
}

/// A published path that leaves the handoff root is refused.
///
/// This is the hole the coverage sweep cannot see: the sweep walks the root, so
/// bytes named outside it are never compared against anything. Without the
/// containment check the manifest could name `../evil.json`, B would read it,
/// and it would reach `intake::v2::admit` with no published claim behind it.
#[test]
#[trace("Task-012")]
fn a_published_path_that_escapes_the_root_is_refused() {
    let root = scratch("escaping");
    let outside = root.parent().expect("a parent").join("outside-evil.json");
    std::fs::write(&outside, b"{}").expect("write the file outside the root");
    let digest = format!("{:x}", ByteDigest::of(b"{}"));
    std::fs::write(
        root.join(handoff::PUBLISHED_CHECKSUMS_FILE),
        format!("{digest}  ./../outside-evil.json\n"),
    )
    .expect("write the manifest");
    assert!(
        matches!(
            Handoff::load_from(&root),
            Err(Absence::EscapingMember { .. })
        ),
        "a path escaping the handoff root was not refused"
    );
    let _ = std::fs::remove_file(&outside);
}

/// A symbolic link's bytes are not the published ones.
#[cfg(unix)]
#[test]
#[trace("Task-012")]
fn a_symbolic_link_in_the_handoff_is_refused() {
    let root = scratch("symlink");
    let sums = member(&root, handoff::PUBLISHED_OFFER_FILE, b"{}");
    std::os::unix::fs::symlink(handoff::PUBLISHED_OFFER_FILE, root.join("linked.json"))
        .expect("create the link");
    std::fs::write(root.join(handoff::PUBLISHED_CHECKSUMS_FILE), sums).expect("write the manifest");
    assert!(
        matches!(Handoff::load_from(&root), Err(Absence::SymbolicLink { .. })),
        "a symbolic link in the handoff was not refused"
    );
}

/// A ceiling above A's own hard maximum did not come from A's publisher.
#[test]
#[trace("Task-012")]
fn a_ceiling_above_a_s_hard_maximum_is_refused() {
    let mut selected = handoff::SelectedArtifactLimits::try_new(artifact::Limits::default())
        .expect("A's hard maxima fit the published record");
    assert!(
        handoff::selected_limits(&selected).is_ok(),
        "A's own hard maxima were refused"
    );
    selected.payload_bytes += 1;
    assert!(
        matches!(
            handoff::selected_limits(&selected),
            Err(Absence::ExcessiveLimit {
                dimension: artifact::Dimension::PayloadBytes,
                ..
            })
        ),
        "a ceiling above A's hard maximum was not refused"
    );
}

/// A's producer runs publish under one content-addressed directory: a second
/// run reuses the first run's directory, whose bytes are unchanged, and that is
/// the directory the lane loads.
///
/// Tracing: Task-012.
#[test]
#[trace("Task-012")]
fn producer_runs_publish_once_and_are_reused() {
    let first = handoff::qsl_producer::emit_and_publish().expect("first producer run");
    let inventory = |root: &std::path::Path| {
        ["v1", "v2"].map(|version| {
            std::fs::read(root.join(version).join("SHA256SUMS")).expect("published inventory")
        })
    };
    let before = inventory(&first);
    let second = handoff::qsl_producer::emit_and_publish().expect("second producer run");
    assert_eq!(
        first, second,
        "an identical run must reuse the published directory"
    );
    assert_eq!(
        inventory(&second),
        before,
        "a reused run must keep its bytes"
    );
    assert_eq!(
        handoff::qsl_producer::v2().expect("lane handoff"),
        second.join("v2"),
        "the lane must load the published directory"
    );
    let name = first
        .file_name()
        .and_then(|name| name.to_str())
        .expect("published directory name");
    assert!(
        name.strip_prefix("handoff-")
            .is_some_and(|digest| digest.len() == 64),
        "published directory is content-addressed: {name}"
    );
}

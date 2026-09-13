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
use quire_spec_language::{
    protocol_artifact::{self as artifact, v2},
    ByteDigest,
};

#[path = "support/v2_handoff/mod.rs"]
mod handoff;

use handoff::{Absence, Handoff, MutationInput};

/// A's mutation corpus as published at the pinned revision `04254a3`, sorted.
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
        Err(absence) => {
            panic!("a /2 handoff was configured but could not be assembled: {absence}")
        }
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
#[ignore = "requires QUIRE_PROTOCOL_A_V2_HANDOFF; see A_OWED_API"]
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
#[ignore = "requires QUIRE_PROTOCOL_A_V2_HANDOFF; see A_OWED_API"]
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
#[ignore = "requires QUIRE_PROTOCOL_A_V2_HANDOFF; see A_OWED_API"]
#[trace("Task-012")]
fn the_published_handoff_is_admitted_and_linked() {
    let loaded = configured();
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
#[ignore = "requires QUIRE_PROTOCOL_A_V2_HANDOFF; see A_OWED_API"]
#[trace("Task-012")]
fn every_declared_mutation_is_refused_with_its_declared_identity() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let limits = loaded.limits();

    let mut wrong = Vec::new();
    let mut submitted = Vec::new();
    for case in &loaded.mutations().cases {
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
        submitted.push(case.identity.as_str());
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
    // `wrong` being empty is also what an empty corpus produces, so the corpus
    // is checked against the inventory A declared at the pinned revision. A case
    // that A stops publishing is a finding here, not a silent shrink.
    assert_eq!(
        submitted.len(),
        DECLARED_CASES,
        "{} of {DECLARED_CASES} declared cases were submitted",
        submitted.len()
    );
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
#[ignore = "requires QUIRE_PROTOCOL_A_V2_HANDOFF; see A_OWED_API"]
#[trace("Task-012")]
fn the_mutation_expectation_admits_the_unmutated_offer() {
    let loaded = configured();
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let report = quire_protocol::admit_and_link_v2(loaded.offer(), &expected, loaded.limits());
    assert!(
        report.result().is_ok(),
        "the unmutated offer was refused under the mutation-loop expectation"
    );
}

/// Every axis A declared at the pinned revision is still exercised, so an axis
/// cannot drop out of the corpus unnoticed.
///
/// The corpus cannot be its own witness: an axis list derived from the cases
/// always covers the cases. `DECLARED_AXES` is what A published at `04254a3`,
/// frozen here, so a dropped axis fails and an added one is reported rather
/// than absorbed.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_V2_HANDOFF; see A_OWED_API"]
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
/// Every caller is `#[ignore]`d into the handoff lane, so reaching this function
/// means the lane was asked for by name and a handoff must be there. An absent
/// one fails rather than returning a value that would make the caller's
/// assertions vacuous.
fn configured() -> Handoff {
    match Handoff::load() {
        Ok(loaded) => loaded,
        Err(absence) => panic!(
            "the /2 handoff lane was run but no handoff could be assembled: {absence}.\n\
             What B still needs from A:\n  - {}",
            handoff::A_OWED_API.join("\n  - ")
        ),
    }
}

/// `None` when the outcome is the declared refusal; otherwise what was observed.
fn outcome(report: artifact::Report<v2::AdmittedPackage>, declared: &str) -> Option<&'static str> {
    match report.result() {
        Ok(_) => Some("admission"),
        Err(error) if handoff::matches_declared_code(error, declared) => None,
        Err(error) => Some(handoff::reported_code(error)),
    }
}

// ---------------------------------------------------------------------------
// The integrity mechanism, under the gate
// ---------------------------------------------------------------------------
//
// The lane above needs A's 13 MB handoff, which this repository does not retain,
// so it runs only when asked for by name. The refusals below are what stops a
// handoff B does not trust from reaching the reader, and they must be checked on
// every run — a mechanism exercised only out of band is a mechanism nobody
// gates. Each case builds a small synthetic handoff under `CARGO_TARGET_TMPDIR`
// and asserts the refusal, so no adverse input ever needs A's bundle.

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
    let sums = member(&root, "compiled-protocol-v2.json", b"{}");
    std::fs::write(
        root.join("SHA256SUMS"),
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
    let sums = member(&root, "compiled-protocol-v2.json", b"{}");
    std::fs::write(
        root.join("compiled-protocol-v2.json"),
        b"{\"changed\": true}",
    )
    .expect("replace the member's bytes");
    std::fs::write(root.join("SHA256SUMS"), sums).expect("write the manifest");
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
    let sums = member(&root, "compiled-protocol-v2.json", b"{}");
    std::fs::write(root.join("mutations/smuggled.json"), b"{}").ok();
    std::fs::create_dir_all(root.join("mutations")).expect("create the mutation directory");
    std::fs::write(root.join("mutations/smuggled.json"), b"{}").expect("write the unlisted file");
    std::fs::write(root.join("SHA256SUMS"), sums).expect("write the manifest");
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
/// and it would reach `admit_and_link_v2` with no published claim behind it.
#[test]
#[trace("Task-012")]
fn a_published_path_that_escapes_the_root_is_refused() {
    let root = scratch("escaping");
    let outside = root.parent().expect("a parent").join("outside-evil.json");
    std::fs::write(&outside, b"{}").expect("write the file outside the root");
    let digest = format!("{:x}", ByteDigest::of(b"{}"));
    std::fs::write(
        root.join("SHA256SUMS"),
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
    let sums = member(&root, "compiled-protocol-v2.json", b"{}");
    std::os::unix::fs::symlink("compiled-protocol-v2.json", root.join("linked.json"))
        .expect("create the link");
    std::fs::write(root.join("SHA256SUMS"), sums).expect("write the manifest");
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

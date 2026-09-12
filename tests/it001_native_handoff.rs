// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001/Task-009: actual A-owned four-unit compiled-protocol handoff.

use std::{fs, path::Path};

use ix_trace_rs::trace;
use quire_contract_ir as ir;
use quire_protocol::{
    admit_and_link,
    digest::producer::{CrossDomainDigest, DigestDomain, DigestSlot, OfferedDomain},
    inventory::{
        axis::DivergedSelection,
        map::{admit_discriminating, AdmissionRefusal},
        SelectedInventory,
    },
    wire, AdmittedModel, AdmittedPackage, ByteDigest, Expected, ExpectedDeclaration,
    ExpectedForeignSource, ExpectedSource, Invalid, Limits, NumberComponent, NumberError,
    SuppliedDependency,
};
use quire_spec_language::{
    formal_source::FormalSource,
    model_source::{self, ModelSourceLimits},
    native_model::ModelLimits,
    Source, SourceIdentity,
};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    artifact: wire::ArtifactRef,
    contract: wire::ArtifactRef,
    baseline: wire::ArtifactRef,
    producer: wire::Producer,
    language: wire::Language,
    sources: Vec<SelectedSource>,
    dependencies: Vec<SelectedDependency>,
    model: SelectedModel,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedSource {
    source: wire::Source,
    file: String,
    declarations: Vec<SelectedDeclaration>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedDeclaration {
    name: String,
    span: wire::Span,
    requirement: wire::Requirement,
    clause: String,
    execution: wire::Execution,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedDependency {
    artifact: wire::ArtifactRef,
    file: String,
    requires: Vec<wire::ArtifactRef>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedModel {
    artifact: wire::ArtifactRef,
    source: wire::Source,
    source_file: String,
    source_format: String,
}

struct Fixture {
    selection: Selection,
    dependency_bytes: Vec<Vec<u8>>,
    model: quire_spec_language::native_model::NativeModel,
}

impl Fixture {
    fn load(root: &Path) -> Self {
        // This B-owned immutable selection fixture is loaded before the offer.
        // It is distinct from A's emitted `expected.json`, which the harness
        // never reads. The offered package can therefore not select its own
        // acceptance context.
        let selection: Selection =
            serde_json::from_slice(include_bytes!("fixtures/a-four-unit-selections.json"))
                .expect("B selection fixture shape");
        assert!(selection
            .sources
            .iter()
            .all(|source| !source.file.is_empty()));
        let model_source = &selection.model.source;
        let model_bytes = fs::read(root.join(&selection.model.source_file)).expect("model bytes");
        assert_eq!(ByteDigest::of(&model_bytes), model_source.artifact.digest);
        let source = Source::read(
            SourceIdentity {
                identity: model_source.native.identity.clone(),
                revision: model_source.native.revision.clone(),
            },
            &model_source.path,
            &model_bytes,
            quire_spec_language::Limits::default().source_bytes,
        )
        .expect("model source");
        let formal = FormalSource::new(
            source,
            ir::SourceIdentity::new(
                ir::SourceDocumentId::new(&model_source.formal.document).expect("formal document"),
                ir::SourceRevision::new(
                    model_source
                        .formal
                        .revision
                        .value
                        .parse()
                        .expect("formal revision"),
                )
                .expect("revision"),
            ),
        );
        let model = model_source::read(
            formal,
            &selection.model.source_format,
            ModelSourceLimits::default(),
        )
        .expect("admitted model draft")
        .admit(ModelLimits::default())
        .expect("admitted model");
        assert_eq!(
            model.artifact_bytes(),
            dependency_bytes(root, &selection.dependencies, &selection.model.artifact)
        );
        let dependency_bytes = selection
            .dependencies
            .iter()
            .map(|dependency| fs::read(root.join(&dependency.file)).expect("dependency bytes"))
            .collect();
        Self {
            selection,
            dependency_bytes,
            model,
        }
    }

    fn admit(&self, bytes: &[u8]) -> quire_protocol::Report<quire_protocol::AdmittedPackage> {
        self.admit_selected(&self.selection, bytes)
    }

    fn admit_selected(
        &self,
        selection: &Selection,
        bytes: &[u8],
    ) -> quire_protocol::Report<quire_protocol::AdmittedPackage> {
        self.offer(selection, bytes, |bytes, expected| {
            admit_and_link(bytes, expected, Limits::default())
        })
    }

    /// Offer the same bytes through B's discriminating consumer boundary, which
    /// resolves A's single shared `Invalid::Selection` code against B's own
    /// independently selected inventory and guards the digest domains first.
    fn discriminate(
        &self,
        selection: &Selection,
        bytes: &[u8],
    ) -> Result<AdmittedPackage, AdmissionRefusal> {
        self.offer(selection, bytes, |bytes, expected| {
            admit_discriminating(bytes, expected, Limits::default())
        })
    }

    /// Build the independently selected acceptance context once and hand it to
    /// the chosen consumer entry point.
    fn offer<T>(
        &self,
        selection: &Selection,
        bytes: &[u8],
        entry: impl FnOnce(&[u8], &Expected<'_>) -> T,
    ) -> T {
        assert_eq!(
            selection.dependencies.len(),
            self.dependency_bytes.len(),
            "each selected dependency must have supplied bytes"
        );
        let dependencies = selection
            .dependencies
            .iter()
            .zip(&self.dependency_bytes)
            .map(|(dependency, bytes)| SuppliedDependency {
                artifact: &dependency.artifact,
                bytes,
                requires: &dependency.requires,
            })
            .collect::<Vec<_>>();
        let foreign = ExpectedForeignSource {
            artifact: &selection.model.source.artifact,
            formal: &selection.model.source.formal,
            bytes: self.model.source().source().text().as_bytes(),
        };
        let models = [AdmittedModel {
            artifact: &selection.model.artifact,
            model: &self.model,
            source: &foreign,
        }];
        let sources = selection
            .sources
            .iter()
            .map(|source| {
                let declarations = source
                    .declarations
                    .iter()
                    .map(|item| ExpectedDeclaration {
                        name: &item.name,
                        span: &item.span,
                        requirement: &item.requirement,
                        clause: &item.clause,
                        execution: &item.execution,
                    })
                    .collect::<Vec<_>>();
                (source, declarations)
            })
            .collect::<Vec<_>>();
        let expected_sources = sources
            .iter()
            .map(|(source, declarations)| ExpectedSource {
                artifact: &source.source.artifact,
                native: &source.source.native,
                path: &source.source.path,
                formal: &source.source.formal,
                text: &source.source.text,
                declarations,
            })
            .collect::<Vec<_>>();
        entry(
            bytes,
            &Expected {
                artifact: &selection.artifact,
                contract: &selection.contract,
                baseline: &selection.baseline,
                producer: &selection.producer,
                language: &selection.language,
                sources: &expected_sources,
                dependencies: &dependencies,
                models: &models,
            },
        )
    }
}

fn dependency_bytes(
    root: &Path,
    dependencies: &[SelectedDependency],
    expected: &wire::ArtifactRef,
) -> Vec<u8> {
    let dependency = dependencies
        .iter()
        .find(|item| item.artifact == *expected)
        .expect("selected model dependency");
    fs::read(root.join(&dependency.file)).expect("selected model bytes")
}

fn fixture_root() -> std::path::PathBuf {
    std::env::var_os("QUIRE_PROTOCOL_A_FIXTURE")
        .map(std::path::PathBuf::from)
        .expect("set QUIRE_PROTOCOL_A_FIXTURE to A's generated four-unit fixture directory")
}

fn resealed(selection: &Selection, bytes: &[u8]) -> Selection {
    let mut selection = selection.clone();
    selection.artifact.digest = ByteDigest::of(bytes);
    selection
}

fn replace_once(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    assert_eq!(
        before.len(),
        after.len(),
        "mutation preserves compact wire length"
    );
    let index = bytes
        .windows(before.len())
        .position(|window| window == before)
        .expect("fixture contains mutation target");
    let mut changed = bytes.to_vec();
    changed[index..index + before.len()].copy_from_slice(after);
    changed
}

/// The complete top-level `baseline` field as A's producer emits it. Scoping a
/// mutation to this literal keeps the axis under test on the baseline selection:
/// the same artifact also appears in the dependency inventory, and a mutation
/// that moved both could be refused by either check.
const BASELINE_FIELD: &str = concat!(
    r#""baseline":{"refVersion":"ix.artifact-ref/3-draft","kind":"source","#,
    r#""authority":"ix://agent-ix/quire-specification","identity":"ix:native","#,
    r#""revision":{"namespace":"quire/native-definition-revision","value":"1-draft.2"},"#,
    r#""digest":"sha256:6f9cb8bf8521ca51a5e6340511b545db43001879446a59c5c87efe4985acdb06","#,
    r#""wire":{"identity":"text/markdown","version":"1"}}"#,
);

/// Replace a target that must occur exactly once, so a future A fixture cannot
/// silently widen a one-axis mutation across additional fields.
fn replace_unique(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    assert_eq!(
        before.len(),
        after.len(),
        "mutation preserves compact wire length"
    );
    let occurrences = bytes
        .windows(before.len())
        .filter(|window| *window == before)
        .count();
    assert_eq!(occurrences, 1, "mutation target must identify one field");
    replace_once(bytes, before, after)
}

/// One baseline field with exactly one substring respelled in place.
fn mutated_baseline_field(before: &str, after: &str) -> String {
    assert_eq!(before.len(), after.len());
    assert_eq!(BASELINE_FIELD.matches(before).count(), 1);
    BASELINE_FIELD.replace(before, after)
}

/// An offered integer value in the declaration graph, typed by the model scalar
/// `Tally` (domain 0..=5). Anchoring on the enclosing value operation keeps the
/// numeric mutations on a *value*; the same spelling also appears as a type
/// bound, where a different check would refuse it.
const INTEGER_VALUE: &str =
    r#""operation":{"kind":"number","value":{"kind":"integer","decimal":"5"}}"#;

/// The offered rational value, typed by the model scalar `Ratio`.
const RATIONAL_VALUE: &str = r#""operation":{"kind":"number","value":{"kind":"rational","numerator":"1","denominator":"1"}}"#;

/// `Tally`'s declared representation bound, which must equal the selected
/// model's own bound rather than merely admit the values that occur.
const INTEGER_BOUND: &str =
    r#""minimum":{"kind":"integer","decimal":"0"},"maximum":{"kind":"integer","decimal":"5"}"#;

/// Replace a target that must occur exactly once, without the compact-length
/// constraint: a numeric spelling's exact decimal width is the property under
/// test, so the mutation may resize the payload.
fn respell_unique(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    let occurrences = bytes
        .windows(before.len())
        .filter(|window| *window == before)
        .count();
    assert_eq!(occurrences, 1, "mutation target must identify one field");
    let index = bytes
        .windows(before.len())
        .position(|window| window == before)
        .expect("fixture contains mutation target");
    let mut changed = Vec::with_capacity(bytes.len() - before.len() + after.len());
    changed.extend_from_slice(&bytes[..index]);
    changed.extend_from_slice(after);
    changed.extend_from_slice(&bytes[index + before.len()..]);
    changed
}

/// Respell one numeric field inside an anchored, unique enclosing context.
fn respell_within(bytes: &[u8], anchor: &str, before: &str, after: &str) -> Vec<u8> {
    assert_eq!(anchor.matches(before).count(), 1);
    respell_unique(
        bytes,
        anchor.as_bytes(),
        anchor.replace(before, after).as_bytes(),
    )
}

/// Re-encode an admitted package through A's accepted canonical encoder and
/// require the exact offered bytes back.
///
/// A's reader already compares its own canonical re-encoding before admitting,
/// so this cannot fail for a package that admits. What it adds is that the
/// round-trip is reproducible through B's public boundary under a fresh
/// `Limits::default()` budget, and that no number is echoed on the way: every
/// admitted number is re-spelled from its validated signed-64 components, so
/// byte identity here also pins the canonical numeric spelling.
fn assert_round_trips(admitted: &quire_protocol::AdmittedPackage, offered: &[u8]) {
    let candidate = quire_spec_language::protocol_artifact::encode_candidate(
        admitted.package(),
        Limits::default(),
    )
    .into_result()
    .expect("canonical re-encoding");
    assert_eq!(candidate.bytes(), offered, "re-encoded bytes must be exact");
    assert_eq!(candidate.digest(), admitted.digest());
}

fn numeric_error(report: quire_protocol::Report<quire_protocol::AdmittedPackage>) -> NumberError {
    match report.into_result().unwrap_err() {
        quire_protocol::Error::Numeric(error) => error,
        other => panic!("expected a numeric refusal, got {other:?}"),
    }
}

/// Return the first Cargo.lock package paragraph whose `name` exactly matches
/// `name`; this is a static dependency-boundary assertion, not a runtime
/// package reader.
fn locked_package(name: &str) -> &str {
    include_str!("../Cargo.lock")
        .split("[[package]]")
        .find(|paragraph| {
            paragraph
                .lines()
                .any(|line| line == format!(r#"name = "{name}""#))
        })
        .expect("locked package")
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_inventory_identifies_one_b_owned_reader_path() {
    // This is intentionally source/dependency inspection: the absence of a
    // second B-owned parser is a containment property with no runtime input
    // that can exercise it. It does not make a claim about A's compiler
    // internals, which remain A-owned behind this one public reader call.
    let manifest = include_str!("../Cargo.toml");
    let production = manifest
        .split("[dependencies]\n")
        .nth(1)
        .expect("production dependency section")
        .split("\n[dev-dependencies]")
        .next()
        .expect("production dependency terminator");
    let dependencies = production
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name.trim()))
        .collect::<Vec<_>>();
    assert_eq!(dependencies, ["quire-spec-language", "serde_json"]);

    let consumer = locked_package("quire-protocol");
    assert!(consumer.contains(r#""quire-spec-language""#));
    assert!(consumer.contains(r#""serde_json""#));
    assert!(!consumer.contains(r#""logos""#));

    let facade = include_str!("../src/lib.rs");
    let reader_call = "quire_spec_language::protocol_artifact::read(bytes, expected, limits)";
    assert_eq!(facade.matches(reader_call).count(), 1);
    assert!(!facade.contains("pub mod parser;"));
    assert!(!facade.contains("pub mod frontend;"));
}

/// Tracing: IT-001, FR-001-AC-1, Task-009.
///
/// This scenario links the handoff, so `FR-002-AC-1` would be the natural tag.
/// It is deliberately absent: the owner has ruled AC-1 unresolved until runtime
/// instance identities exist (issue #4), and A carries binding requirements
/// rather than instances, so nothing here can fail when AC-1 is violated.
/// Tagging it would count the criterion as backed by a test that cannot check
/// it — and this test is `#[ignore]`d besides, so it would be backed by a test
/// that does not run.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "FR-001-AC-1", "Task-009")]
fn it_001_accepts_and_links_the_actual_four_unit_handoff() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("positive admission");
    assert_eq!(admitted.package().sources.len(), 4);
    assert_eq!(admitted.package().declarations.len(), 7);
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_changed_artifact_bytes_before_linking() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let mut bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    bytes.push(b' ');
    let error = fixture.admit(&bytes).into_result().unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Invalid(Invalid::Seal)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_producer_identity_change() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // `native_protocol_handoff` also spells the producer binary identity and a
    // dependency identity; anchor on the implementation field so this stays the
    // producer-identity axis whatever order A emits the payload in.
    let changed = replace_unique(
        &bytes,
        br#""implementation":"quire-spec-language/native_protocol_handoff""#,
        br#""implementation":"quire-spec-language/native_protocol_handovv""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    // The producer implementation label is its own axis: A collapses it into
    // the shared `Invalid::Selection`, so this asserts B's own producer-identity
    // code, which no other landed mutation can produce.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection(DivergedSelection::ProducerIdentity),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_unknown_feature() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    let changed = replace_once(
        &bytes,
        b"quire.protocol.values/1",
        b"quire.protocol.valuex/1",
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Unsupported(_)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, FR-001-AC-2, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "FR-001-AC-2", "Task-009")]
fn it_001_refuses_a_mutated_referenced_operation_binding_before_linking() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // BeforeApply's execution binding actually references Workflow::apply at
    // model export 11. Pointing it at Workflow (export 10) keeps the compact
    // representation well-formed but disagrees with B's independently selected
    // declaration inventory, so selection must refuse before package exposure.
    let changed = replace_unique(
        &bytes,
        br#""execution":{"kind":"pre","operation":{"model":0,"export":11}}"#,
        br#""execution":{"kind":"pre","operation":{"model":0,"export":10}}"#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Invalid(Invalid::Selection)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_non_numeric_integer_component() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // This alters one exact numeric component but leaves compact JSON framing
    // untouched. Resealing proves that numeric validation, rather than only the
    // outer artifact digest, refuses the value before a linked package exists.
    let changed = respell_within(
        &bytes,
        INTEGER_VALUE,
        r#""decimal":"5""#,
        r#""decimal":"x""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Numeric(_)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_wire_version_change() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    let changed = replace_once(
        &bytes,
        b"\"wire\":\"quire.compiled-protocol/1\"",
        b"\"wire\":\"quire.compiled-protocol/2\"",
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Unsupported(quire_protocol::Unsupported::Wire)
        ),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_stale_baseline_selection() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // Only the top-level baseline revision moves. B's frozen selection, not the
    // offer, decides which semantic baseline is accepted.
    let changed = replace_unique(
        &bytes,
        BASELINE_FIELD.as_bytes(),
        mutated_baseline_field("\"value\":\"1-draft.2\"", "\"value\":\"1-draft.7\"").as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    // A superseded revision of the selected baseline identity: the baseline
    // selection state moved, not the baseline's identity and not its digest.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection(DivergedSelection::BaselineSelectionState),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_changed_baseline_digest() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // The same artifact stays correct in the dependency inventory, so only the
    // baseline digest check can refuse this offer.
    let changed = replace_unique(
        &bytes,
        BASELINE_FIELD.as_bytes(),
        mutated_baseline_field("4985acdb06", "4985acdb07").as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    // The value diverges inside the baseline's own producer digest domain, so
    // this is the digest axis and not the cross-domain axis below.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection(DivergedSelection::BaselineDigest),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_cross_domain_baseline_digest() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // Only the baseline digest's domain label moves; its hexadecimal body and
    // every other digest in the offer are unchanged.
    let changed = replace_unique(
        &bytes,
        BASELINE_FIELD.as_bytes(),
        mutated_baseline_field("\"sha256:6f9cb8", "\"sha512:6f9cb8").as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    // A parse failure would prove nothing here: it would only say the label is
    // unknown to A's `ByteDigest`, which a well-formed substitution avoids
    // entirely. B must refuse this on the digest-domain axis itself.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::CrossDomainDigest(CrossDomainDigest {
            slot: DigestSlot::Baseline,
            selected: DigestDomain::RawArtifact,
            offered: OfferedDomain::ForeignLabel("sha512".to_owned()),
            value: "sha512:6f9cb8bf8521ca51a5e6340511b545db43001879446a59c5c87efe4985acdb06"
                .to_owned(),
        }),
        "got {error:?}"
    );
}

/// The load-bearing case: a cross-domain substitution that **parses cleanly**.
///
/// The offered value is a syntactically perfect `sha256:` digest. It is simply
/// the digest of a different domain — B's own RFC 8785/JCS protocol-result
/// digest of the same baseline reference, copied into the slot that accepts
/// only the producer's exact-bytes digest. Nothing about its spelling is wrong,
/// so no parser and no syntactic guard can refuse it; only an independently
/// selected inventory that holds both domains can tell the two apart.
///
/// Tracing: IT-001, FR-001-AC-4.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "FR-001-AC-4")]
fn it_001_refuses_well_formed_cross_domain_baseline_digest() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    let result_domain = fixture.offer(&fixture.selection, &bytes, |_, expected| {
        SelectedInventory::from_expected(expected)
            .expect("B selections canonicalise")
            .result_domain_text(DigestSlot::Baseline)
            .expect("baseline is a selected slot")
    });
    let selected_digest = fixture.selection.baseline.digest.to_string();
    assert_ne!(result_domain, selected_digest);
    assert!(result_domain.starts_with("sha256:"));
    let changed = replace_unique(
        &bytes,
        BASELINE_FIELD.as_bytes(),
        mutated_baseline_field(&selected_digest, &result_domain).as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::CrossDomainDigest(CrossDomainDigest {
            slot: DigestSlot::Baseline,
            selected: DigestDomain::RawArtifact,
            offered: OfferedDomain::Identified(DigestDomain::VerificationJcs),
            value: result_domain,
        }),
        "got {error:?}"
    );
}

/// Tracing: IT-001, FR-001-AC-4, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "FR-001-AC-4", "Task-009")]
fn it_001_round_trips_the_admitted_numeric_wire_profile() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("positive admission");
    assert_round_trips(&admitted, &bytes);
}

/// Tracing: IT-001, FR-001-AC-4, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "FR-001-AC-4", "Task-009")]
fn it_001_round_trips_signed_64_field_boundaries() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // These literals occur in the reachable `Workflow.wide` path through
    // Healthy -> Bounded, not merely in an unused scalar declaration.
    for decimal in ["-9223372036854775808", "9223372036854775807"] {
        let operation = format!(
            r#""operation":{{"kind":"number","value":{{"kind":"integer","decimal":"{decimal}"}}}}"#
        );
        assert!(
            bytes
                .windows(operation.len())
                .any(|window| window == operation.as_bytes()),
            "fixture must carry {decimal} as a compiled numeric operation"
        );
    }
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("endpoint admission");
    assert_round_trips(&admitted, &bytes);
}

/// Tracing: IT-001, FR-001-AC-4, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "FR-001-AC-4", "Task-009")]
fn it_001_round_trips_exact_rational_boundaries() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // The independent model selection gives Exact its signed-64 numerator and
    // denominator domains; Bounded references both emitted endpoint pairs.
    for value in [
        r#"{"kind":"rational","numerator":"9223372036854775807","denominator":"9223372036854775806"}"#,
        r#"{"kind":"rational","numerator":"-9223372036854775808","denominator":"9223372036854775807"}"#,
    ] {
        let operation = format!(r#""operation":{{"kind":"number","value":{value}}}"#);
        assert!(
            bytes
                .windows(operation.len())
                .any(|window| window == operation.as_bytes()),
            "fixture must carry reduced exact boundary {value} as a compiled numeric operation"
        );
    }
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("endpoint admission");
    assert_round_trips(&admitted, &bytes);
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_non_canonical_decimal() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // Every spelling below denotes a value the field admits. Only the spelling
    // is wrong, so a reader that repaired text instead of refusing it would
    // admit each one.
    for spelling in ["05", "+5", "5.0", " 5", "-0"] {
        let changed = respell_within(
            &bytes,
            INTEGER_VALUE,
            r#""decimal":"5""#,
            &format!(r#""decimal":"{spelling}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::NonCanonicalDecimal {
                component: NumberComponent::Decimal,
            },
            "spelling {spelling:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_unsafe_json_number() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // A bare JSON number is the form that would silently acquire a float or
    // double interpretation on the way in. Each is refused at shape, before any
    // numeric component is admitted, so none reaches the exact-decimal codec:
    // a bare integer fails the required string shape, and every other bare form
    // fails the structural-integer rule that keeps JSON numbers out of the
    // exact numeric domain entirely.
    let changed = respell_within(&bytes, INTEGER_VALUE, r#""decimal":"5""#, r#""decimal":5"#);
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Json { .. }),
        "bare 5: got {error:?}"
    );

    for bare in ["5.0", "1e3", "-0", "9223372036854775808"] {
        let changed = respell_within(
            &bytes,
            INTEGER_VALUE,
            r#""decimal":"5""#,
            &format!(r#""decimal":{bare}"#),
        );
        let selection = resealed(&fixture.selection, &changed);
        let error = fixture
            .admit_selected(&selection, &changed)
            .into_result()
            .unwrap_err();
        assert!(
            matches!(
                error,
                quire_protocol::Error::Invalid(Invalid::StructuralInteger)
            ),
            "bare {bare}: got {error:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_out_of_range_component() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // One past each signed-64 endpoint: canonically spelled, outside the domain.
    for spelling in ["9223372036854775808", "-9223372036854775809"] {
        let changed = respell_within(
            &bytes,
            INTEGER_VALUE,
            r#""decimal":"5""#,
            &format!(r#""decimal":"{spelling}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::ComponentOutOfRange {
                component: NumberComponent::Decimal,
            },
            "spelling {spelling:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_labels_the_refused_rational_component() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // The refusal names the component that was wrong, so a caller can act on it
    // without reading diagnostic text. A numerator failure is not a decimal one.
    let changed = respell_within(
        &bytes,
        RATIONAL_VALUE,
        r#""numerator":"1""#,
        r#""numerator":"01""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    assert_eq!(
        numeric_error(fixture.admit_selected(&selection, &changed)),
        NumberError::NonCanonicalDecimal {
            component: NumberComponent::Numerator,
        }
    );

    let changed = respell_within(
        &bytes,
        RATIONAL_VALUE,
        r#""denominator":"1""#,
        r#""denominator":"9223372036854775808""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    assert_eq!(
        numeric_error(fixture.admit_selected(&selection, &changed)),
        NumberError::ComponentOutOfRange {
            component: NumberComponent::Denominator,
        }
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_non_positive_denominator() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    for denominator in ["0", "-1"] {
        let changed = respell_within(
            &bytes,
            RATIONAL_VALUE,
            r#""denominator":"1""#,
            &format!(r#""denominator":"{denominator}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::NonPositiveDenominator,
            "denominator {denominator:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_unreduced_rational() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // 2/4 denotes the admitted 1/2 and 0/2 denotes the admitted 0/1. Both are
    // inside the field's domain, so only the reduction rule can refuse them,
    // and the reader must refuse rather than reduce.
    for (numerator, denominator) in [("2", "4"), ("0", "2")] {
        let changed = respell_within(
            &bytes,
            RATIONAL_VALUE,
            r#""numerator":"1","denominator":"1""#,
            &format!(r#""numerator":"{numerator}","denominator":"{denominator}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::UnreducedRational,
            "{numerator}/{denominator}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_a_value_outside_its_declared_field_domain() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // Well-formed under the numeric wire profile, and outside the domain the
    // declaration's own type gives the field. Numeric well-formedness alone
    // therefore never admits a value.
    let integer = respell_within(
        &bytes,
        INTEGER_VALUE,
        r#""decimal":"5""#,
        r#""decimal":"9223372036854775807""#,
    );
    let selection = resealed(&fixture.selection, &integer);
    let error = fixture
        .admit_selected(&selection, &integer)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::NumericDomain)
        ),
        "integer: got {error:?}"
    );

    let rational = respell_within(
        &bytes,
        RATIONAL_VALUE,
        r#""numerator":"1","denominator":"1""#,
        r#""numerator":"-9223372036854775808","denominator":"9223372036854775807""#,
    );
    let selection = resealed(&fixture.selection, &rational);
    let error = fixture
        .admit_selected(&selection, &rational)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::NumericDomain)
        ),
        "rational: got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_a_substituted_numeric_kind() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // Integer 5 and rational 5/1 denote one quantity in two admitted forms.
    // In a value position the substitution leaves the field's declared integer
    // domain; in an integer-only bound position it is refused as a wrong kind.
    let value = respell_within(
        &bytes,
        INTEGER_VALUE,
        r#"{"kind":"integer","decimal":"5"}"#,
        r#"{"kind":"rational","numerator":"5","denominator":"1"}"#,
    );
    let selection = resealed(&fixture.selection, &value);
    let error = fixture
        .admit_selected(&selection, &value)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::NumericDomain)
        ),
        "value: got {error:?}"
    );

    let bound = respell_within(
        &bytes,
        INTEGER_BOUND,
        r#""maximum":{"kind":"integer","decimal":"5"}"#,
        r#""maximum":{"kind":"rational","numerator":"5","denominator":"1"}"#,
    );
    let selection = resealed(&fixture.selection, &bound);
    let error = fixture
        .admit_selected(&selection, &bound)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::WrongNumericKind)
        ),
        "bound: got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_a_type_bound_that_disagrees_with_the_selected_model() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    // Widening a declared representation bound keeps every value in the package
    // inside it, so only agreement with the independently selected model bytes
    // can refuse this. The offer cannot relax its own type domains.
    let changed = respell_within(
        &bytes,
        INTEGER_BOUND,
        r#""maximum":{"kind":"integer","decimal":"5"}"#,
        r#""maximum":{"kind":"integer","decimal":"9223372036854775807"}"#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Invalid(Invalid::Type)),
        "got {error:?}"
    );
}

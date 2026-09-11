// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001/Task-009: actual A-owned four-unit compiled-protocol handoff.

use std::{fs, path::Path};

use ix_trace_rs::trace;
use quire_contract_ir as ir;
use quire_protocol::{
    admit_and_link,
    digest::producer::{CrossDomainDigest, DigestDomain, DigestSlot, OfferedDomain},
    inventory::{
        axis::SelectionAxis,
        map::{admit_discriminating, AdmissionRefusal},
        SelectedInventory,
    },
    wire, AdmittedModel, AdmittedPackage, ByteDigest, Expected, ExpectedDeclaration,
    ExpectedForeignSource, ExpectedSource, Invalid, Limits, SuppliedDependency,
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

/// Tracing: IT-001, Task-009.
#[test]
#[ignore = "requires QUIRE_PROTOCOL_A_FIXTURE generated by A's stripped release producer"]
#[trace("IT-001", "Task-009")]
fn it_001_accepts_and_links_the_actual_four_unit_handoff() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = fs::read(root.join("compiled-protocol.json")).expect("A compiled bytes");
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("positive admission");
    assert_eq!(admitted.package().sources.len(), 4);
    assert_eq!(admitted.package().declarations.len(), 6);
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
    let changed = replace_once(
        &bytes,
        b"native_protocol_handoff",
        b"native_protocol_handovv",
    );
    let selection = resealed(&fixture.selection, &changed);
    // The producer implementation label is its own axis: A collapses it into
    // the shared `Invalid::Selection`, so this asserts B's own producer-identity
    // code, which no other landed mutation can produce.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection(SelectionAxis::ProducerIdentity),
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
    let changed = replace_once(&bytes, b"\"decimal\":\"5\"", b"\"decimal\":\"x\"");
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
        AdmissionRefusal::Selection(SelectionAxis::BaselineSelectionState),
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
        AdmissionRefusal::Selection(SelectionAxis::BaselineDigest),
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
            selected: DigestDomain::ProducerArtifactBytes,
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
            selected: DigestDomain::ProducerArtifactBytes,
            offered: OfferedDomain::Identified(DigestDomain::ProtocolResultJcs),
            value: result_domain,
        }),
        "got {error:?}"
    );
}

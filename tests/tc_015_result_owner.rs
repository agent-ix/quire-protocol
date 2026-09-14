// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-006/TC-015: complete canonical result owner and Contract-IR mapping.

use ix_trace_rs::trace;
use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits as ObservationLimits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, ProducerSelection,
    QualifiedObservation, ResourceLimits, ScopeKind, ScopeSelection, Subject, SubjectKind,
    ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT, PRODUCER_INTERFACE_VERSION,
};
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure, ScopeState};
use quire_protocol::repro::{CorpusIdentity, ReproductionInputs, Seed, ToolchainIdentity};
use quire_protocol::result::{
    self, contract_ir, Activation, Adequacy, Assumption, AssumptionState, AxisInputs, Claim,
    ClaimKind, ClaimStrength, CorrectionInput, CorrectionRelation, DecisionSupport,
    DependenceRelation, ErrorCode, GlobalConformanceInput, GlobalPremise, Input, Limitation,
    Limits, Participation, PremiseKind, ProtocolAdequacy, ProvenanceRecord, Settlement,
    SettlementBasis, SupportRef, Truth,
};
use quire_protocol::{v2, wire as w, ByteDigest};
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};
use serde_json::Value;

#[path = "support/v2_handoff/mod.rs"]
#[allow(dead_code, unused_imports)]
mod handoff;

struct Fixture {
    predicate: checked_predicate::ValidatedCheckedPredicate,
    temporal: temporal_subject::ValidatedTemporalSubject,
    decision: ScopeViews,
    surrounding: ScopeViews,
}

struct ScopeViews {
    progress_open: authority::progress::View,
    progress_closed: authority::progress::View,
    closure_open: authority::closure::View,
    closure_closed: authority::closure::View,
    completeness_complete: authority::completeness::View,
    completeness_incomplete: authority::completeness::View,
    completeness_contradicted: authority::completeness::View,
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}
fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn fixture() -> Fixture {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let admitted = quire_protocol::admit_and_link_v2(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package");
    let (predicate_declaration, predicate_root) = checked_leaf(&admitted);
    let predicate_selection =
        checked_predicate::ClauseSelection::new(predicate_declaration, predicate_root);
    let predicate_document = checked_predicate::derive(
        &admitted,
        predicate_selection.clone(),
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("checked predicate document");
    let predicate = checked_predicate::read(
        predicate_document.bytes(),
        &admitted,
        predicate_selection,
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("checked predicate view");
    let temporal_declaration = admitted
        .inherited()
        .declarations
        .iter()
        .position(|declaration| matches!(declaration.body, w::Body::Temporal { .. }))
        .and_then(|index| u32::try_from(index).ok())
        .expect("temporal declaration");
    let temporal_selection = temporal_subject::DeclarationSelection::new(temporal_declaration);
    let temporal_document = temporal_subject::derive(
        &admitted,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject document");
    let temporal = temporal_subject::read(
        temporal_document.bytes(),
        &admitted,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject view");
    Fixture {
        predicate,
        temporal,
        decision: scope_views("decision"),
        surrounding: scope_views("surrounding"),
    }
}

fn checked_leaf(package: &v2::AdmittedPackage) -> (u32, w::Handle) {
    package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            let root = match &declaration.body {
                w::Body::Predicate { root, .. } | w::Body::State { root, .. } => root.clone(),
                w::Body::Temporal { .. } => declaration.temporal.iter().find_map(|node| {
                    if let w::TemporalOperation::Holds { value } = &node.operation {
                        Some(value.clone())
                    } else {
                        None
                    }
                })?,
                w::Body::Protocol {
                    controls, finish, ..
                } => controls
                    .iter()
                    .find_map(|control| {
                        if let w::ControlOperation::Check { value, .. } = &control.operation {
                            Some(value.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| finish.constraint.clone()),
            };
            Some((u32::try_from(index).ok()?, root))
        })
        .expect("checked Boolean leaf")
}

fn scope_views(tag: &str) -> ScopeViews {
    let qualified = qualified(tag);
    let owner = AuthoritySelection {
        definition_identity: id(format!("definition:{tag}")),
        definition_revision: id("1"),
        definition_digest: digest(9),
    };
    let subject = SubjectSelection {
        scope_identity: id(format!("window:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let progress_open = progress_view(context, OpenClosed::Open, tag);
    let progress_closed = progress_view(context, OpenClosed::Closed, tag);
    let closure_open = closure_view(context, OpenClosed::Open, tag);
    let closure_closed = closure_view(context, OpenClosed::Closed, tag);
    let completeness_complete = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Available,
        true,
        tag,
    );
    let completeness_incomplete = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Incomplete,
        false,
        tag,
    );
    let completeness_contradicted = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Contradicted,
        true,
        tag,
    );
    ScopeViews {
        progress_open,
        progress_closed,
        closure_open,
        closure_closed,
        completeness_complete,
        completeness_incomplete,
        completeness_contradicted,
    }
}

fn qualified(tag: &str) -> Box<QualifiedObservation> {
    let subject = Subject {
        kind: SubjectKind::Order,
        identity: id(format!("order:{tag}")),
    };
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}")),
            revision: id("1"),
            digest: digest(1),
        },
        producer: ProducerSelection {
            interface_version: PRODUCER_INTERFACE_VERSION.to_owned(),
            document_identity: id(format!("producer:{tag}")),
            document_digest: digest(2),
            model_identity: id(format!("model:{tag}")),
            configuration_identity: id(format!("configuration:{tag}")),
            configuration_digest: digest(3),
        },
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            subject_kind: SubjectKind::Order,
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id(format!("membership:{tag}")),
            membership_digest: digest(4),
            membership_document: vec![],
            required_member_identities: vec![id(format!("member:{tag}"))],
            observation_sources: vec![id(format!("source:{tag}"))],
            completeness_dependencies: vec![id(format!("completeness:{tag}"))],
            progress_dependencies: vec![id(format!("progress:{tag}"))],
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Window {
                window_identity: id(format!("window:{tag}")),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id(format!("member:{tag}")),
                record_identity: id("unsealed-record"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            subject,
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            value: ValueState::Present {
                value_type: id("text"),
                canonical_value: tag.to_owned(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            causal_relationship_identity: None,
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 1,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let record_identity = authority::observation::record_identity(
        &request.records[0],
        ObservationLimits::owner_max(),
    )
    .expect("record identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .expect("population identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("qualification failed: {other:?}"),
    }
}

fn boundary(state: OpenClosed) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: if state == OpenClosed::Closed { 30 } else { 20 },
    }
}
fn cutoff(tag: &str) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id(format!("clock:{tag}")),
        id("1"),
        40,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}
fn progress_selection(state: OpenClosed, tag: &str) -> authority::progress::Selection {
    authority::progress::Selection::new(
        authority::clock::Selection::new(id(format!("clock:{tag}")), id("1")),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
        id(format!("trigger:{tag}")),
        cutoff(tag),
        id(format!("restoration:{tag}")),
    )
}
fn progress_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::progress::View {
    let selection = progress_selection(state, tag);
    let document = authority::progress::derive(context, &selection, ObservationLimits::owner_max())
        .expect("progress document");
    authority::progress::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("progress view")
}
fn closure_selection(state: OpenClosed, tag: &str) -> authority::closure::Selection {
    authority::closure::Selection::new(
        id(format!("clock:{tag}")),
        id("1"),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
    )
}
fn closure_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::closure::View {
    let selection = closure_selection(state, tag);
    let document = authority::closure::derive(context, &selection, ObservationLimits::owner_max())
        .expect("closure document");
    authority::closure::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("closure view")
}
fn completeness_view(
    context: Context<'_>,
    qualified: &QualifiedObservation,
    status: authority::completeness::FactStatus,
    observed: bool,
    tag: &str,
) -> authority::completeness::View {
    let selection = authority::completeness::Selection::new(
        id(format!("boundary:{tag}")),
        vec![authority::completeness::Fact::new(
            id(format!("member:{tag}")),
            observed.then(|| qualified.records()[0].identity.clone()),
            status,
        )],
    );
    let document =
        authority::completeness::derive(context, &selection, ObservationLimits::owner_max())
            .expect("completeness document");
    authority::completeness::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("completeness view")
}

fn global_premises() -> Vec<GlobalPremise> {
    vec![
        GlobalPremise {
            kind: PremiseKind::Execution,
            identity: "premise:execution".into(),
            scope_identity: "scope:global".into(),
            authority_identity: "authority:global".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Branch,
            identity: "premise:branch".into(),
            scope_identity: "scope:global".into(),
            authority_identity: "authority:global".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Workflow,
            identity: "premise:workflow".into(),
            scope_identity: "scope:global".into(),
            authority_identity: "authority:global".into(),
        },
    ]
}

fn reproduction(
    seed: u64,
    toolchain_identity: &[u8],
    corpus_identity: &[u8],
    display: &str,
) -> quire_protocol::repro::ReproductionRecord {
    ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(seed))
        .with_toolchain(ToolchainIdentity::new(toolchain_identity, display))
        .with_corpus(CorpusIdentity::new(corpus_identity, display))
        .record()
        .expect("complete reproduction identity")
}

fn base_input<'a>(fixture: &'a Fixture) -> Input<'a> {
    let observation_identity = fixture
        .decision
        .completeness_complete
        .payload()
        .facts()
        .next()
        .and_then(|fact| fact.observation_identity)
        .expect("available fact")
        .to_owned();
    let premises = global_premises();
    Input {
        checked_predicate: &fixture.predicate,
        temporal_subject: &fixture.temporal,
        axes: AxisInputs {
            decision_scope_progress: &fixture.decision.progress_closed,
            decision_scope_closure: &fixture.decision.closure_closed,
            surrounding_execution_progress: &fixture.surrounding.progress_closed,
            surrounding_execution_closure: &fixture.surrounding.closure_closed,
        },
        completeness: &fixture.decision.completeness_complete,
        execution: AssessmentExecution::Completed,
        truth: Truth::Satisfied,
        settlement: Settlement {
            basis: SettlementBasis::ClosedScope,
            supporting_fact_identities: vec![observation_identity.clone()],
            supporting_progress_identities: vec![fixture
                .decision
                .progress_closed
                .identity()
                .as_str()
                .to_owned()],
        },
        activation: Activation::Triggered,
        participation: Participation::Known {
            required_role_instance_refs: vec!["role:buyer".into()],
            participating_role_instance_refs: vec!["role:buyer".into()],
        },
        assumptions: vec![Assumption {
            identity: "assumption:one".into(),
            state: AssumptionState::Met,
        }],
        observation_adequacy: Adequacy::Adequate,
        protocol_adequacy: ProtocolAdequacy::Demonstrated,
        claim: Claim {
            kind: ClaimKind::GlobalConformance,
            strength: Some(ClaimStrength::Proven),
            admitted_fragment_identity: Some("fragment:finite".into()),
            backend_identity: Some("backend:model-checker".into()),
        },
        decision_support: vec![DecisionSupport {
            oracle_identity: "oracle:one".into(),
            observation_identity,
            relation: DependenceRelation::Independent,
            predecessor: None,
            dependence_group: None,
        }],
        global_conformance: GlobalConformanceInput {
            state: GlobalConformanceClosure::Closed,
            required_premises: premises.clone(),
            premises,
        },
        limitations: vec![Limitation {
            code: "bound:finite".into(),
            affected_identity: "claim:one".into(),
        }],
        provenance: ProvenanceRecord {
            source_identity: "source:one".into(),
            profile_identity: "profile:one".into(),
            model_identity: "model:one".into(),
            protocol_identity: "protocol:one".into(),
            request_identity: "request:one".into(),
            binding_identity: "binding:one".into(),
            input_identity: "input:one".into(),
            backend_identity: "backend:model-checker".into(),
            producer_identity: "producer:one".into(),
            reproduction: reproduction(
                0x5eed_0000_0000_0001,
                b"rustc-1.98.0-x86_64-unknown-linux-gnu",
                b"corpus:9f2c1ad4",
                "local reproduction inputs",
            ),
        },
        correction: None,
    }
}

fn validated<'a>(input: Input<'a>) -> result::ValidatedResult {
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("strict result")
}

fn assert_result_mutation(label: &str, baseline: &result::Document, input: Input<'_>) {
    match result::produce(input, Limits::owner_max()) {
        Ok(mutant) => {
            assert_ne!(
                mutant.result_id(),
                baseline.result_id(),
                "{label} did not affect the result identity"
            );
            assert_ne!(
                mutant.bytes(),
                baseline.bytes(),
                "{label} did not affect the canonical result bytes"
            );
        }
        Err(refusal) => {
            assert!(
                !refusal.code().as_str().is_empty(),
                "{label} returned an untyped refusal"
            );
            assert_eq!(
                refusal.envelope().partial_output(),
                quire_protocol::refusal::PartialOutput::NoneEmitted,
                "{label} exposed partial output beside its refusal"
            );
        }
    }
}

#[test]
#[trace("TC-008", "NFR-001-AC-1", "FR-017-AC-5", "NFR-008-AC-7", "Task-008")]
fn tc_008_result_bytes_ignore_allocation_layout_and_localized_rendering() {
    let first_fixture = fixture();
    let first_input = base_input(&first_fixture);
    let first = result::produce(first_input, Limits::owner_max()).expect("first result");

    // Perturb allocator history and rebuild every owner view independently.
    let allocation_noise: Vec<Vec<u8>> = (0_u16..257)
        .map(|length| vec![0xa5; usize::from(length)])
        .collect();
    assert_eq!(allocation_noise.len(), 257);
    let second_fixture = fixture();
    let mut second_input = base_input(&second_fixture);
    second_input.provenance.reproduction = reproduction(
        0x5eed_0000_0000_0001,
        b"rustc-1.98.0-x86_64-unknown-linux-gnu",
        b"corpus:9f2c1ad4",
        "Yerelleştirilmiş çalıştırma",
    );
    let second = result::produce(second_input, Limits::owner_max()).expect("second result");

    assert_eq!(first.result_id(), second.result_id());
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.bytes(), second.bytes());

    let json: Value = serde_json::from_slice(first.bytes()).expect("owner JSON");
    let reproduction = &json["provenance"]["reproduction"];
    assert_eq!(reproduction["seed"], "6840123409045651457");
    assert_eq!(
        reproduction["toolchainIdentity"],
        "72757374632d312e39382e302d7838365f36342d756e6b6e6f776e2d6c696e75782d676e75"
    );
    assert_eq!(
        reproduction["corpusIdentity"],
        "636f727075733a3966326331616434"
    );
}

#[test]
#[trace("TC-008", "NFR-001-AC-1", "Task-008")]
fn tc_008_process_locale_does_not_change_canonical_result() {
    const CHILD: &str = "QUIRE_TC008_LOCALE_CHILD";
    const MARKER: &str = "TC008-CANONICAL:";
    if std::env::var_os(CHILD).is_some() {
        let owner = fixture();
        let document =
            result::produce(base_input(&owner), Limits::owner_max()).expect("locale result");
        println!("{MARKER}{}:{}", document.result_id(), document.digest());
        return;
    }

    fn identity_under(locale: &str, child: &str, marker: &str) -> String {
        let output = std::process::Command::new(
            std::env::current_exe().expect("current integration-test executable"),
        )
        .args([
            "--exact",
            "tc_008_process_locale_does_not_change_canonical_result",
            "--nocapture",
        ])
        .env(child, "1")
        .env("LC_ALL", locale)
        .output()
        .expect("locale-isolated child process");
        assert!(
            output.status.success(),
            "locale child {locale} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("Rust test output is UTF-8")
            .lines()
            .find_map(|line| line.strip_prefix(marker).map(str::to_owned))
            .expect("locale child emitted the canonical marker")
    }

    assert_eq!(
        identity_under("C", CHILD, MARKER),
        identity_under("tr_TR.UTF-8", CHILD, MARKER)
    );
}

#[test]
#[trace("TC-008", "NFR-001-AC-2", "NFR-001-AC-3", "FR-031-AC-5", "Task-008")]
fn tc_008_every_named_semantic_identity_is_discriminated() {
    let owner = fixture();
    let baseline = result::produce(base_input(&owner), Limits::owner_max()).expect("baseline");

    macro_rules! mutate {
        ($label:literal, |$input:ident| $mutation:block) => {{
            let mut input = base_input(&owner);
            let $input = &mut input;
            $mutation
            assert_result_mutation($label, &baseline, input);
        }};
    }

    mutate!("source", |input| {
        input.provenance.source_identity = "source:two".into();
    });
    mutate!("accepted source-baseline/profile", |input| {
        input.provenance.profile_identity = "profile:two".into();
    });
    mutate!("model", |input| {
        input.provenance.model_identity = "model:two".into();
    });
    mutate!("compiled protocol", |input| {
        input.provenance.protocol_identity = "protocol:two".into();
    });
    mutate!("request", |input| {
        input.provenance.request_identity = "request:two".into();
    });
    mutate!("binding", |input| {
        input.provenance.binding_identity = "binding:two".into();
    });
    mutate!("input", |input| {
        input.provenance.input_identity = "input:two".into();
    });
    mutate!("producer", |input| {
        input.provenance.producer_identity = "producer:two".into();
    });
    mutate!("backend", |input| {
        input.provenance.backend_identity = "backend:two".into();
        input.claim.backend_identity = Some("backend:two".into());
    });
    mutate!("implementation", |input| {
        input.provenance.reproduction = reproduction(
            0x5eed_0000_0000_0001,
            b"rustc-1.99.0-x86_64-unknown-linux-gnu",
            b"corpus:9f2c1ad4",
            "implementation mutation",
        );
    });
    mutate!("observation", |input| {
        input.decision_support[0].observation_identity = "observation:substituted".into();
    });
    mutate!("decision-scope progress", |input| {
        input.axes.decision_scope_progress = &owner.decision.progress_open;
    });
    mutate!("decision-scope closure", |input| {
        input.axes.decision_scope_closure = &owner.decision.closure_open;
    });
    mutate!("surrounding-execution progress", |input| {
        input.axes.surrounding_execution_progress = &owner.surrounding.progress_open;
    });
    mutate!("surrounding-execution closure", |input| {
        input.axes.surrounding_execution_closure = &owner.surrounding.closure_open;
    });
    mutate!("complete-global closure", |input| {
        input.global_conformance.state = GlobalConformanceClosure::Incomplete;
    });

    // Source identity bytes are exact: a presentation-changing trim would be
    // a semantic rewrite, not an equivalent presentation.
    mutate!("identity-changing source presentation", |input| {
        input.provenance.source_identity = " source:one ".into();
    });

    // The execution/branch/workflow list is an exact authored sequence. It is
    // not presentation-sorted into acceptance.
    mutate!("authored global-premise order", |input| {
        input.global_conformance.premises.swap(0, 1);
    });
}

#[test]
#[trace("TC-008", "TC-009", "NFR-002-AC-1", "Task-008")]
fn tc_009_reproduction_provenance_obeys_result_string_bounds() {
    let owner = fixture();
    let mut input = base_input(&owner);
    input.provenance.reproduction = reproduction(1, b"abc", b"d", "bounded identities");
    let limits = Limits {
        max_string_bytes: 5,
        ..Limits::owner_max()
    };
    let refusal = result::produce(input, limits).expect_err("six encoded bytes exceed five");
    assert_eq!(refusal.code(), ErrorCode::ResourceIncomplete);
    assert_eq!(refusal.usage().string_bytes, 6);
    assert_eq!(
        refusal.envelope().partial_output(),
        quire_protocol::refusal::PartialOutput::NoneEmitted
    );
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-1",
    "FR-031-AC-1",
    "FR-031-AC-2",
    "FR-031-AC-3",
    "FR-042-AC-1",
    "FR-042-AC-4",
    "FR-042-AC-5"
)]
fn tc_015_every_result_combination_maps_without_boolean_coercion() {
    let fixture = fixture();
    let executions = AssessmentExecution::ALL;
    let truths = [
        Truth::Satisfied,
        Truth::Violated,
        Truth::Pending,
        Truth::Unavailable,
    ];
    let settlements = [
        SettlementBasis::ClosedScope,
        SettlementBasis::DecisiveWitness,
        SettlementBasis::DecisiveCounterexample,
        SettlementBasis::Unsettled,
        SettlementBasis::Unavailable,
    ];
    let activations = [
        Activation::Triggered,
        Activation::Inactive,
        Activation::ActivationUnknown,
    ];
    let scope_states = [OpenClosed::Open, OpenClosed::Closed];
    let mut admitted = 0usize;
    let mut refused = 0usize;

    for execution in executions {
        for truth in truths {
            for settlement in settlements {
                for activation in activations {
                    for progress in scope_states {
                        for closure in scope_states {
                            for has_support in [false, true] {
                                let expected = expected_result_combination(
                                    execution,
                                    truth,
                                    settlement,
                                    activation,
                                    progress,
                                    closure,
                                    has_support,
                                );
                                let mut input = base_input(&fixture);
                                input.execution = execution;
                                input.truth = truth;
                                input.settlement.basis = settlement;
                                input.activation = activation;
                                input.axes.decision_scope_progress = match progress {
                                    OpenClosed::Open => &fixture.decision.progress_open,
                                    OpenClosed::Closed => &fixture.decision.progress_closed,
                                };
                                input.axes.decision_scope_closure = match closure {
                                    OpenClosed::Open => &fixture.decision.closure_open,
                                    OpenClosed::Closed => &fixture.decision.closure_closed,
                                };
                                input.settlement.supporting_progress_identities = vec![input
                                    .axes
                                    .decision_scope_progress
                                    .identity()
                                    .as_str()
                                    .to_owned()];
                                if !has_support {
                                    input.decision_support.clear();
                                    input.settlement.supporting_fact_identities.clear();
                                }

                                match result::produce(input.clone(), Limits::owner_max()) {
                                    Ok(document) => {
                                        assert!(expected, "unexpectedly admitted {input:?}");
                                        let source = result::read(
                                            document.bytes(),
                                            result::Expected {
                                                input,
                                                enclosing_digest: document.digest(),
                                            },
                                            Limits::owner_max(),
                                        )
                                        .expect("every admitted row strict-reads");
                                        let mapped = contract_ir::map(
                                            &source,
                                            contract_ir::MappingSelection::current(),
                                            Limits::owner_max(),
                                        )
                                        .expect("every admitted row maps");
                                        let expected_value = match (execution, truth) {
                                            (AssessmentExecution::Completed, Truth::Satisfied) => {
                                                Some(true)
                                            }
                                            (AssessmentExecution::Completed, Truth::Violated) => {
                                                Some(false)
                                            }
                                            _ => None,
                                        };
                                        assert_eq!(mapped.value(), expected_value);
                                        match (mapped.non_value(), expected_value) {
                                            (Some(non_value), None) => {
                                                assert_eq!(non_value.execution(), execution);
                                                assert_eq!(non_value.truth(), truth);
                                                assert_eq!(non_value.settlement(), settlement);
                                                assert_eq!(
                                                    non_value.completeness(),
                                                    source.completeness_state()
                                                );
                                            }
                                            (None, Some(_)) => {}
                                            _ => panic!("value/non-value discriminator diverged"),
                                        }
                                        let reread = contract_ir::read(
                                            mapped.bytes(),
                                            &source,
                                            contract_ir::MappingSelection::current(),
                                            Limits::owner_max(),
                                        )
                                        .expect("every admitted mapping strict-reads");
                                        assert_eq!(reread.bytes(), mapped.bytes());
                                        admitted += 1;
                                    }
                                    Err(error) => {
                                        assert!(!expected, "unexpected refusal: {error:?}");
                                        assert_eq!(error.code(), ErrorCode::InvalidCombination);
                                        refused += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(admitted, 126);
    assert_eq!(refused, 2_274);
}

fn expected_result_combination(
    execution: AssessmentExecution,
    truth: Truth,
    settlement: SettlementBasis,
    activation: Activation,
    progress: OpenClosed,
    closure: OpenClosed,
    has_support: bool,
) -> bool {
    use AssessmentExecution as E;
    use SettlementBasis as S;
    use Truth as T;

    match (execution, truth, settlement, activation) {
        (E::Completed, T::Satisfied | T::Violated, S::ClosedScope, Activation::Triggered) => {
            progress == OpenClosed::Closed && closure == OpenClosed::Closed && has_support
        }
        (E::Completed, T::Satisfied, S::DecisiveWitness, Activation::Triggered)
        | (E::Completed, T::Violated, S::DecisiveCounterexample, Activation::Triggered) => {
            closure == OpenClosed::Open && has_support
        }
        (E::Completed, T::Pending, S::Unsettled, Activation::Triggered) => {
            progress == OpenClosed::Open
        }
        (
            E::Completed,
            T::Unavailable,
            S::Unavailable,
            Activation::Inactive | Activation::ActivationUnknown,
        ) => true,
        (E::Completed, T::Unavailable, S::Unavailable, Activation::Triggered) => !has_support,
        (
            E::Refused | E::Unsupported | E::ResourceIncomplete | E::Failed,
            T::Unavailable,
            S::Unavailable,
            _,
        ) => true,
        _ => false,
    }
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-1",
    "FR-032-AC-3",
    "FR-042-AC-1",
    "FR-042-AC-3",
    "FR-042-AC-5"
)]
fn tc_015_validated_and_mapped_views_expose_every_contract_axis_without_json_parsing() {
    let fixture = fixture();
    let source = validated(base_input(&fixture));

    assert_eq!(source.execution(), AssessmentExecution::Completed);
    assert_eq!(source.truth(), Truth::Satisfied);
    assert_eq!(source.activation(), Activation::Triggered);
    assert_eq!(source.observation_adequacy(), Adequacy::Adequate);
    assert_eq!(source.protocol_adequacy(), ProtocolAdequacy::Demonstrated);
    assert_eq!(source.claim().kind, ClaimKind::GlobalConformance);
    assert_eq!(source.assumptions().len(), 1);
    assert!(matches!(
        source.participation(),
        Participation::Known { .. }
    ));
    assert_eq!(
        source.global_conformance_state(),
        GlobalConformanceClosure::Closed
    );
    assert_eq!(source.global_premises().len(), 3);
    assert_eq!(source.limitations().len(), 1);
    assert_eq!(source.provenance().source_identity, "source:one");
    assert_eq!(source.decision_scope_progress().state(), ScopeState::Closed);
    assert_eq!(source.decision_scope_closure().state(), ScopeState::Closed);
    assert_eq!(
        source.surrounding_execution_progress().state(),
        ScopeState::Closed
    );
    assert_eq!(
        source.surrounding_execution_closure().state(),
        ScopeState::Closed
    );
    assert_eq!(
        source.completeness().state(),
        authority::completeness::State::Complete
    );
    assert_eq!(source.completeness().fact_identities().len(), 1);
    assert!(source.correction().is_none());

    let mapped = contract_ir::map(
        &source,
        contract_ir::MappingSelection::current(),
        Limits::owner_max(),
    )
    .expect("complete typed mapping view");
    assert_eq!(mapped.execution(), source.execution());
    assert_eq!(mapped.truth(), source.truth());
    assert_eq!(
        mapped.native_subject_identity(),
        source.native_subject_identity()
    );
    assert_eq!(
        mapped.native_correspondence_identity(),
        source.native_correspondence_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().document_identity(),
        source.decision_scope_progress().document_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().contract(),
        source.decision_scope_progress().contract()
    );
    assert_eq!(
        mapped.decision_scope_progress().schema_sha256(),
        source.decision_scope_progress().schema_sha256()
    );
    assert_eq!(
        mapped.decision_scope_progress().document_digest(),
        source.decision_scope_progress().document_digest()
    );
    assert_eq!(
        mapped.decision_scope_progress().authority_identity(),
        source.decision_scope_progress().authority_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().authority_revision(),
        source.decision_scope_progress().authority_revision()
    );
    assert_eq!(
        mapped.decision_scope_progress().authority_digest(),
        source.decision_scope_progress().authority_digest()
    );
    assert_eq!(
        mapped.decision_scope_progress().population_identity(),
        source.decision_scope_progress().population_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().scope_identity(),
        source.decision_scope_progress().scope_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().state(),
        source.decision_scope_progress().state()
    );
    assert_ne!(
        source.decision_scope_progress().document_digest(),
        source.decision_scope_progress().authority_digest(),
        "the fixture must kill a swapped digest delegation"
    );
    assert_eq!(
        mapped.decision_scope_closure().scope_identity(),
        source.decision_scope_closure().scope_identity()
    );
    assert_eq!(
        mapped.surrounding_execution_progress().state(),
        ScopeState::Closed
    );
    assert_eq!(
        mapped.surrounding_execution_closure().state(),
        ScopeState::Closed
    );
    assert_eq!(
        mapped.completeness().document_identity(),
        source.completeness().document_identity()
    );
    assert_eq!(mapped.completeness().fact_identities().len(), 1);
    assert_eq!(mapped.settlement().basis, SettlementBasis::ClosedScope);
    assert_eq!(mapped.decision_support(), source.decision_support());
    assert!(mapped.correction().is_none());
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-3",
    "FR-032-AC-3",
    "FR-032-AC-4",
    "FR-032-AC-7",
    "FR-042-AC-3"
)]
fn tc_015_four_axes_and_fr_295_completeness_are_independent() {
    let fixture = fixture();
    let progress = [
        &fixture.decision.progress_open,
        &fixture.decision.progress_closed,
    ];
    let closure = [
        &fixture.decision.closure_open,
        &fixture.decision.closure_closed,
    ];
    let surrounding_progress = [
        &fixture.surrounding.progress_open,
        &fixture.surrounding.progress_closed,
    ];
    let surrounding_closure = [
        &fixture.surrounding.closure_open,
        &fixture.surrounding.closure_closed,
    ];
    let completeness = [
        &fixture.decision.completeness_complete,
        &fixture.decision.completeness_incomplete,
        &fixture.decision.completeness_contradicted,
    ];
    let mut combinations = 0usize;
    for decision_progress in progress {
        for decision_closure in closure {
            for execution_progress in surrounding_progress {
                for execution_closure in surrounding_closure {
                    for complete in completeness {
                        let mut input = base_input(&fixture);
                        input.axes = AxisInputs {
                            decision_scope_progress: decision_progress,
                            decision_scope_closure: decision_closure,
                            surrounding_execution_progress: execution_progress,
                            surrounding_execution_closure: execution_closure,
                        };
                        input.settlement.supporting_progress_identities =
                            vec![decision_progress.identity().as_str().to_owned()];
                        input.completeness = complete;
                        input.execution = AssessmentExecution::Unsupported;
                        input.truth = Truth::Unavailable;
                        input.settlement.basis = SettlementBasis::Unavailable;
                        input.claim = Claim {
                            kind: ClaimKind::Monitorability,
                            strength: None,
                            admitted_fragment_identity: None,
                            backend_identity: None,
                        };
                        input.global_conformance = GlobalConformanceInput {
                            state: GlobalConformanceClosure::NotRequired,
                            required_premises: vec![],
                            premises: vec![],
                        };
                        validated(input);
                        combinations += 1;
                    }
                }
            }
        }
    }
    assert_eq!(combinations, 48);
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-4",
    "FR-006-AC-5",
    "FR-006-AC-7",
    "FR-031-AC-5",
    "FR-033-AC-1",
    "FR-042-AC-2",
    "FR-042-AC-6"
)]
fn tc_015_strict_mutations_lineage_and_limits_fail_closed() {
    let fixture = fixture();
    let input = base_input(&fixture);
    let document = result::produce(input.clone(), Limits::owner_max()).expect("control result");
    let mut value: Value = serde_json::from_slice(document.bytes()).expect("result JSON");
    value["unexpected"] = Value::Bool(true);
    let unknown = quire_protocol::digest::jcs::canonicalize(&value).expect("canonical mutation");
    let error = result::read(
        unknown.as_bytes(),
        result::Expected {
            input: input.clone(),
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect_err("unknown field");
    assert_eq!(error.code(), ErrorCode::InvalidJson);

    let mut missing_reproduction: Value =
        serde_json::from_slice(document.bytes()).expect("result JSON");
    missing_reproduction["provenance"]
        .as_object_mut()
        .expect("provenance object")
        .remove("reproduction");
    let missing_reproduction = quire_protocol::digest::jcs::canonicalize(&missing_reproduction)
        .expect("canonical missing-field mutation");
    assert_eq!(
        result::read(
            missing_reproduction.as_bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: document.digest(),
            },
            Limits::owner_max(),
        )
        .expect_err("missing reproduction provenance")
        .code(),
        ErrorCode::InvalidJson
    );

    let mut changed_reproduction: Value =
        serde_json::from_slice(document.bytes()).expect("result JSON");
    changed_reproduction["provenance"]["reproduction"]["toolchainIdentity"] =
        Value::String("00".into());
    let changed_reproduction = quire_protocol::digest::jcs::canonicalize(&changed_reproduction)
        .expect("canonical reproduction mutation");
    let changed_digest = quire_protocol::digest::jcs::ResultDigest::of_result(
        &serde_json::from_str(&changed_reproduction).expect("mutated result value"),
    )
    .expect("mutated enclosing digest")
    .text();
    assert_eq!(
        result::read(
            changed_reproduction.as_bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: &changed_digest,
            },
            Limits::owner_max(),
        )
        .expect_err("changed reproduction identity")
        .code(),
        ErrorCode::IdentityMismatch
    );

    let mut trailing = document.bytes().to_vec();
    trailing.push(b'\n');
    assert_eq!(
        result::read(
            &trailing,
            result::Expected {
                input: input.clone(),
                enclosing_digest: document.digest()
            },
            Limits::owner_max()
        )
        .expect_err("trailing data")
        .code(),
        ErrorCode::NonCanonical
    );
    assert_eq!(
        result::read(
            document.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest:
                    "sha256-jcs:0000000000000000000000000000000000000000000000000000000000000000"
            },
            Limits::owner_max()
        )
        .expect_err("digest mismatch")
        .code(),
        ErrorCode::DigestMismatch
    );

    let mut missing_global = input.clone();
    missing_global.global_conformance.premises.pop();
    assert_eq!(
        result::produce(missing_global, Limits::owner_max())
            .expect_err("missing premise")
            .code(),
        ErrorCode::GlobalPremiseMismatch
    );

    let mut mismatched_settlement_fact = input.clone();
    mismatched_settlement_fact
        .settlement
        .supporting_fact_identities[0] = "observation:other".into();
    assert_eq!(
        result::produce(mismatched_settlement_fact, Limits::owner_max())
            .expect_err("settlement fact outside exact decision support")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut mismatched_progress = input.clone();
    mismatched_progress
        .settlement
        .supporting_progress_identities[0] = "progress:other".into();
    assert_eq!(
        result::produce(mismatched_progress, Limits::owner_max())
            .expect_err("settlement progress outside selected owner assertions")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut open_closed_scope = input.clone();
    open_closed_scope.axes.decision_scope_progress = &fixture.decision.progress_open;
    open_closed_scope.settlement.supporting_progress_identities = vec![fixture
        .decision
        .progress_open
        .identity()
        .as_str()
        .to_owned()];
    assert_eq!(
        result::produce(open_closed_scope, Limits::owner_max())
            .expect_err("closed-scope settlement on open progress")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut closed_decisive_witness = input.clone();
    closed_decisive_witness.settlement.basis = SettlementBasis::DecisiveWitness;
    assert_eq!(
        result::produce(closed_decisive_witness, Limits::owner_max())
            .expect_err("decisive witness on a closed decision scope")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut incomplete_closed_scope = input.clone();
    incomplete_closed_scope.completeness = &fixture.decision.completeness_incomplete;
    assert_eq!(
        result::produce(incomplete_closed_scope, Limits::owner_max())
            .expect_err("closed-scope settlement over incomplete input")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut missing_predecessor = input.clone();
    missing_predecessor.decision_support.push(DecisionSupport {
        oracle_identity: "oracle:derived".into(),
        observation_identity: "observation:derived".into(),
        relation: DependenceRelation::Derived,
        predecessor: Some(SupportRef {
            oracle_identity: "oracle:missing".into(),
            observation_identity: "observation:missing".into(),
        }),
        dependence_group: None,
    });
    assert_eq!(
        result::produce(missing_predecessor, Limits::owner_max())
            .expect_err("missing predecessor")
            .code(),
        ErrorCode::LineageInvalid
    );

    let mut limited = Limits::owner_max();
    limited.max_support_records = 0;
    assert_eq!(
        result::produce(input, limited)
            .expect_err("support limit")
            .code(),
        ErrorCode::ResourceIncomplete
    );
}

#[test]
#[trace("TC-009", "TC-015", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-4")]
fn tc_009_result_output_budget_precedes_canonical_allocation() {
    let fixture = fixture();
    let input = base_input(&fixture);
    let control = result::produce(input.clone(), Limits::owner_max()).expect("control result");
    let required = control.bytes().len();
    assert!(required > 1);
    let inventory = production_inventory().expect("production inventory");

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, required - 1)
        .expect("positive one-short grant");
    let mut short_resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut short_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.requested() == required
    ));
    assert_eq!(
        short_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        0
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, required)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    let OperationOutcome::Completed(exact_document) =
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut exact_resources)
    else {
        panic!("fresh exact output budget must complete");
    };
    assert_eq!(exact_document.bytes(), control.bytes());
    assert_eq!(
        exact_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        required
    );

    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one logical result allocation");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut allocation_resources
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut allocation_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let fresh_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("fresh logical result allocation");
    let mut fresh_resources = ResourceAccountant::new(fresh_allocation, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut fresh_resources),
        OperationOutcome::Completed(_)
    ));

    let usage = control.usage();
    let result_items = usage
        .claims
        .checked_add(usage.premises)
        .and_then(|value| value.checked_add(usage.support_records))
        .and_then(|value| value.checked_add(usage.limitations))
        .expect("bounded result items");
    let item_short = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items - 1)
        .expect("positive one-short result-item grant");
    let mut item_resources = ResourceAccountant::new(item_short, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut item_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        0
    );

    let item_exact = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items)
        .expect("positive exact result-item grant");
    let mut item_resources = ResourceAccountant::new(item_exact, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut item_resources),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        result_items
    );

    let one_lineage = Budget::nothing_granted()
        .grant(BudgetDimension::Lineage, usage.lineage_nodes)
        .expect("positive one-result lineage grant");
    let mut lineage_resources = ResourceAccountant::new(one_lineage, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut lineage_resources),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut lineage_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Lineage
                && value.consumed() == usage.lineage_nodes
                && value.requested() == usage.lineage_nodes
    ));

    let owner_support_short = Limits {
        max_support_records: usage.support_records - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), owner_support_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == usage.support_records - 1
                && value.requested() == usage.support_records
    ));

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, required - 1)
        .expect("positive one-short grant");
    let mut short_resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut short_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.requested() == required
    ));
    assert_eq!(
        short_resources.usage().consumed(BudgetDimension::InputByte),
        0
    );

    let retained_string = control.usage().string_bytes;
    let total_input = required
        .checked_add(retained_string)
        .expect("bounded test input");
    let retained_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input - 1)
        .expect("positive retained-string one-short grant");
    let mut retained_short_resources = ResourceAccountant::new(retained_short, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut retained_short_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == required
                && value.requested() == retained_string
    ));

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut exact_resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let one_read_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one logical result-read allocation");
    let mut read_resources = ResourceAccountant::new(one_read_allocation, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert_eq!(
        exact_resources.usage().consumed(BudgetDimension::InputByte),
        total_input
    );

    let visited = control.usage().visited_fields;
    let depth = control.usage().depth;
    let operations = visited.checked_add(depth).expect("bounded test document");
    assert!(operations > 1);
    let short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operations - 1)
        .expect("positive one-short grant");
    let mut short_resources = ResourceAccountant::new(short, &inventory);
    let short_outcome = result::read_bounded(
        control.bytes(),
        result::Expected {
            input: input.clone(),
            enclosing_digest: control.digest(),
        },
        Limits::owner_max(),
        &mut short_resources,
    );
    let consumed = short_resources.usage().consumed(BudgetDimension::Operation);
    assert!(
        matches!(
            short_outcome,
            OperationOutcome::ResourceIncomplete(value)
                if value.dimension() == BudgetDimension::Operation
                    && value.consumed() == depth
                    && value.requested() == visited
        ),
        "unexpected outcome; depth={depth}; visited={visited}; consumed={consumed}"
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operations)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut exact_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        exact_resources.usage().consumed(BudgetDimension::Operation),
        operations
    );

    let owner_input_short = Limits {
        max_input_bytes: required - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_input_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.granted() == required - 1
                && value.requested() == required
    ));

    let owner_depth_short = Limits {
        max_depth: depth - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_depth_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.granted() == depth - 1
                && value.requested() == depth
    ));

    let owner_string_short = Limits {
        max_string_bytes: retained_string - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_string_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.granted() == retained_string - 1
                && value.requested() == retained_string
    ));

    let owner_field_short = Limits {
        max_visited_fields: visited - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_field_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.granted() == visited - 1
                && value.consumed() == 0
                && value.requested() == visited
    ));

    let owner_short = Limits {
        max_output_bytes: required - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::produce_bounded(input, owner_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.granted() == required - 1
                && value.requested() == required
    ));
}

#[test]
#[trace("TC-009", "TC-015", "NFR-002-AC-1", "FR-016-AC-2", "FR-042-AC-6")]
fn tc_009_contract_ir_mapping_charges_derived_artifacts_and_output() {
    let fixture = fixture();
    let source = validated(base_input(&fixture));
    let selection = contract_ir::MappingSelection::current();
    let control = contract_ir::map(&source, selection, Limits::owner_max()).expect("control map");
    let required = control.bytes().len();
    let inventory = production_inventory().expect("production inventory");

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, required - 1)
        .expect("positive one-short output grant");
    let mut resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.requested() == required
    ));
    assert_eq!(resources.usage().consumed(BudgetDimension::OutputByte), 0);

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, required)
        .expect("positive exact output grant");
    let mut resources = ResourceAccountant::new(exact, &inventory);
    let OperationOutcome::Completed(mapped) =
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources)
    else {
        panic!("exact output budget must complete mapping");
    };
    assert_eq!(mapped.bytes(), control.bytes());

    let result_items = control.usage().claims + control.usage().support_records;
    let one_short_items = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items - 1)
        .expect("positive one-short mapping result-item grant");
    let mut item_resources = ResourceAccountant::new(one_short_items, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut item_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        0,
        "the mapping result-item preflight is transactional"
    );

    let exact_items = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items)
        .expect("exact mapping result-item grant");
    let mut item_resources = ResourceAccountant::new(exact_items, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut item_resources,),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        result_items
    );

    let support_owner_short = Limits {
        max_support_records: 0,
        ..Limits::owner_max()
    };
    let mut item_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, support_owner_short, &mut item_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == 0
                && value.requested() == 1
    ));

    let one_artifact = Budget::nothing_granted()
        .grant(BudgetDimension::DerivedArtifact, 1)
        .expect("positive one-artifact grant");
    let mut resources = ResourceAccountant::new(one_artifact, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::DerivedArtifact
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let mut fresh_resources = ResourceAccountant::new(
        Budget::nothing_granted()
            .grant(BudgetDimension::DerivedArtifact, 1)
            .expect("positive fresh grant"),
        &inventory,
    );
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut fresh_resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let one_map_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one logical mapping allocation");
    let mut allocation_resources = ResourceAccountant::new(one_map_allocation, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut allocation_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut allocation_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let one_read_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one mapping-read allocation");
    let mut read_resources = ResourceAccountant::new(one_read_allocation, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let two_read_allocations = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 2)
        .expect("decode plus rederivation allocations");
    let mut read_resources = ResourceAccountant::new(two_read_allocations, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let owner_short = Limits {
        max_output_bytes: required - 1,
        ..Limits::owner_max()
    };
    let mut resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, owner_short, &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.granted() == required - 1
                && value.requested() == required
    ));

    let retained_string = control.usage().string_bytes;
    let total_input = required
        .checked_add(retained_string)
        .expect("bounded mapping input");
    let operations = control
        .usage()
        .depth
        .checked_add(control.usage().visited_fields)
        .expect("bounded mapping traversal");
    let input_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, required - 1)
        .expect("positive one-short mapping input grant");
    let mut resources = ResourceAccountant::new(input_short, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == 0
                && value.requested() == required
    ));

    let retained_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input - 1)
        .expect("positive one-short mapping retention grant");
    let mut resources = ResourceAccountant::new(retained_short, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == required
                && value.requested() == retained_string
    ));

    let exact_read = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input)
        .expect("positive mapping input grant")
        .grant(BudgetDimension::Operation, operations)
        .expect("positive mapping operation grant")
        .grant(BudgetDimension::DerivedArtifact, 1)
        .expect("positive mapping artifact grant")
        .grant(BudgetDimension::OutputByte, required)
        .expect("positive mapping output grant");
    let mut resources = ResourceAccountant::new(exact_read, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let operation_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operations - 1)
        .expect("positive one-short operation grant");
    let mut resources = ResourceAccountant::new(operation_short, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.consumed() == control.usage().depth
                && value.requested() == control.usage().visited_fields
    ));
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-6",
    "FR-033-AC-2",
    "FR-033-AC-4",
    "FR-033-AC-5"
)]
fn tc_015_correction_is_direct_and_preserves_predecessor_bytes() {
    let fixture = fixture();
    let predecessor = validated(base_input(&fixture));
    let prior_bytes = predecessor.bytes().to_vec();
    let prior_id = predecessor.result_id().to_owned();
    let mut corrected = base_input(&fixture);
    corrected.execution = AssessmentExecution::Unsupported;
    corrected.truth = Truth::Unavailable;
    corrected.settlement.basis = SettlementBasis::Unavailable;
    corrected.provenance.input_identity = "input:corrected".into();
    corrected.correction = Some(CorrectionInput {
        relation: CorrectionRelation::Invalidates,
        predecessor: &predecessor,
        contradicted_premise_identity: "premise:branch",
        corrected_input_identity: "input:corrected",
    });
    let correction = validated(corrected);
    assert_ne!(correction.result_id(), prior_id);
    assert_eq!(predecessor.result_id(), prior_id);
    assert_eq!(predecessor.bytes(), prior_bytes);
    let edge = correction
        .correction()
        .expect("typed direct correction edge");
    assert_eq!(edge.relation(), CorrectionRelation::Invalidates);
    assert_eq!(edge.predecessor_result_id(), prior_id);
    assert_eq!(edge.predecessor_digest(), predecessor.digest());
    assert_eq!(edge.contradicted_premise_identity(), "premise:branch");
    assert_eq!(edge.corrected_input_identity(), "input:corrected");

    let mapped = contract_ir::map(
        &correction,
        contract_ir::MappingSelection::current(),
        Limits::owner_max(),
    )
    .expect("corrected mapping");
    let mapped_edge = mapped.correction().expect("mapped direct correction edge");
    assert_eq!(mapped_edge.predecessor_result_id(), prior_id);
    assert_eq!(mapped_edge.predecessor_digest(), predecessor.digest());
}

#[test]
#[trace("TC-015", "FR-042-AC-1", "FR-042-AC-5")]
fn tc_015_schema_bytes_and_digests_are_immutable_and_closed() {
    assert_eq!(
        format!("{:x}", ByteDigest::of(result::SCHEMA_BYTES)),
        result::SCHEMA_SHA256
    );
    assert_eq!(
        format!("{:x}", ByteDigest::of(contract_ir::SCHEMA_BYTES)),
        contract_ir::SCHEMA_SHA256
    );
    for schema in [result::SCHEMA_BYTES, contract_ir::SCHEMA_BYTES] {
        let value: Value = serde_json::from_slice(schema).expect("schema JSON");
        assert_eq!(value["additionalProperties"], Value::Bool(false));
    }
}

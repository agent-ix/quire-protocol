// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-003: graph-owned transactional compensation assessment.

use ix_trace_rs::trace;
use quire_observation::StaticProducerBundle;
use quire_observation::{QualifiedSubject, SubjectIdentity, SubjectKind};
use quire_protocol::assessment::{
    with_assessment_after_resolving, AssessmentInputs, ResolvedAssessmentScope,
};
use quire_protocol::bounds::SystemMonotonicClock;
use quire_protocol::claims::Claim;
use quire_protocol::compensation::algebra::{AttemptOutcome, EffectDisposition, OrderingViolation};
use quire_protocol::compensation::graph::{MissingEvidence, Outcome, Refusal, Transition};
use quire_protocol::model::v2::{
    Internality, LinkedAssessment, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticSelection, StaticSlot,
    RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::outcome::OperationOutcome;
use quire_protocol::producer_contracts::{
    ProducerContract, ProducerReadiness, ProducerResolutionRequest,
};
use quire_protocol::version_lock::{ResolutionBudget, VersionLockManifest};
use quire_protocol::wire::{BindingKind, Body, ControlOperation};
use quire_spec_language::state::{
    EvaluationOutcome, Limits as StateLimits, StateView, ValueKind as StateValueKind,
};
use std::time::Duration;

// The complete owner handoff fixture is intentionally broader than this
// scenario-specific target; the harness retains its internal fixture helpers.
#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod handoff;
// The shared owner fixture also contains scope views used by the retained
// pure-kernel TC-003; this integration target exercises its ordering slice.
#[allow(dead_code)]
#[path = "support/observation_owner.rs"]
mod observation_owner;
#[path = "support/producer_selection.rs"]
mod producer_selection;
#[path = "support/v2_state.rs"]
mod v2_state;

const ORDER_KIND: &[u8] = b"ix://agent-ix/commerce/type/Order";

fn package() -> quire_spec_language::protocol_artifact::v2::AdmittedPackage {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("published v2 offer admits")
}

fn producer() -> quire_observation::AdmittedStaticBundle {
    StaticProducerBundle::admit_json(include_bytes!("fixtures/fcd-static-bundle-1.2.json"))
        .expect("shared FCD static bundle is admitted")
}

fn subject(
    producer: &quire_observation::AdmittedStaticBundle,
    identity: &[u8],
) -> QualifiedSubject {
    QualifiedSubject::new(
        producer,
        SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
        SubjectIdentity::new(identity.to_vec()).expect("non-empty identity"),
    )
}

struct Fixture {
    package: quire_spec_language::protocol_artifact::v2::AdmittedPackage,
    definition: StaticBindingDefinition,
    runtime: RuntimeBindingSet,
    first_attempt: RuntimeBinding,
    first_effect: RuntimeBinding,
    second_effect: RuntimeBinding,
    commit: RuntimeBinding,
}

fn fixture() -> Fixture {
    fixture_with_options(None, true)
}

fn fixture_with_ordering_subject(shared: Option<&QualifiedSubject>) -> Fixture {
    fixture_with_options(shared, true)
}

fn fixture_without_forward_effect() -> Fixture {
    fixture_with_options(None, false)
}

fn fixture_with_options(
    shared: Option<&QualifiedSubject>,
    include_forward_effect: bool,
) -> Fixture {
    let package = package();
    let (declaration_handle, declaration) = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            matches!(&declaration.body, Body::Protocol { .. }).then_some((index, declaration))
        })
        .map(|(index, declaration)| {
            (
                u32::try_from(index).expect("bounded protocol handle"),
                declaration,
            )
        })
        .expect("published protocol declaration");
    let Body::Protocol {
        roles,
        compensations,
        controls,
        ..
    } = &declaration.body
    else {
        unreachable!("selected protocol declaration")
    };
    let compensation = &compensations[0];
    let partial = &compensations[1];
    let forward_slot = declaration
        .bindings
        .iter()
        .position(|binding| {
            binding.kind == BindingKind::Effect
                && matches!(&binding.subject, quire_protocol::wire::Subject::Control { control } if *control == compensation.forward_effect)
        })
        .and_then(|index| u32::try_from(index).ok())
        .expect("selected forward effect slot");
    let commit_handle = compensation.commit.0.as_ref().expect("forbidding commit");
    let commit_slot = match &controls[commit_handle.index as usize].operation {
        ControlOperation::Commit { instance, .. } => *instance,
        _ => panic!("selected commit control"),
    };
    let mut slot_handles = vec![
        compensation.attempt_instance,
        compensation.effect_instance,
        partial.effect_instance,
        commit_slot,
    ];
    if include_forward_effect {
        slot_handles.insert(0, forward_slot);
    }
    let slots = slot_handles
        .into_iter()
        .map(|handle| StaticSlot {
            handle: SlotHandle::new(handle),
            compatibility: SlotCompatibility::Subject {
                kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
            },
        })
        .collect();
    let producer = producer();
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        StaticSelection {
            identity: "binding-definition:compensation".into(),
            revision: "1".into(),
            digest: [7; 32],
        },
        producer.key().clone(),
        declaration_handle,
        roles
            .iter()
            .enumerate()
            .map(|(index, _)| RoleClassification {
                role_handle: u32::try_from(index).expect("bounded role"),
                internality: Internality::Internal,
            })
            .collect(),
        slots,
    )
    .expect("compensation static definition");
    let forward_subject = shared
        .cloned()
        .unwrap_or_else(|| subject(&producer, b"forward-effect:1"));
    let forward = RuntimeBinding::subject(SlotHandle::new(forward_slot), forward_subject);
    let first_attempt = RuntimeBinding::subject(
        SlotHandle::new(compensation.attempt_instance),
        subject(&producer, b"compensation-attempt:1"),
    );
    let effect_subject = shared
        .cloned()
        .unwrap_or_else(|| subject(&producer, b"compensation-effect:1"));
    let first_effect = RuntimeBinding::subject(
        SlotHandle::new(compensation.effect_instance),
        effect_subject,
    );
    let second_effect = RuntimeBinding::subject(
        SlotHandle::new(partial.effect_instance),
        subject(&producer, b"partial-compensation-effect:1"),
    );
    let commit_subject = shared
        .cloned()
        .unwrap_or_else(|| subject(&producer, b"commit:1"));
    let commit = RuntimeBinding::subject(SlotHandle::new(commit_slot), commit_subject);
    let mut bindings = vec![
        first_attempt.clone(),
        first_effect.clone(),
        second_effect.clone(),
        commit.clone(),
    ];
    if include_forward_effect {
        bindings.insert(0, forward);
    }
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        bindings,
    )
    .expect("runtime selection");
    Fixture {
        package,
        definition,
        runtime,
        first_attempt,
        first_effect,
        second_effect,
        commit,
    }
}

fn with_engine<'a, T>(
    fixture: &'a Fixture,
    run: impl for<'scope> FnOnce(ResolvedAssessmentScope<'scope, 'a>) -> T,
) -> T {
    let runtime_selection = producer_selection::runtime(&fixture.runtime, 500_000)
        .expect("fixture runtime selection is bounded");
    let request = ProducerResolutionRequest::for_assessment(
        fixture.package.digest(),
        fixture.definition.selection().digest,
        &[Claim::Conformance],
        runtime_selection,
    )
    .expect("the closed claim selection is representable");
    let manifest = VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .map(ProducerContract::manifest_entry)
            .collect(),
    );
    let budget = ResolutionBudget::new(
        ProducerContract::ALL.len(),
        runtime_selection.framed_bytes(),
        Duration::from_secs(1),
    )
    .expect("exact producer budget");
    let ProducerReadiness::Executable(OperationOutcome::Completed(output)) =
        with_assessment_after_resolving(
            &request,
            &manifest,
            budget,
            SystemMonotonicClock,
            |permit| {
                AssessmentInputs::new(
                    permit,
                    LinkedAssessment::new(&fixture.package, &fixture.definition, &fixture.runtime)
                        .expect("linked assessment"),
                    vec![Claim::Conformance],
                )
            },
            run,
        )
    else {
        panic!("request-scoped engine must start")
    };
    output
}

fn no_expression_work() -> StateLimits {
    StateLimits {
        expression_work: 0,
        ..StateLimits::default()
    }
}

fn lower_expression_limit() -> StateLimits {
    StateLimits {
        expression_work: StateLimits::default().expression_work - 1,
        ..StateLimits::default()
    }
}

fn assessed_ordering(order: [usize; 3]) -> Vec<OrderingViolation> {
    let views = observation_owner::ordering_views("compensation", "base", order);
    let fixture = fixture_with_ordering_subject(Some(&views.subject));
    let compensation = match &fixture.package.inherited().declarations
        [fixture.definition.declaration_handle() as usize]
        .body
    {
        Body::Protocol { compensations, .. } => &compensations[0],
        _ => unreachable!("protocol"),
    };
    let guard = StateView {
        binders: vec![v2_state::plain_binder(
            &fixture.package,
            compensation.trigger.clone(),
            false,
        )],
        populations: Vec::new(),
    };
    with_engine(&fixture, |mut engine| {
        assert_eq!(
            engine.activate_compensation(0, &guard, StateLimits::default()),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.record_compensation_effect(0, &fixture.first_effect, None),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.record_compensation_commit(0, &fixture.commit),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.assess_compensation_ordering(
                0,
                &views.observations[0],
                &views.observations[1],
                Some(&views.observations[2]),
                &views.positions,
            ),
            Outcome::Completed(Transition::Applied)
        );
        engine.compensation_state().orderings()[0]
            .violations()
            .to_vec()
    })
}

/// The production Engine walks the admitted guard/retry/recovery handles and
/// commits only exact graph/QObs candidates. Replay is a no-op and every
/// refusal leaves the owned snapshot unchanged.
#[trace(
    "TC-003",
    "FR-003-AC-2",
    "FR-003-AC-4",
    "FR-003-AC-5",
    "FR-003-AC-6",
    "FR-003-AC-7",
    "FR-020-AC-1",
    "FR-020-AC-3",
    "FR-020-AC-4",
    "FR-022-AC-1",
    "FR-022-AC-2",
    "FR-022-AC-3",
    "FR-022-AC-4",
    "FR-023-AC-1",
    "FR-023-AC-2",
    "FR-023-AC-5",
    "FR-024-AC-2",
    "FR-024-AC-1",
    "FR-024-AC-3",
    "FR-024-AC-4",
    "FR-024-AC-5",
    "FR-024-AC-6"
)]
#[test]
fn tc_003_engine_is_graph_owned_exact_and_transactional() {
    let fixture = fixture();
    let producer = producer();
    let compensation = match &fixture.package.inherited().declarations
        [fixture.definition.declaration_handle() as usize]
        .body
    {
        Body::Protocol { compensations, .. } => &compensations[0],
        _ => unreachable!("protocol"),
    };
    let partial = match &fixture.package.inherited().declarations
        [fixture.definition.declaration_handle() as usize]
        .body
    {
        Body::Protocol { compensations, .. } => &compensations[1],
        _ => unreachable!("protocol"),
    };

    let missing_fixture = fixture_without_forward_effect();
    let missing_compensation = match &missing_fixture.package.inherited().declarations
        [missing_fixture.definition.declaration_handle() as usize]
        .body
    {
        Body::Protocol { compensations, .. } => &compensations[0],
        _ => unreachable!("protocol"),
    };
    let missing_guard = StateView {
        binders: vec![v2_state::plain_binder(
            &missing_fixture.package,
            missing_compensation.trigger.clone(),
            false,
        )],
        populations: Vec::new(),
    };
    with_engine(&missing_fixture, |mut missing_engine| {
        let Outcome::UnavailableAfterEvaluation {
            missing: MissingEvidence::ForwardEffect,
            report,
        } = missing_engine.activate_compensation(0, &missing_guard, StateLimits::default())
        else {
            panic!("true guard without its exact forward effect must preserve both facts")
        };
        assert!(matches!(
            report.outcome(),
            EvaluationOutcome::Completed(value)
                if matches!(value.kind(), StateValueKind::Boolean(true))
        ));
        assert!(missing_engine.compensation().registrations().is_empty());
        assert!(missing_engine.compensation_state().activations().is_empty());
    });

    with_engine(&fixture, |mut engine| {
        assert!(engine.compensation().registrations().is_empty());

        let false_guard = StateView {
            binders: vec![v2_state::plain_binder(
                &fixture.package,
                compensation.trigger.clone(),
                true,
            )],
            populations: Vec::new(),
        };
        let before = engine.compensation().clone();
        assert!(matches!(
            engine.activate_compensation(0, &false_guard, StateLimits::default()),
            Outcome::GuardFalse(_)
        ));
        assert_eq!(engine.compensation(), &before);
        assert!(matches!(
            engine.activate_compensation(0, &StateView::default(), StateLimits::default()),
            Outcome::Source(_)
        ));
        assert_eq!(engine.compensation(), &before);

        let guard = StateView {
            binders: vec![v2_state::plain_binder(
                &fixture.package,
                compensation.trigger.clone(),
                false,
            )],
            populations: Vec::new(),
        };
        assert!(matches!(
            engine.activate_compensation(0, &guard, no_expression_work()),
            Outcome::Source(_)
        ));
        assert_eq!(engine.compensation(), &before);
        assert_eq!(
            engine.activate_compensation(0, &guard, StateLimits::default()),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(engine.compensation().registrations().len(), 1);
        assert_eq!(
            engine.conformance().registrations().len(),
            1,
            "the conformance kernel retains the same committed compensation snapshot"
        );
        assert_eq!(
            engine.activate_compensation(0, &guard, StateLimits::default()),
            Outcome::Completed(Transition::Replay)
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.activate_compensation(0, &guard, lower_expression_limit()),
            Outcome::Refused(Refusal::ConflictingReplay)
        );
        assert_eq!(engine.compensation_state(), &before);

        let before = engine.compensation_state().clone();
        assert!(matches!(
            engine.admit_compensation_attempt(
                0,
                &fixture.first_effect,
                None,
                Some(AttemptOutcome::Failed),
                None,
                StateLimits::default(),
            ),
            Outcome::Refused(Refusal::UnexpectedSlot { .. })
        ));
        assert_eq!(engine.compensation_state(), &before);

        assert_eq!(
            engine.admit_compensation_attempt(
                0,
                &fixture.first_attempt,
                None,
                Some(AttemptOutcome::Failed),
                None,
                StateLimits::default(),
            ),
            Outcome::Completed(Transition::Applied)
        );
        let recovery = v2_state::published_recovery_view(
            &fixture.package,
            compensation.forward.clone(),
            compensation.trigger.clone(),
            compensation.recovery.clone(),
            2,
            1,
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.evaluate_compensation_recovery(0, &recovery, StateLimits::default()),
            Outcome::Unavailable(MissingEvidence::CompensationEffect)
        );
        assert_eq!(engine.compensation_state(), &before);

        let missing_related_attempt = RuntimeBinding::subject(
            SlotHandle::new(compensation.attempt_instance),
            subject(&producer, b"compensation-attempt:missing-related-effect"),
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.admit_compensation_attempt(
                0,
                &missing_related_attempt,
                Some(&fixture.first_effect),
                Some(AttemptOutcome::Successful),
                None,
                StateLimits::default(),
            ),
            Outcome::Unavailable(MissingEvidence::CompensationEffect)
        );
        assert_eq!(engine.compensation_state(), &before);

        assert_eq!(
            engine.record_compensation_effect(0, &fixture.first_effect, None),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.record_compensation_effect(0, &fixture.first_effect, None),
            Outcome::Completed(Transition::Replay)
        );
        let after_effect = engine.compensation_state().clone();
        assert!(matches!(
            engine.record_compensation_effect(
                0,
                &fixture.first_effect,
                Some(EffectDisposition::Duplicate),
            ),
            Outcome::Refused(Refusal::ConflictingReplay)
        ));
        assert_eq!(engine.compensation_state(), &after_effect);

        let retry = v2_state::retry_view(
            &fixture.package,
            compensation.forward.clone(),
            compensation.trigger.clone(),
            compensation.earlier.clone(),
            compensation.later.clone(),
            1,
            2,
        );
        let refused_retry = v2_state::retry_view(
            &fixture.package,
            compensation.forward.clone(),
            compensation.trigger.clone(),
            compensation.earlier.clone(),
            compensation.later.clone(),
            2,
            1,
        );
        let refused_candidate = RuntimeBinding::subject(
            SlotHandle::new(compensation.attempt_instance),
            subject(&producer, b"compensation-attempt:refused"),
        );
        let before = engine.compensation_state().clone();
        let Outcome::Refused(Refusal::RetryNotAuthorized(report)) = engine
            .admit_compensation_attempt(
                0,
                &refused_candidate,
                Some(&fixture.first_effect),
                Some(AttemptOutcome::Successful),
                Some(&refused_retry),
                StateLimits::default(),
            )
        else {
            panic!("completed-false retry must retain its exact owner report")
        };
        assert!(matches!(
            report.outcome(),
            EvaluationOutcome::Completed(value)
                if matches!(value.kind(), StateValueKind::Boolean(false))
        ));
        assert_eq!(engine.compensation_state(), &before);
        for ordinal in 2..=3 {
            let candidate = RuntimeBinding::subject(
                SlotHandle::new(compensation.attempt_instance),
                subject(
                    &producer,
                    format!("compensation-attempt:{ordinal}").as_bytes(),
                ),
            );
            if ordinal == 2 {
                let before = engine.compensation_state().clone();
                assert!(matches!(
                    engine.admit_compensation_attempt(
                        0,
                        &candidate,
                        Some(&fixture.first_effect),
                        Some(AttemptOutcome::Successful),
                        Some(&retry),
                        no_expression_work(),
                    ),
                    Outcome::Source(_)
                ));
                assert_eq!(engine.compensation_state(), &before);
            }
            assert_eq!(
                engine.admit_compensation_attempt(
                    0,
                    &candidate,
                    Some(&fixture.first_effect),
                    Some(AttemptOutcome::Successful),
                    Some(&retry),
                    StateLimits::default(),
                ),
                Outcome::Completed(Transition::Applied)
            );
        }
        let second_attempt = RuntimeBinding::subject(
            SlotHandle::new(compensation.attempt_instance),
            subject(&producer, b"compensation-attempt:2"),
        );
        assert_eq!(
            engine.admit_compensation_attempt(
                0,
                &second_attempt,
                Some(&fixture.first_effect),
                Some(AttemptOutcome::Successful),
                Some(&retry),
                StateLimits::default(),
            ),
            Outcome::Completed(Transition::Replay)
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.admit_compensation_attempt(
                0,
                &second_attempt,
                Some(&fixture.first_effect),
                Some(AttemptOutcome::Successful),
                Some(&retry),
                lower_expression_limit(),
            ),
            Outcome::Refused(Refusal::ConflictingReplay)
        );
        assert_eq!(engine.compensation_state(), &before);
        let over = RuntimeBinding::subject(
            SlotHandle::new(compensation.attempt_instance),
            subject(&producer, b"compensation-attempt:4"),
        );
        let before = engine.compensation_state().clone();
        assert!(matches!(
            engine.admit_compensation_attempt(
                0,
                &over,
                Some(&fixture.first_effect),
                Some(AttemptOutcome::Successful),
                Some(&retry),
                StateLimits::default(),
            ),
            Outcome::Refused(Refusal::RetryBoundExceeded {
                bound: 3,
                attempted: 4
            })
        ));
        assert_eq!(engine.compensation_state(), &before);
        assert!(matches!(
            engine.admit_compensation_attempt(
                0,
                &fixture.first_attempt,
                None,
                Some(AttemptOutcome::Successful),
                None,
                StateLimits::default(),
            ),
            Outcome::Refused(Refusal::ConflictingReplay)
        ));
        assert_eq!(engine.compensation_state(), &before);

        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.evaluate_compensation_recovery(0, &recovery, StateLimits::default()),
            Outcome::Unavailable(quire_protocol::compensation::graph::MissingEvidence::Commit)
        );
        assert_eq!(engine.compensation_state(), &before);
        assert_eq!(
            engine.record_compensation_commit(0, &fixture.commit),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.record_compensation_commit(0, &fixture.commit),
            Outcome::Completed(Transition::Replay)
        );
        let conflicting_commit = RuntimeBinding::subject(
            fixture.commit.slot(),
            subject(&producer, b"commit:conflicting"),
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.record_compensation_commit(0, &conflicting_commit),
            Outcome::Refused(Refusal::ConflictingReplay)
        );
        assert_eq!(engine.compensation_state(), &before);
        let before = engine.compensation_state().clone();
        assert!(matches!(
            engine.evaluate_compensation_recovery(0, &recovery, no_expression_work()),
            Outcome::Source(_)
        ));
        assert_eq!(engine.compensation_state(), &before);
        assert_eq!(
            engine.evaluate_compensation_recovery(0, &recovery, StateLimits::default()),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.evaluate_compensation_recovery(0, &recovery, StateLimits::default()),
            Outcome::Completed(Transition::Replay)
        );
        assert!(matches!(
            engine.compensation_state().recoveries()[0]
                .report()
                .outcome(),
            EvaluationOutcome::Completed(value)
                if matches!(value.kind(), StateValueKind::Boolean(false))
        ));
        assert_eq!(
            engine.compensation_state().recoveries()[0].view(),
            &recovery
        );
        let changed_recovery = v2_state::published_recovery_view(
            &fixture.package,
            compensation.forward.clone(),
            compensation.trigger.clone(),
            compensation.recovery.clone(),
            1,
            1,
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.evaluate_compensation_recovery(0, &changed_recovery, StateLimits::default(),),
            Outcome::Refused(Refusal::ConflictingReplay)
        );
        assert_eq!(engine.compensation_state(), &before);

        let partial_guard = StateView {
            binders: vec![v2_state::plain_binder(
                &fixture.package,
                partial.trigger.clone(),
                false,
            )],
            populations: Vec::new(),
        };
        assert_eq!(
            engine.activate_compensation(1, &partial_guard, StateLimits::default()),
            Outcome::Completed(Transition::Applied)
        );
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.record_compensation_commit(1, &fixture.commit),
            Outcome::Refused(Refusal::CommitNotSelected)
        );
        assert_eq!(engine.compensation_state(), &before);
        assert_eq!(
            engine.record_compensation_effect(1, &fixture.second_effect, None),
            Outcome::Completed(Transition::Applied)
        );
        let before = engine.compensation_state().clone();
        assert!(matches!(
            engine.evaluate_compensation_recovery(1, &StateView::default(), StateLimits::default()),
            Outcome::Source(_)
        ));
        assert_eq!(engine.compensation_state(), &before);
        let partial_recovery = v2_state::published_recovery_view(
            &fixture.package,
            partial.forward.clone(),
            partial.trigger.clone(),
            partial.recovery.clone(),
            2,
            1,
        );
        assert_eq!(
            engine.evaluate_compensation_recovery(1, &partial_recovery, StateLimits::default()),
            Outcome::Completed(Transition::Applied)
        );
        assert!(matches!(
            engine.compensation_state().recoveries()[1]
                .report()
                .outcome(),
            EvaluationOutcome::Completed(value)
                if matches!(value.kind(), StateValueKind::Boolean(true))
        ));
        assert_eq!(engine.compensation_state().attempts().len(), 3);
        assert_eq!(engine.compensation_state().effects().len(), 2);
        assert_eq!(engine.compensation_state().commits().len(), 1);
        assert_eq!(engine.compensation_state().recoveries().len(), 2);
        assert_eq!(
            engine.compensation_state().recoveries()[0]
                .report()
                .accounting_version(),
            quire_spec_language::state::ACCOUNTING_VERSION
        );
        assert_eq!(engine.compensation().registrations().len(), 2);
        assert_eq!(
            engine.conformance().registrations().len(),
            2,
            "conformance retains the exact committed compensation population"
        );
        assert_eq!(engine.compensation_state().activations().len(), 2);
        assert_eq!(engine.compensation_state().activations()[0].view(), &guard);
        assert_eq!(engine.compensation_state().attempts().len(), 3);
        assert!(engine.compensation_state().attempts()[0]
            .established_effect()
            .is_none());
        assert_eq!(
            engine.compensation_state().attempts()[1].established_effect(),
            fixture.first_effect.subject_value()
        );
        assert_eq!(
            engine.compensation_state().attempts()[1].retry_view(),
            Some(&retry)
        );
        assert_eq!(engine.compensation_state().effects().len(), 2);
        assert_eq!(engine.compensation_state().commits().len(), 1);
        assert_eq!(engine.compensation_state().recoveries().len(), 2);
    });
}

/// Commit ordering consumes only strict QObs observation and position views.
/// Missing owner evidence cannot mutate prior assessment state, while exact
/// replay and each authored violation remain distinct.
#[trace(
    "TC-003",
    "FR-003-AC-3",
    "FR-003-AC-4",
    "FR-021-AC-1",
    "FR-021-AC-2",
    "FR-021-AC-3",
    "FR-021-AC-4"
)]
#[test]
fn tc_003_ordering_uses_exact_qobs_positions() {
    let valid = observation_owner::ordering_views("compensation", "base", [0, 1, 2]);
    let fixture = fixture_with_ordering_subject(Some(&valid.subject));
    let compensation = match &fixture.package.inherited().declarations
        [fixture.definition.declaration_handle() as usize]
        .body
    {
        Body::Protocol { compensations, .. } => &compensations[0],
        _ => unreachable!("protocol"),
    };
    let guard = StateView {
        binders: vec![v2_state::plain_binder(
            &fixture.package,
            compensation.trigger.clone(),
            false,
        )],
        populations: Vec::new(),
    };
    with_engine(&fixture, |mut engine| {
        assert_eq!(
            engine.activate_compensation(0, &guard, StateLimits::default()),
            Outcome::Completed(Transition::Applied)
        );
        assert_eq!(
            engine.record_compensation_effect(0, &fixture.first_effect, None),
            Outcome::Completed(Transition::Applied)
        );

        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.assess_compensation_ordering(
                0,
                &valid.observations[0],
                &valid.observations[1],
                None,
                &valid.positions,
            ),
            Outcome::Unavailable(MissingEvidence::Commit)
        );
        assert_eq!(engine.compensation_state(), &before);
        assert_eq!(
            engine.record_compensation_commit(0, &fixture.commit),
            Outcome::Completed(Transition::Applied)
        );

        let alternate = observation_owner::ordering_views("compensation", "alternate", [0, 1, 2]);
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.assess_compensation_ordering(
                0,
                &valid.observations[0],
                &valid.observations[1],
                Some(&valid.observations[2]),
                &alternate.positions,
            ),
            Outcome::Unavailable(MissingEvidence::Position)
        );
        assert_eq!(engine.compensation_state(), &before);

        assert_eq!(
            engine.assess_compensation_ordering(
                0,
                &valid.observations[0],
                &valid.observations[1],
                Some(&valid.observations[2]),
                &valid.positions,
            ),
            Outcome::Completed(Transition::Applied)
        );
        assert!(engine.compensation_state().orderings()[0]
            .violations()
            .is_empty());
        assert_eq!(
            engine.compensation_state().orderings()[0].forward(),
            &valid.observations[0]
        );
        assert_eq!(
            engine.compensation_state().orderings()[0].effect(),
            &valid.observations[1]
        );
        assert_eq!(
            engine.compensation_state().orderings()[0].commit(),
            Some(&valid.observations[2])
        );
        assert_eq!(
            engine.compensation_state().orderings()[0].positions(),
            &valid.positions
        );
        assert_eq!(
            engine.assess_compensation_ordering(
                0,
                &valid.observations[0],
                &valid.observations[1],
                Some(&valid.observations[2]),
                &valid.positions,
            ),
            Outcome::Completed(Transition::Replay)
        );

        let effect_first = observation_owner::ordering_views("compensation", "base", [1, 0, 2]);
        let before = engine.compensation_state().clone();
        assert_eq!(
            engine.assess_compensation_ordering(
                0,
                &effect_first.observations[0],
                &effect_first.observations[1],
                Some(&effect_first.observations[2]),
                &effect_first.positions,
            ),
            Outcome::Refused(Refusal::ConflictingReplay)
        );
        assert_eq!(engine.compensation_state(), &before);
        assert_eq!(
            assessed_ordering([1, 0, 2]),
            &[OrderingViolation::CompensationBeforeForwardEffect]
        );

        assert_eq!(
            assessed_ordering([0, 2, 1]),
            &[OrderingViolation::CompensationAfterCommit]
        );

        assert_eq!(
            assessed_ordering([2, 1, 0]),
            &[
                OrderingViolation::RegistrationAfterCommit,
                OrderingViolation::CompensationBeforeForwardEffect,
                OrderingViolation::CompensationAfterCommit,
            ]
        );
    });
}

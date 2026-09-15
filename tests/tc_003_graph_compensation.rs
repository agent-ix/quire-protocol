// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-003: graph-owned transactional compensation assessment.

use agent_ix_baseline_producer::StaticProducerBundle;
use ix_trace_rs::trace;
use quire_observation::{QualifiedSubject, SubjectIdentity, SubjectKind};
use quire_protocol::assessment::Engine;
use quire_protocol::claims::Claim;
use quire_protocol::compensation::algebra::{AttemptOutcome, EffectDisposition};
use quire_protocol::compensation::graph::{Outcome, Refusal, Transition};
use quire_protocol::model::v2::{
    Internality, LinkedAssessment, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticSelection, StaticSlot,
    RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::wire::{BindingKind, Body, ControlOperation};
use quire_spec_language::state::{Limits as StateLimits, StateView};

#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod handoff;
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
    commit: RuntimeBinding,
}

fn fixture() -> Fixture {
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
    let slots = [
        forward_slot,
        compensation.attempt_instance,
        compensation.effect_instance,
        commit_slot,
    ]
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
    let forward = RuntimeBinding::subject(
        SlotHandle::new(forward_slot),
        subject(&producer, b"forward-effect:1"),
    );
    let first_attempt = RuntimeBinding::subject(
        SlotHandle::new(compensation.attempt_instance),
        subject(&producer, b"compensation-attempt:1"),
    );
    let first_effect = RuntimeBinding::subject(
        SlotHandle::new(compensation.effect_instance),
        subject(&producer, b"compensation-effect:1"),
    );
    let commit = RuntimeBinding::subject(
        SlotHandle::new(commit_slot),
        subject(&producer, b"commit:1"),
    );
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![
            forward,
            first_attempt.clone(),
            first_effect.clone(),
            commit.clone(),
        ],
    )
    .expect("runtime selection");
    Fixture {
        package,
        definition,
        runtime,
        first_attempt,
        first_effect,
        commit,
    }
}

fn engine(fixture: &Fixture) -> Engine<'_> {
    let context = LinkedAssessment::new(&fixture.package, &fixture.definition, &fixture.runtime)
        .expect("linked assessment");
    Engine::new(context, vec![Claim::Conformance]).expect("request-scoped engine")
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
    let mut engine = engine(&fixture);
    assert!(engine.compensation().registrations().is_empty());

    let before = engine.compensation().clone();
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
    assert_eq!(
        engine.activate_compensation(0, &guard, StateLimits::default()),
        Outcome::Completed(Transition::Applied)
    );
    assert_eq!(engine.compensation().registrations().len(), 1);
    assert_eq!(
        engine.activate_compensation(0, &guard, StateLimits::default()),
        Outcome::Completed(Transition::Replay)
    );

    assert_eq!(
        engine.admit_compensation_attempt(
            0,
            &fixture.first_attempt,
            Some(AttemptOutcome::Failed),
            None,
            StateLimits::default(),
        ),
        Outcome::Completed(Transition::Applied)
    );
    let retry = v2_state::retry_view(
        &fixture.package,
        compensation.forward.clone(),
        compensation.trigger.clone(),
        compensation.earlier.clone(),
        compensation.later.clone(),
    );
    for ordinal in 2..=3 {
        let candidate = RuntimeBinding::subject(
            SlotHandle::new(compensation.attempt_instance),
            subject(
                &producer,
                format!("compensation-attempt:{ordinal}").as_bytes(),
            ),
        );
        assert_eq!(
            engine.admit_compensation_attempt(
                0,
                &candidate,
                Some(AttemptOutcome::Successful),
                Some(&retry),
                StateLimits::default(),
            ),
            Outcome::Completed(Transition::Applied)
        );
    }
    let over = RuntimeBinding::subject(
        SlotHandle::new(compensation.attempt_instance),
        subject(&producer, b"compensation-attempt:4"),
    );
    let before = engine.compensation_state().clone();
    assert!(matches!(
        engine.admit_compensation_attempt(
            0,
            &over,
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
            Some(AttemptOutcome::Successful),
            None,
            StateLimits::default(),
        ),
        Outcome::Refused(Refusal::ConflictingReplay)
    ));
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

    assert_eq!(
        engine.record_compensation_commit(0, &fixture.commit),
        Outcome::Completed(Transition::Applied)
    );
    let recovery = v2_state::published_recovery_view(
        &fixture.package,
        compensation.forward.clone(),
        compensation.trigger.clone(),
        compensation.recovery.clone(),
    );
    assert_eq!(
        engine.evaluate_compensation_recovery(0, &recovery, StateLimits::default()),
        Outcome::Completed(Transition::Applied)
    );
    assert_eq!(
        engine.evaluate_compensation_recovery(0, &recovery, StateLimits::default()),
        Outcome::Completed(Transition::Replay)
    );
    assert_eq!(engine.compensation_state().attempts().len(), 3);
    assert_eq!(engine.compensation_state().effects().len(), 1);
    assert_eq!(engine.compensation_state().commits().len(), 1);
    assert_eq!(engine.compensation_state().recoveries().len(), 1);
    assert_eq!(
        engine.compensation_state().recoveries()[0]
            .report()
            .accounting_version(),
        quire_spec_language::state::ACCOUNTING_VERSION
    );
}

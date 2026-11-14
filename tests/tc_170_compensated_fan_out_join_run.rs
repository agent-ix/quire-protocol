// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-170: the branching compensated choreography with fan-out and join, run
//! end to end (V1-RUN-012, IT-005). The timed-refund run is in TC-154's file,
//! where the deadline fixture lives.

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use std::num::NonZeroU32;

use ix_trace_rs::trace;
use quire_protocol::compensation::algebra::{
    AttemptAdmission, AttemptFact, AttemptLedger, AttemptOutcome, Definition, EffectDisposition,
    ForwardFact, Limits as AlgebraLimits, Registry,
};
use quire_protocol::compensation::purity::{CapabilityInventory, PureAssessment};
use quire_protocol::dynamics::{assess_joins, JoinEvent, JoinFact, JoinOutcome};
use quire_protocol::ids::{AttemptId, DefinitionId, EffectId};
use support::dynamic::{at, workflow, Dynamic};

struct Pure;

impl CapabilityInventory for Pure {
    fn capabilities(&self) -> &[quire_protocol::compensation::purity::Capability] {
        &[]
    }
}

fn pure() -> PureAssessment {
    PureAssessment::checked(&Pure).expect("empty capability surface")
}

fn effect(name: &str) -> EffectId {
    EffectId::new(name.as_bytes(), name)
}

/// `JoinAll` fans out to branches `a`, `b` and `c`. `a` and `b` complete and
/// establish their forward effects; `c` never completes, so the `all` join
/// does not settle and the two completed branches are compensated: `a` by one
/// successful attempt, `b` by a failed attempt and a successful retry. Each
/// stage reads only the facts of the stage before it.
#[trace("TC-170")]
#[test]
fn a_fan_out_whose_join_cannot_settle_compensates_each_completed_branch() {
    let dynamic = Dynamic::with("workflow", 3);
    let join = dynamic.named("control", "JoinAll");
    let facts: Vec<JoinFact> = ["a", "b"]
        .iter()
        .enumerate()
        .map(|(index, branch)| JoinFact {
            at: at(u64::try_from(index).expect("small") + 1),
            join: join.clone(),
            workflow: workflow("w1"),
            repeat_ordinals: Vec::new(),
            event: JoinEvent::BranchCompleted {
                branch: (*branch).into(),
            },
        })
        .collect();
    let outcomes = assess_joins(dynamic.linked().declarations(), &facts, |node| {
        dynamic.construct(node)
    })
    .expect("a declared join");
    let [JoinOutcome::Assessed(assessment)] = outcomes.as_slice() else {
        panic!("one assessed join: {outcomes:?}");
    };
    assert!(!assessment.settled);
    assert_eq!(assessment.completed.len(), 2);

    let mut registry = Registry::default();
    let mut registrations = Vec::new();
    for branch in &assessment.completed {
        let forward = effect(&format!("effect:{branch}"));
        let definition = Definition {
            identity: DefinitionId::new(b"definition:undo", "undo a branch"),
            paired_forward_effect: forward.clone(),
            retry_bound: NonZeroU32::new(2),
        };
        let events = registry
            .observe(
                pure(),
                &[definition],
                &ForwardFact::EstablishedEffect(forward),
            )
            .expect("bounded registration");
        assert!(events[0].created);
        registrations.push((branch.clone(), events[0].registration.clone()));
    }
    assert_ne!(
        registrations[0].1.identity, registrations[1].1.identity,
        "each branch's effect has its own registration"
    );

    for (branch, registration) in &registrations {
        let undone = effect(&format!("effect:undo-{branch}"));
        let attempt = |number: u32, outcome, established| AttemptFact {
            attempt: AttemptId::new(
                format!("attempt:{branch}:{number}").as_bytes(),
                "undo attempt",
            ),
            registration: registration.identity.clone(),
            established_effect: established,
            outcome: Some(outcome),
        };
        let mut ledger =
            AttemptLedger::new(registration.clone(), AlgebraLimits::owner_max()).expect("ledger");
        let mut admitted = 0;
        if &**branch == "b" {
            let first = ledger.admit(pure(), attempt(1, AttemptOutcome::Failed, None));
            assert!(matches!(first, AttemptAdmission::Admitted { .. }));
            admitted += 1;
        }
        let last = ledger.admit(
            pure(),
            attempt(2, AttemptOutcome::Successful, Some(undone.clone())),
        );
        assert!(matches!(last, AttemptAdmission::Admitted { .. }));
        admitted += 1;
        assert_eq!(ledger.attempts().len(), admitted);
        let effects = ledger.effects();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].effect, undone);
        assert_ne!(effects[0].disposition, Some(EffectDisposition::Duplicate));
    }
}

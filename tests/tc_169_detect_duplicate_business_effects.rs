// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-169: a duplicate business effect is decided from effect identity,
//! independently of duplicate delivery (FR-023-AC-6).

use std::num::NonZeroU32;

use ix_trace_rs::trace;
use quire_protocol::compensation::algebra::{
    AttemptFact, AttemptLedger, AttemptOutcome, Definition, EffectDisposition, ForwardFact, Limits,
    Registry,
};
use quire_protocol::compensation::purity::{Capability, CapabilityInventory, PureAssessment};
use quire_protocol::ids::{AttemptId, DefinitionId, EffectId};

struct Pure;

impl CapabilityInventory for Pure {
    fn capabilities(&self) -> &[Capability] {
        &[]
    }
}

fn pure() -> PureAssessment {
    PureAssessment::checked(&Pure).expect("empty capability surface")
}

fn effect(name: &str) -> EffectId {
    EffectId::new(name.as_bytes(), name)
}

/// A ledger whose attempts, in order, establish `effects` (`None` for an
/// attempt that established none).
fn ledger(effects: &[Option<&str>]) -> AttemptLedger {
    let forward = effect("effect:forward");
    let definition = Definition {
        identity: DefinitionId::new(b"definition:refund", "refund"),
        paired_forward_effect: forward.clone(),
        retry_bound: NonZeroU32::new(8),
    };
    let events = Registry::default()
        .observe(
            pure(),
            &[definition],
            &ForwardFact::EstablishedEffect(forward),
        )
        .expect("bounded registration");
    let registration = events[0].registration.clone();
    let mut ledger = AttemptLedger::new(registration.clone(), Limits::owner_max()).expect("ledger");
    for (number, established) in effects.iter().enumerate() {
        ledger.admit(
            pure(),
            AttemptFact {
                attempt: AttemptId::new(format!("attempt:{number}").as_bytes(), "attempt"),
                registration: registration.identity.clone(),
                established_effect: established.map(effect),
                outcome: Some(AttemptOutcome::Successful),
            },
        );
    }
    ledger
}

#[test]
#[trace("TC-169", "FR-023-AC-6")]
fn a_second_distinct_effect_is_a_duplicate_and_a_redelivered_one_is_not() {
    let one = ledger(&[None, Some("refund-1")]).effects();
    assert_eq!(
        one.len(),
        1,
        "an attempt that establishes nothing adds none"
    );
    assert_eq!(one[0].disposition, None);

    let redelivered = ledger(&[Some("refund-1"), Some("refund-1")]).effects();
    assert_eq!(redelivered, one, "a redelivery is not a second effect");

    let doubled = ledger(&[Some("refund-1"), Some("refund-2")]).effects();
    assert_eq!(doubled.len(), 2);
    assert_eq!(doubled[0].disposition, None);
    assert_eq!(doubled[1].disposition, Some(EffectDisposition::Duplicate));

    let mixed = ledger(&[Some("refund-1"), Some("refund-1"), Some("refund-2")]).effects();
    assert_eq!(
        mixed, doubled,
        "delivery duplication does not change the verdict"
    );

    assert!(ledger(&[]).effects().is_empty());
}

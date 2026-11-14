// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-169: a duplicate business effect is decided from effect identity,
//! independently of duplicate delivery (FR-023-AC-6).

use ix_trace_rs::trace;
use quire_protocol::compensation::algebra::{retain_effects, EffectDisposition};
use quire_protocol::ids::{EffectId, RegistrationId};

fn effect(name: &str) -> EffectId {
    EffectId::new(name.as_bytes(), name)
}

#[test]
#[trace("TC-169", "FR-023-AC-6")]
fn a_second_distinct_effect_is_a_duplicate_and_a_redelivered_one_is_not() {
    let registration = RegistrationId::new(b"registration:one", "registration");
    let one = retain_effects(&registration, [effect("refund-1")]);
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].disposition, None);

    let redelivered = retain_effects(&registration, [effect("refund-1"), effect("refund-1")]);
    assert_eq!(redelivered, one, "a redelivery is not a second effect");

    let doubled = retain_effects(&registration, [effect("refund-1"), effect("refund-2")]);
    assert_eq!(doubled.len(), 2);
    assert_eq!(doubled[0].disposition, None);
    assert_eq!(doubled[1].disposition, Some(EffectDisposition::Duplicate));
    assert_eq!(doubled[1].registration, registration);

    let mixed = retain_effects(
        &registration,
        [effect("refund-1"), effect("refund-1"), effect("refund-2")],
    );
    assert_eq!(
        mixed, doubled,
        "delivery duplication does not change the verdict"
    );

    assert!(retain_effects(&registration, []).is_empty());
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-002/Task-002: the closed protocol identity set (FR-002-AC-1, FR-003-AC-3).
//!
//! The compile-time half of FR-002-AC-1 — that no two identities are
//! interchangeable — is proved by the `compile_fail` examples on
//! `quire_protocol::ids`, which run as documentation tests under `cargo test`.
//! This file proves the runtime half: the set is closed and distinct, display
//! text is never identity, and producer identity bytes are retained unchanged.

use std::any::TypeId;
use std::collections::{BTreeMap, HashMap, HashSet};

use ix_trace_rs::trace;
use quire_protocol::ids::{
    AttemptId, ChannelId, CommitId, ComponentId, DefinitionId, DeliveryId, EffectId, EndpointId,
    IdentityKind, MessageId, ParticipantId, ReceiveId, RecoveryId, RegistrationId, RoleInstanceId,
    SendId, WorkflowInstanceId,
};
use quire_protocol::wire;

/// Every identity type in the closed set, named once so that a seventeenth type
/// cannot be introduced without this list and `IdentityKind::ALL` disagreeing.
fn every_type_id() -> Vec<(&'static str, TypeId)> {
    vec![
        ("DefinitionId", TypeId::of::<DefinitionId>()),
        ("RoleInstanceId", TypeId::of::<RoleInstanceId>()),
        ("WorkflowInstanceId", TypeId::of::<WorkflowInstanceId>()),
        ("ParticipantId", TypeId::of::<ParticipantId>()),
        ("ComponentId", TypeId::of::<ComponentId>()),
        ("EndpointId", TypeId::of::<EndpointId>()),
        ("ChannelId", TypeId::of::<ChannelId>()),
        ("MessageId", TypeId::of::<MessageId>()),
        ("SendId", TypeId::of::<SendId>()),
        ("ReceiveId", TypeId::of::<ReceiveId>()),
        ("DeliveryId", TypeId::of::<DeliveryId>()),
        ("AttemptId", TypeId::of::<AttemptId>()),
        ("EffectId", TypeId::of::<EffectId>()),
        ("RegistrationId", TypeId::of::<RegistrationId>()),
        ("CommitId", TypeId::of::<CommitId>()),
        ("RecoveryId", TypeId::of::<RecoveryId>()),
    ]
}

#[trace("TC-002", "Task-002")]
#[test]
fn tc_002_newtypes_are_not_interchangeable() {
    let types = every_type_id();

    // A type alias would make two names share one `TypeId`; sixteen distinct
    // ids is the runtime witness that sixteen distinct types exist.
    let distinct: HashSet<TypeId> = types.iter().map(|(_, id)| *id).collect();
    assert_eq!(
        distinct.len(),
        types.len(),
        "two identity names resolve to the same type: aliases are interchangeable"
    );

    // The type list and the kind list are the same closed set.
    assert_eq!(types.len(), IdentityKind::ALL.len());
    assert_eq!(IdentityKind::ALL.len(), 16);

    let kinds: HashSet<IdentityKind> = IdentityKind::ALL.iter().copied().collect();
    assert_eq!(
        kinds.len(),
        IdentityKind::ALL.len(),
        "duplicate identity kind"
    );

    let tags: HashSet<&str> = IdentityKind::ALL.iter().map(|kind| kind.tag()).collect();
    assert_eq!(tags.len(), IdentityKind::ALL.len(), "two kinds share a tag");
}

#[trace("TC-002", "Task-002")]
#[test]
fn tc_002_each_newtype_reports_its_own_kind() {
    assert_eq!(
        DefinitionId::new(b"a", "a").kind(),
        IdentityKind::Definition
    );
    assert_eq!(
        RoleInstanceId::new(b"a", "a").kind(),
        IdentityKind::RoleInstance
    );
    assert_eq!(
        WorkflowInstanceId::new(b"a", "a").kind(),
        IdentityKind::WorkflowInstance
    );
    assert_eq!(
        ParticipantId::new(b"a", "a").kind(),
        IdentityKind::Participant
    );
    assert_eq!(ComponentId::new(b"a", "a").kind(), IdentityKind::Component);
    assert_eq!(EndpointId::new(b"a", "a").kind(), IdentityKind::Endpoint);
    assert_eq!(ChannelId::new(b"a", "a").kind(), IdentityKind::Channel);
    assert_eq!(MessageId::new(b"a", "a").kind(), IdentityKind::Message);
    assert_eq!(SendId::new(b"a", "a").kind(), IdentityKind::Send);
    assert_eq!(ReceiveId::new(b"a", "a").kind(), IdentityKind::Receive);
    assert_eq!(DeliveryId::new(b"a", "a").kind(), IdentityKind::Delivery);
    assert_eq!(AttemptId::new(b"a", "a").kind(), IdentityKind::Attempt);
    assert_eq!(EffectId::new(b"a", "a").kind(), IdentityKind::Effect);
    assert_eq!(
        RegistrationId::new(b"a", "a").kind(),
        IdentityKind::Registration
    );
    assert_eq!(CommitId::new(b"a", "a").kind(), IdentityKind::Commit);
    assert_eq!(RecoveryId::new(b"a", "a").kind(), IdentityKind::Recovery);
}

#[trace("TC-002", "Task-002")]
#[test]
fn tc_002_display_text_is_not_identity() {
    // Equal producer bytes, different display text: the same identity.
    let renamed = SendId::new(b"send-7f21", "Place order");
    let relabelled = SendId::new(b"send-7f21", "Bestellung aufgeben");
    assert_eq!(renamed, relabelled);
    assert_ne!(renamed.display_text(), relabelled.display_text());

    // Equal display text, different producer bytes: different identities.
    let first = SendId::new(b"send-7f21", "Place order");
    let second = SendId::new(b"send-9c04", "Place order");
    assert_ne!(first, second);
    assert_eq!(first.display_text(), second.display_text());

    // Lookup keys off the identity, not the rendering.
    let mut hashed: HashMap<SendId, u32> = HashMap::new();
    hashed.insert(renamed.clone(), 1);
    assert_eq!(hashed.get(&relabelled), Some(&1));
    assert_eq!(hashed.get(&second), None);

    let mut ordered: BTreeMap<SendId, u32> = BTreeMap::new();
    ordered.insert(renamed.clone(), 1);
    ordered.insert(relabelled.clone(), 2);
    assert_eq!(
        ordered.len(),
        1,
        "display text split one identity into two keys"
    );
    ordered.insert(second.clone(), 3);
    assert_eq!(ordered.len(), 2);

    // Ordering follows the bytes, not the rendering: `9c04` sorts after `7f21`
    // although its display text is identical.
    assert!(first < second);

    // Rendering an identity and re-rendering it changes nothing observable.
    assert_eq!(renamed.to_string(), "Place order");
    assert_eq!(relabelled.to_string(), "Bestellung aufgeben");
    assert_eq!(renamed, relabelled);
    assert_eq!(renamed.identity_bytes(), relabelled.identity_bytes());
}

#[trace("TC-002", "Task-002")]
#[test]
fn tc_002_identity_bytes_are_retained_unchanged() {
    // No trim, no case fold, no Unicode normalisation, no UTF-8 requirement.
    let adversarial: &[&[u8]] = &[
        b"  leading-and-trailing  ",
        b"MixedCase",
        b"mixedcase",
        b"\x00embedded-nul",
        b"tab\tand\nnewline",
        &[0xff, 0xfe, 0x00, 0x41],
        "e\u{0301}".as_bytes(), // decomposed
        "\u{00e9}".as_bytes(),  // composed
    ];

    for bytes in adversarial {
        let identity = ChannelId::new(bytes, "orders");
        assert_eq!(identity.identity_bytes(), *bytes);
    }

    // Forms a producer may consider "the same name" stay distinct identities.
    assert_ne!(
        ChannelId::new(b"  orders  ", "orders"),
        ChannelId::new(b"orders", "orders")
    );
    assert_ne!(
        ChannelId::new(b"Orders", "orders"),
        ChannelId::new(b"orders", "orders")
    );
    assert_ne!(
        ChannelId::new("e\u{0301}".as_bytes(), "é"),
        ChannelId::new("\u{00e9}".as_bytes(), "é")
    );
}

#[trace("TC-002", "Task-002")]
#[test]
fn tc_002_wire_package_identities_round_trip_unchanged() {
    // A `wire::Package` definition entry, taken as the producer offers it.
    let definition = wire::Definition {
        identity: "  Orders.Protocol\u{00e9}  ".to_owned(),
        revision: wire::Revision {
            namespace: "quire.baseline".to_owned(),
            value: "2026-09-01".to_owned(),
        },
        artifact: 0,
        rules: vec![0],
        requires: vec![],
    };

    let retained = DefinitionId::new(definition.identity.as_bytes(), "Orders protocol");
    assert_eq!(retained.identity_bytes(), definition.identity.as_bytes());
    assert_eq!(
        String::from_utf8(retained.identity_bytes().to_vec()).unwrap(),
        definition.identity
    );

    // The declaration's authored requirement identity is a different producer
    // field and therefore a different protocol identity, even though both
    // render under the same display text.
    let requirement = wire::Requirement {
        package: "orders".to_owned(),
        identity: "FR-050".to_owned(),
        revision: wire::Revision {
            namespace: "quire.baseline".to_owned(),
            value: "2026-09-01".to_owned(),
        },
    };
    let other = DefinitionId::new(requirement.identity.as_bytes(), "Orders protocol");
    assert_ne!(retained, other);
    assert_eq!(retained.display_text(), other.display_text());
}

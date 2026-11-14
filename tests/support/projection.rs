// SPDX-License-Identifier: AGPL-3.0-or-later
//! A three-role protocol over the recipe model, QSL's real `protocol-analysis`
//! engine registered as the provider, and the claim that projects it.

// Each test binary uses a different part of these helpers.
#![allow(dead_code)]

use std::convert::Infallible;

use qsl_analyze::protocol_analysis::ProjectionRequest;
use qsl_foundation::witness::ObservationIdentity;
use qsl_replay::protocol_analysis::{AssessedRootDraft, ProjectionBoundsDraft, RunBudget};
use qsl_route::ProviderOrigin;
use quire_contract_model::CheckedNodeId;
use quire_exact::{
    EffectiveId, NodeKey, ObjectId, ObjectReference, OriginalDeclarationKey, PathSegment,
    UniverseId,
};
use quire_protocol::dispatch::{
    dispatch, Answer, Binding, ChannelPremise, Claim, Fragment, Premises, ProtocolClaimKind,
    ProtocolOutcome, ProtocolRequest, ProviderBoundary,
};
use quire_protocol::engine::{engine_descriptor, EngineRequest, EngineResult, QslEngine};
use quire_protocol::link::LinkedProtocol;

use super::dynamic::Dynamic;

pub type Asked = Claim<(), (), (), ()>;
pub type Boundary<'p> = ProviderBoundary<'p, Asked, Answer<EngineResult, Infallible>>;

pub const RECORDS: &str = "record Ack { ok: Boolean; }\nrecord Note { ok: Boolean; }\n\
    record PickMsg { ok: Boolean; }\nrecord CancelMsg { ok: Boolean; }\n";

pub const MEMBERS: &str = "role Buyer on M::Workflow;\nrole Seller on M::Provider;\n\
    role Warehouse on M::Depot;\n\
    channel Order from Buyer to Seller carries Ack ordering unordered delivery exactly-once-premise capacity 4 overflow reject;\n\
    channel Pick from Seller to Warehouse carries PickMsg ordering unordered delivery exactly-once-premise capacity 4 overflow reject;\n\
    channel Cancelled from Seller to Warehouse carries CancelMsg ordering unordered delivery exactly-once-premise capacity 4 overflow reject;\n";

pub const PICK: &str = "send PickIt via Pick as (p: PickMsg) { true };\n\
    receive PickedUp via Pick of PickIt as (q: PickMsg) { true };";
pub const PICK2: &str = "send PickIt2 via Pick as (p2: PickMsg) { true };\n\
    receive PickedUp2 via Pick of PickIt2 as (q2: PickMsg) { true };";
pub const CANCELLED: &str = "send Cn via Cancelled as (c: CancelMsg) { true };\n\
    receive Cn2 via Cancelled of Cn as (d: CancelMsg) { true };";

/// `Buyer` places an order; `Seller` reads it and owns `ship | cancel`.
pub fn worked(ship: &str, cancel: &str) -> Dynamic {
    Dynamic::from_source(&format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile pp = \"quire.protocol.finite-global/v1\";\n\
         model M = \"agent-ix/checked-handoff\";\n\
         {RECORDS}protocol Flow using pp over (input: M::Workflow) on origin {{\n{MEMBERS}\n\
         run sequence Main {{\n\
         send Placed via Order as (placed: Ack) {{ true }};\n\
         receive Got via Order of Placed as (got: Ack) {{ true }};\n\
         choice Decide by Seller visible (got.ok) {{\n\
           case no when {{ not got.ok }} sequence Cancel {{ {cancel} }}\n\
           case yes when {{ got.ok }} sequence Ship {{ {ship} }}\n\
         }} }}\n\
         finish Done as (zz: Boolean) {{ true }};\n}}\n"
    ))
}

pub fn assessed_root() -> AssessedRootDraft {
    let key = |seed: &str| NodeKey::new("fixture", vec![PathSegment::Text(seed.to_owned())], 0);
    AssessedRootDraft {
        observation: Some(ObservationIdentity {
            authority: "test".to_owned(),
            identity: "protocol-claim".to_owned(),
        }),
        object: Some(ObjectReference::new(
            UniverseId::new(key("u").expect("well formed")),
            EffectiveId::new(None, OriginalDeclarationKey::new("fixture", "e")),
            ObjectId::new("flow-1").expect("an object id"),
        )),
    }
}

/// The claim: project the global protocol onto the channels and roles named.
pub fn claim(dynamic: &Dynamic) -> ProtocolRequest<(), (), ()> {
    claim_of(
        ProtocolClaimKind::LocalProjection,
        dynamic.linked(),
        dynamic.clause(),
        |form, name| dynamic.named(form, name),
    )
}

/// The claim of `kind` over the channels and roles of a linked protocol,
/// each found by `named` from its form and declared name.
pub fn claim_of(
    kind: ProtocolClaimKind,
    linked: &LinkedProtocol,
    clause: &CheckedNodeId,
    named: impl Fn(&str, &str) -> CheckedNodeId,
) -> ProtocolRequest<(), (), ()> {
    let channels: Vec<_> = ["Order", "Pick", "Cancelled"]
        .iter()
        .map(|name| named("channel", name))
        .collect();
    let roles: Vec<_> = ["Buyer", "Seller", "Warehouse"]
        .iter()
        .map(|name| named("role", name))
        .collect();
    ProtocolRequest {
        kind,
        fragment: Fragment {
            channels: channels.clone(),
            roles: roles.clone(),
        },
        backend: None,
        assumptions: (),
        premises: Premises {
            channels: channels
                .into_iter()
                .map(|node| {
                    let declared = linked
                        .declarations()
                        .channel(&node)
                        .expect("a declared channel");
                    let premise = ChannelPremise {
                        ordering: declared.ordering.clone(),
                        delivery: declared.delivery,
                        unread: (),
                    };
                    (node, premise)
                })
                .collect(),
            roles: roles.into_iter().map(|node| (node, ())).collect(),
        },
        sources: vec![clause.clone()],
    }
}

/// QSL's engine registered as the provider, asked to project onto `role`
/// with `profile`.
pub fn boundary<'p>(
    dynamic: &'p Dynamic,
    role: &'static str,
    profile: &'static str,
) -> Boundary<'p> {
    let mut boundary = Boundary::new();
    let engine = QslEngine::new(dynamic.package(), move |_: &Binding<(), (), ()>| {
        EngineRequest::Projection(Box::new(ProjectionRequest {
            protocol: dynamic.clause().clone(),
            definition_key: Some("ix://test/Flow".to_owned()),
            role: dynamic.named("role", role),
            profile: Some(profile.to_owned()),
            bounds: Some(ProjectionBoundsDraft {
                max_depth: Some(64),
            }),
            assessed_root: Some(assessed_root()),
            budget: RunBudget::default(),
        }))
    })
    .expect("QSL's linked engines register");
    boundary
        .register(
            engine_descriptor(ProviderOrigin::Linked).expect("QSL's own descriptor"),
            engine,
        )
        .expect("the registration is accepted");
    boundary
}

pub fn settle(
    dynamic: &Dynamic,
    role: &'static str,
    profile: &'static str,
) -> ProtocolOutcome<EngineResult, Binding<(), (), ()>> {
    let mut boundary = boundary(dynamic, role, profile);
    dispatch(
        &mut boundary,
        dynamic.linked().declarations(),
        &claim(dynamic),
    )
}

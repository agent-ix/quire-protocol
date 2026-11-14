// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-164: `local-projection` claims dispatched to QSL's real
//! `protocol-analysis` engine, with no stand-in (FR-150, FR-152, FR-153,
//! TL-387 AC1, AC2).

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use std::convert::Infallible;

use ix_trace_rs::trace;
use qsl_analyze::protocol_analysis::manifest::BACKEND_ID;
use qsl_analyze::protocol_analysis::{ProjectionRequest, ProtocolAnalysisArtifact};
use qsl_foundation::witness::ObservationIdentity;
use qsl_replay::protocol_analysis::{
    AnalysisOutcome, AssessedRootDraft, ProjectionBoundsDraft, RunBudget,
};
use qsl_replay::{SettlementBasis, Truth};
use qsl_route::ProviderOrigin;
use quire_exact::{
    EffectiveId, NodeKey, ObjectId, ObjectReference, OriginalDeclarationKey, PathSegment,
    UniverseId,
};
use quire_protocol::dispatch::{
    dispatch, Answer, BackendResult, Binding, ChannelPremise, Claim, Fragment, Premises,
    ProtocolClaimKind, ProtocolOutcome, ProtocolRequest, ProviderBoundary,
};
use quire_protocol::disposition::{ClaimSettlement, Disposition};
use quire_protocol::engine::{engine_descriptor, EngineRequest, EngineResult, QslEngine};
use support::dynamic::Dynamic;

type Asked = Claim<(), (), (), ()>;
type Boundary<'p> = ProviderBoundary<'p, Asked, Answer<EngineResult, Infallible>>;

const RECORDS: &str = "record Ack { ok: Boolean; }\nrecord Note { ok: Boolean; }\n\
    record PickMsg { ok: Boolean; }\nrecord CancelMsg { ok: Boolean; }\n";

const MEMBERS: &str = "role Buyer on M::Workflow;\nrole Seller on M::Provider;\n\
    role Warehouse on M::Depot;\n\
    channel Order from Buyer to Seller carries Ack ordering unordered delivery exactly-once-premise capacity 4 overflow reject;\n\
    channel Pick from Seller to Warehouse carries PickMsg ordering unordered delivery exactly-once-premise capacity 4 overflow reject;\n\
    channel Cancelled from Seller to Warehouse carries CancelMsg ordering unordered delivery exactly-once-premise capacity 4 overflow reject;\n";

const PICK: &str = "send PickIt via Pick as (p: PickMsg) { true };\n\
    receive PickedUp via Pick of PickIt as (q: PickMsg) { true };";
const CANCELLED: &str = "send Cn via Cancelled as (c: CancelMsg) { true };\n\
    receive Cn2 via Cancelled of Cn as (d: CancelMsg) { true };";

/// `Buyer` places an order; `Seller` reads it and owns `ship | cancel`.
fn worked(ship: &str, cancel: &str) -> Dynamic {
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

fn assessed_root() -> AssessedRootDraft {
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
fn claim(dynamic: &Dynamic) -> ProtocolRequest<(), (), ()> {
    let channels: Vec<_> = ["Order", "Pick", "Cancelled"]
        .iter()
        .map(|name| dynamic.named("channel", name))
        .collect();
    let roles: Vec<_> = ["Buyer", "Seller", "Warehouse"]
        .iter()
        .map(|name| dynamic.named("role", name))
        .collect();
    ProtocolRequest {
        kind: ProtocolClaimKind::LocalProjection,
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
                    let declared = dynamic
                        .linked()
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
        sources: vec![dynamic.clause().clone()],
    }
}

/// QSL's engine registered as the provider, asked to project onto `role`
/// with `profile`.
fn boundary<'p>(dynamic: &'p Dynamic, role: &'static str, profile: &'static str) -> Boundary<'p> {
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

fn settle(
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

#[test]
#[trace("TC-164", "FR-150-AC-12")]
fn a_sound_projection_is_decided_by_the_real_engine_and_carried_satisfied() {
    let dynamic = worked(PICK, CANCELLED);
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, premises }) =
        settle(&dynamic, "Warehouse", "bounded-static")
    else {
        panic!("the engine decides the claim");
    };
    assert_eq!(
        premises.selected.as_ref().map(|id| id.as_str()),
        Some(BACKEND_ID)
    );
    assert_eq!(result.truth(), Truth::Satisfied);
    assert_eq!(result.basis(), SettlementBasis::ClosedScope);
    assert!(result.witness().is_none());
    let Some(ProtocolAnalysisArtifact::Projection(found)) = result.artifact() else {
        panic!("a projection result");
    };
    assert!(matches!(found.outcome, AnalysisOutcome::Proved(_)));
}

#[test]
#[trace("TC-164", "FR-150-AC-13")]
fn a_refuted_projection_carries_the_engines_witness_unchanged() {
    let dynamic = worked(PICK, "");
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) =
        settle(&dynamic, "Warehouse", "bounded-static")
    else {
        panic!("the engine decides the claim");
    };
    assert_eq!(result.truth(), Truth::Violated);
    assert_eq!(result.basis(), SettlementBasis::DecisiveCounterexample);
    let Some(ProtocolAnalysisArtifact::Projection(found)) = result.artifact() else {
        panic!("a projection result");
    };
    let AnalysisOutcome::Refuted(counterexample) = &found.outcome else {
        panic!("a counterexample");
    };
    assert_eq!(result.witness(), Some(&counterexample.separating_witness));
}

#[test]
#[trace("TC-164", "FR-153-AC-8")]
fn a_request_the_engine_refuses_is_not_a_result_and_settles_no_truth() {
    let dynamic = worked(PICK, CANCELLED);
    let outcome = settle(&dynamic, "Warehouse", "unbounded");
    assert!(
        !matches!(
            outcome,
            ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
        ),
        "{outcome:?}"
    );
    assert!(
        matches!(
            outcome,
            ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Refused(_)))
        ),
        "{outcome:?}"
    );
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-165: qualification corpus for `local-projection` over a checked
//! protocol, run through the linked engine with no stand-in (IT-004; FR-150,
//! FR-152, FR-029).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod projection;
}

use std::convert::Infallible;

use ix_trace_rs::trace;
use qsl_replay::Truth;
use quire_protocol::dispatch::{
    dispatch_claims, BackendResult, ClaimDisposition, ClaimRequest, ProtocolOutcome,
};
use quire_protocol::disposition::ClaimSettlement;
use support::projection::{boundary, claim, settle, worked, CANCELLED, PICK, PICK2};

/// The truth the engine settles for projecting `ship`/`cancel` onto each role.
fn truths(ship: &str, cancel: &str) -> Vec<(&'static str, Truth)> {
    let dynamic = worked(ship, cancel);
    ["Buyer", "Seller", "Warehouse"]
        .into_iter()
        .map(|role| {
            let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) =
                settle(&dynamic, role, "bounded-static")
            else {
                panic!("{role}: the engine decides the claim");
            };
            (role, result.truth())
        })
        .collect()
}

#[test]
#[trace("TC-165", "FR-150-AC-14")]
fn the_corpus_settles_each_role_of_each_workflow_by_the_engines_own_decision() {
    // Branches the warehouse can tell apart: every role projects.
    assert_eq!(
        truths(PICK, CANCELLED),
        [
            ("Buyer", Truth::Satisfied),
            ("Seller", Truth::Satisfied),
            ("Warehouse", Truth::Satisfied)
        ]
    );
    // Branches it cannot tell apart: only the uninformed role is refuted.
    assert_eq!(
        truths(PICK, ""),
        [
            ("Buyer", Truth::Satisfied),
            ("Seller", Truth::Satisfied),
            ("Warehouse", Truth::Violated)
        ]
    );
    // Equal first actions merge.
    assert_eq!(truths(PICK, PICK2)[2], ("Warehouse", Truth::Satisfied));
}

#[test]
#[trace("TC-165", "FR-150-AC-15")]
fn a_projection_claim_leaves_a_global_conformance_result_untouched() {
    let dynamic = worked(PICK, CANCELLED);
    let mut boundary = boundary(&dynamic, "Warehouse", "bounded-static");
    let settled = dispatch_claims::<_, _, _, _, (), _, Infallible>(
        &mut boundary,
        dynamic.linked().declarations(),
        &[],
        &[
            ClaimRequest::Protocol(claim(&dynamic)),
            ClaimRequest::GlobalConformance("conformant"),
        ],
    );
    assert!(matches!(
        settled[1],
        ClaimDisposition::GlobalConformance("conformant")
    ));
    let ClaimDisposition::Protocol(ProtocolOutcome::Settled(ClaimSettlement::Result {
        result,
        ..
    })) = &settled[0]
    else {
        panic!("the engine decides the projection");
    };
    assert_eq!(result.truth(), Truth::Satisfied);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-045: the outcome of a protocol-slot evaluation over the checked
//! package, as QSL FR-520's outcome map states it.
//!
//! The evaluation is QSL's: `BarePackage` evaluates the slot from the
//! package's own nodes (QSL FR-531, FR-533). This module maps the result into
//! [`SlotOutcome`], one variant per row of the map, and carries every owner
//! value unchanged. No variant is a Boolean for an undefined, refused,
//! incomplete or unavailable result, and nothing here converts an evaluation
//! to `bool`.

use std::num::NonZeroU32;

use qsl_eval::value::{
    BinderCharges, CheckedPackageEvaluation, ProtocolSlot, ProtocolSlotEvaluation, ProtocolSupply,
};
use qsl_package::BarePackage;
use quire_contract_model::{
    CheckedBindingNameKind, CheckedNodeId, CheckedSemanticNodeV2, CheckedTerm, LiteralKind,
};
use quire_eval::{CallFailure, FamilyOutcome, FamilyResult};
use quire_exact::{Incomplete, Meter, NodeKey, Outcome, Refusal, Undefined, Value};
use quire_semantic_value::catalog::CatalogCode;

/// What a slot evaluation produced.
#[derive(Debug)]
pub enum SlotOutcome {
    /// The slot completed with `true`.
    Holds,
    /// The slot completed with `false`.
    DoesNotHold,
    /// The slot completed with a value that is not a Boolean.
    NotBoolean(Value),
    /// A binder was supplied unavailable: the binder and the owner's cause.
    Unavailable {
        /// The binder read.
        binder: NodeKey,
        /// The owner's catalog code.
        cause: CatalogCode,
    },
    /// The kernel left the slot undefined.
    Undefined(Undefined),
    /// The kernel refused the slot.
    Refused(Refusal),
    /// The kernel stopped on a resource bound.
    Incomplete(Incomplete),
    /// The slot's family reported a result of its own, such as an
    /// unsupported construct.
    Family(FamilyResult),
}

/// Map one owner outcome into the slot outcome, carrying its value unchanged.
pub fn outcome_of(outcome: FamilyOutcome<Value>) -> SlotOutcome {
    match outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(true))) => SlotOutcome::Holds,
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(false))) => {
            SlotOutcome::DoesNotHold
        }
        FamilyOutcome::Evaluated(Outcome::Completed(other)) => SlotOutcome::NotBoolean(other),
        FamilyOutcome::Evaluated(Outcome::Undefined(undefined)) => {
            SlotOutcome::Undefined(undefined)
        }
        FamilyOutcome::Evaluated(Outcome::Refused(refusal)) => SlotOutcome::Refused(refusal),
        FamilyOutcome::Evaluated(Outcome::Incomplete(incomplete)) => {
            SlotOutcome::Incomplete(incomplete)
        }
        FamilyOutcome::FamilyEvaluated(result) => SlotOutcome::Family(result),
    }
}

/// A slot's mapped outcome with QSL's per-binder charge log (QSL FR-520-AC-11),
/// carried unchanged.
#[derive(Debug)]
pub struct SlotEvaluation {
    /// The outcome of FR-520's map.
    pub outcome: SlotOutcome,
    /// The charges of each binder read, the unattributed entry and the
    /// accounting version.
    pub charges: BinderCharges,
}

/// Evaluate `slot` of `package` over `supply` and map the result.
///
/// # Errors
///
/// QSL's [`CallFailure`], unchanged: an admission failure is the owner's
/// record and charges nothing.
pub fn evaluate(
    package: &BarePackage<'_>,
    slot: &ProtocolSlot,
    supply: &ProtocolSupply<'_>,
    meter: &mut Meter,
) -> Result<SlotEvaluation, CallFailure> {
    Ok(match package.evaluate_protocol_slot(slot, supply, meter)? {
        ProtocolSlotEvaluation::Evaluated(evaluation, charges) => SlotEvaluation {
            outcome: outcome_of(evaluation.outcome),
            charges,
        },
        ProtocolSlotEvaluation::Unavailable {
            binder,
            cause,
            charges,
            ..
        } => SlotEvaluation {
            outcome: SlotOutcome::Unavailable { binder, cause },
            charges,
        },
    })
}

/// The retry bound a compensation node authors: its `attempts` argument's
/// `maximum`, which counts the initial attempt (QSL FR-520 "tc_003
/// behaviors"; FR-022).
///
/// # Errors
///
/// [`BoundRefusal`] naming the node when `attempts` is present without a
/// positive integer `maximum`.
pub fn attempts_maximum(node: &CheckedSemanticNodeV2) -> Result<Option<NonZeroU32>, BoundRefusal> {
    let refuse = || BoundRefusal {
        node: node.node_id.clone(),
    };
    let CheckedTerm::Application { arguments, .. } = &node.body else {
        return Err(refuse());
    };
    let Some(attempts) = named(arguments, "attempts") else {
        return Ok(None);
    };
    let CheckedTerm::Aggregate { members } = attempts else {
        return Err(refuse());
    };
    let Some(CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    }) = named(members, "maximum")
    else {
        return Err(refuse());
    };
    value
        .as_str()
        .and_then(|text| text.parse::<u32>().ok())
        .and_then(NonZeroU32::new)
        .map(Some)
        .ok_or_else(refuse)
}

fn named<'t>(terms: &'t [CheckedTerm], wanted: &str) -> Option<&'t CheckedTerm> {
    terms.iter().find_map(|term| match term {
        CheckedTerm::Binding { name, value }
            if name.kind() == CheckedBindingNameKind::Other(wanted) =>
        {
            Some(&**value)
        }
        _ => None,
    })
}

/// A compensation node whose `attempts maximum` is not a positive integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundRefusal {
    /// The compensation node.
    pub node: CheckedNodeId,
}

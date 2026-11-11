// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-101: the captures of an obligation activation, bound as ordered,
//! immutable, atomic occurrences.
//!
//! The values are the owner's. QSL's checked-package evaluation (FR-045)
//! returns, for each authored capture and at the inclusive activation
//! boundary, a slot outcome; this module binds those outcomes and evaluates no
//! capture expression of its own, so a supplied value that differs from what
//! the authored expression would give locally is the bound value. One capture
//! without a value refuses the whole activation and binds none. QSL's
//! `Unsupported::Capture`, a capture of the protocol clause itself, is
//! FR-057's and never reported from here.

use quire_contract_model::CheckedNodeId;
use quire_exact::Value;
use quire_observation::authority::TemporalBoundary;

use crate::compensation::activate::Binding;
use crate::compensation::purity::PureAssessment;
use crate::slot::SlotOutcome;

/// QSL's evaluation of one authored capture at one boundary.
#[derive(Debug)]
pub struct CaptureSupply {
    /// The authored capture.
    pub capture: CheckedNodeId,
    /// The boundary the value was evaluated at.
    pub boundary: TemporalBoundary,
    /// QSL's slot outcome, unchanged.
    pub evaluation: SlotOutcome,
}

/// The identity of one capture occurrence: its activation and its place in
/// the authored order. Identical expressions at two places are two
/// occurrences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureOccurrenceId {
    /// The activation's FR-300 binding.
    pub activation: Binding,
    /// Zero-based place in the authored order.
    pub ordinal: usize,
}

/// One bound capture, immutable.
#[derive(Clone, Debug)]
pub struct CaptureOccurrence {
    id: CaptureOccurrenceId,
    capture: CheckedNodeId,
    value: Value,
}

impl CaptureOccurrence {
    /// The occurrence identity.
    #[must_use]
    pub const fn id(&self) -> &CaptureOccurrenceId {
        &self.id
    }

    /// The authored capture.
    #[must_use]
    pub const fn capture(&self) -> &CheckedNodeId {
        &self.capture
    }

    /// The bound value, exactly as QSL supplied it.
    #[must_use]
    pub const fn value(&self) -> &Value {
        &self.value
    }
}

/// The captures of one activation, in authored order. The fields are private
/// and no accessor returns a mutable reference, so a bound value cannot change
/// after activation:
///
/// ```compile_fail
/// fn change(mut captures: quire_protocol::captures::ActivationCaptures) {
///     captures.occurrences.clear();
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ActivationCaptures {
    activation: Binding,
    boundary: TemporalBoundary,
    occurrences: Vec<CaptureOccurrence>,
}

impl ActivationCaptures {
    /// The activation.
    #[must_use]
    pub const fn activation(&self) -> &Binding {
        &self.activation
    }

    /// The inclusive activation boundary every value was taken at.
    #[must_use]
    pub const fn boundary(&self) -> TemporalBoundary {
        self.boundary
    }

    /// The occurrences, in authored order.
    #[must_use]
    pub fn occurrences(&self) -> &[CaptureOccurrence] {
        &self.occurrences
    }
}

/// Why one capture could not be bound.
#[derive(Debug)]
pub enum CaptureFailure {
    /// QSL supplied nothing for the authored capture.
    Missing,
    /// QSL supplied a value for a capture the activation does not author, or
    /// more times than it authors it.
    Unauthored,
    /// The value was evaluated at another boundary than the activation's.
    WrongBoundary {
        /// The boundary of the supply.
        supplied: TemporalBoundary,
    },
    /// The capture has no value: QSL's outcome, unchanged.
    Undefined {
        /// The outcome.
        outcome: SlotOutcome,
    },
}

/// One capture that could not be bound.
#[derive(Debug)]
pub struct CaptureFault {
    /// Zero-based place in the authored order; for an unauthored supply, the
    /// place after the last authored capture.
    pub ordinal: usize,
    /// The authored capture, or the unauthored one a surplus supply names.
    pub capture: CheckedNodeId,
    /// Why.
    pub failure: CaptureFailure,
}

/// The activation is refused and no capture is bound.
#[derive(Debug)]
pub struct CaptureRefusal {
    /// Every capture that could not be bound, in authored order.
    pub faults: Vec<CaptureFault>,
}

/// The value of a completed slot outcome; any other outcome is returned
/// unchanged and is never read as a Boolean or defaulted.
fn completed(outcome: SlotOutcome) -> Result<Value, Box<SlotOutcome>> {
    match outcome {
        SlotOutcome::Holds => Ok(Value::Boolean(true)),
        SlotOutcome::DoesNotHold => Ok(Value::Boolean(false)),
        SlotOutcome::NotBoolean(value) => Ok(value),
        other @ (SlotOutcome::Unavailable { .. }
        | SlotOutcome::Undefined(_)
        | SlotOutcome::Refused(_)
        | SlotOutcome::Incomplete(_)
        | SlotOutcome::Family(_)) => Err(Box::new(other)),
    }
}

/// Bind the captures of `activation`, whose authored captures are `authored`
/// in order, from QSL's `supplied` evaluations at `boundary`. Each supply
/// answers the authored capture it names, whatever order it arrives in; the
/// occurrences are in authored order.
///
/// # Errors
///
/// [`CaptureRefusal`] naming every capture that is missing, evaluated at
/// another boundary or without a value, and every supply for a capture that
/// is not authored; none is bound then.
pub fn bind_captures(
    _pure: PureAssessment,
    activation: &Binding,
    boundary: TemporalBoundary,
    authored: &[CheckedNodeId],
    supplied: Vec<CaptureSupply>,
) -> Result<ActivationCaptures, CaptureRefusal> {
    let mut supplies: Vec<Option<CaptureSupply>> = supplied.into_iter().map(Some).collect();
    let mut occurrences = Vec::with_capacity(authored.len());
    let mut faults = Vec::new();
    for (ordinal, capture) in authored.iter().enumerate() {
        let fault = |failure| CaptureFault {
            ordinal,
            capture: capture.clone(),
            failure,
        };
        let Some(supply) = supplies
            .iter_mut()
            .find(|slot| {
                slot.as_ref()
                    .is_some_and(|supply| supply.capture == *capture)
            })
            .and_then(Option::take)
        else {
            faults.push(fault(CaptureFailure::Missing));
            continue;
        };
        if supply.boundary != boundary {
            faults.push(fault(CaptureFailure::WrongBoundary {
                supplied: supply.boundary,
            }));
            continue;
        }
        match completed(supply.evaluation) {
            Ok(value) => occurrences.push(CaptureOccurrence {
                id: CaptureOccurrenceId {
                    activation: activation.clone(),
                    ordinal,
                },
                capture: capture.clone(),
                value,
            }),
            Err(outcome) => faults.push(fault(CaptureFailure::Undefined { outcome: *outcome })),
        }
    }
    for (offset, surplus) in supplies.into_iter().flatten().enumerate() {
        faults.push(CaptureFault {
            ordinal: authored.len() + offset,
            capture: surplus.capture,
            failure: CaptureFailure::Unauthored,
        });
    }
    if faults.is_empty() {
        Ok(ActivationCaptures {
            activation: activation.clone(),
            boundary,
            occurrences,
        })
    } else {
        Err(CaptureRefusal { faults })
    }
}

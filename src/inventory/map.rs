// SPDX-License-Identifier: AGPL-3.0-or-later
//! A-error to B-refusal mapping, and the discriminating admission entry point.
//!
//! A owns its own error enums and B cannot extend them. This module is B's
//! discriminating layer *over* what A returns: every A refusal is classified
//! into a B-owned variant carrying typed context, and the one A code that
//! collapses several axes — `Invalid::Selection` — is resolved against B's own
//! independently selected inventory before it reaches the caller.
//!
//! The classification is derived from the cause, not from message text, and
//! every known A variant is named explicitly. A's enums are `#[non_exhaustive]`,
//! so the compiler *requires* one final arm per enum; that arm does not absorb
//! anything silently — it produces the loud, separately assertable
//! [`AdmissionRefusal::UnclassifiedSource`], and
//! `tc_006_classification_names_every_known_source_variant` fails if any
//! currently published A variant reaches it.

use serde_json::Value;

use quire_spec_language::protocol_artifact::{Exhaustion, NumberError};

use crate::digest::producer::{CrossDomainDigest, DigestSlot};
use crate::inventory::axis::SelectionAxis;
use crate::inventory::{DigestClassification, InventoryError, SelectedInventory};
use crate::{admit_and_link, wire, AdmittedPackage, Error, Expected, Invalid, Limits, Unsupported};

/// A B-owned admission refusal. Each variant is independently assertable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionRefusal {
    /// A digest was offered in a slot that does not accept its domain. Decided
    /// before A sees the offer, so a cross-domain value never reaches semantic
    /// evaluation and never degrades into a parse failure.
    CrossDomainDigest(CrossDomainDigest),
    /// A's collapsed selection refusal, resolved to the one axis that diverged.
    Selection(SelectionAxis),
    /// The offered bytes do not match the independently selected external seal.
    Seal,
    /// Any other A `Invalid` classification, retained uninterpreted.
    Invalid(Invalid),
    /// A recognised authority or feature with no supported interpretation.
    Unsupported(Unsupported),
    /// A's numeric refusal, retained exactly.
    Numeric(NumberError),
    /// A's closed JSON shape refusal, with A's original line and column.
    Malformed {
        /// Original byte-oriented line.
        line: usize,
        /// Original byte-oriented column.
        column: usize,
    },
    /// A's allocator refused a reservation already bounded by the limits.
    Allocation,
    /// A refused the next bounded step.
    Incomplete(Exhaustion),
    /// B's own selections could not be reduced to an inventory. This names B's
    /// inputs, not the offer, which has not been read when it is raised.
    UnusableSelections(InventoryError),
    /// An A variant published after this mapping was written.
    ///
    /// Reachable only through the arm `#[non_exhaustive]` forces. It is a
    /// refusal, never a success, and it retains A's original rendering.
    UnclassifiedSource {
        /// A's own `Debug` rendering, retained uninterpreted.
        source: String,
    },
}

/// Admit an offer and return a refusal B can discriminate.
///
/// The digest-domain guard runs first: a value from the wrong domain is refused
/// before A is asked to interpret the offer at all. Only then is the offer
/// handed to A's reader, and only then is A's refusal classified.
pub fn admit_discriminating(
    bytes: &[u8],
    expected: &Expected<'_>,
    limits: Limits,
) -> Result<AdmittedPackage, AdmissionRefusal> {
    let inventory =
        SelectedInventory::from_expected(expected).map_err(AdmissionRefusal::UnusableSelections)?;
    // The offer is read for the domain guard only within the caller's own
    // payload ceiling. Beyond it, A's accounting owns the refusal and B does no
    // unbounded work on untrusted bytes.
    let offer = (bytes.len() <= limits.bounded().payload_bytes)
        .then(|| serde_json::from_slice::<Value>(bytes).ok())
        .flatten();
    if let Some(offer) = offer.as_ref() {
        if let Some(refusal) = cross_domain_refusal(&inventory, offer) {
            return Err(AdmissionRefusal::CrossDomainDigest(refusal));
        }
    }
    match admit_and_link(bytes, expected, limits).into_result() {
        Ok(package) => Ok(package),
        Err(error) => Err(match classify_source(&error) {
            // A's one collapsed code is the only classification that needs B's
            // inventory to become a single axis.
            AdmissionRefusal::Selection(SelectionAxis::Undetermined) => {
                AdmissionRefusal::Selection(selection_axis(&inventory, offer.as_ref(), expected))
            }
            refused => refused,
        }),
    }
}

/// The first producer-reference slot whose offered digest is cross-domain.
///
/// Slot order is declared, not discovered, so the refusal is deterministic.
///
/// Scope: the three top-level selection references. Per-source, per-dependency
/// and per-model digest slots are not guarded here; they need the independently
/// selected per-entry inventory that the fixture-builder chunk lands, and
/// asserting them from this chunk would claim coverage it does not have.
fn cross_domain_refusal(inventory: &SelectedInventory, offer: &Value) -> Option<CrossDomainDigest> {
    for (slot, path) in [
        (DigestSlot::Contract, &["contract", "digest"][..]),
        (DigestSlot::Baseline, &["baseline", "digest"][..]),
        (
            DigestSlot::ProducerBinary,
            &["producer", "binary", "digest"][..],
        ),
    ] {
        let Some(offered) = member_str(offer, path) else {
            continue;
        };
        if let DigestClassification::CrossDomain(refusal) =
            inventory.classify_offered(slot, offered)
        {
            return Some(refusal);
        }
    }
    None
}

/// Classify one A refusal into a B-owned refusal, without any offer context.
///
/// Every A variant published at the pinned revision is named. The two final
/// arms exist only because A's enums are `#[non_exhaustive]`; each produces
/// [`AdmissionRefusal::UnclassifiedSource`] rather than folding an unknown
/// cause into a neighbouring code.
pub fn classify_source(error: &Error) -> AdmissionRefusal {
    match error {
        Error::Allocation => AdmissionRefusal::Allocation,
        Error::Json { line, column } => AdmissionRefusal::Malformed {
            line: *line,
            column: *column,
        },
        Error::Numeric(number) => AdmissionRefusal::Numeric(*number),
        Error::Incomplete(exhaustion) => AdmissionRefusal::Incomplete(exhaustion.clone()),
        Error::Unsupported(code) => classify_unsupported(*code),
        Error::Invalid(code) => classify_invalid(*code),
        other => AdmissionRefusal::UnclassifiedSource {
            source: format!("{other:?}"),
        },
    }
}

fn classify_unsupported(code: Unsupported) -> AdmissionRefusal {
    match code {
        Unsupported::Wire
        | Unsupported::Feature
        | Unsupported::Definition
        | Unsupported::Profile
        | Unsupported::ProducerCorrespondence
        | Unsupported::FamilyProof
        | Unsupported::Export => AdmissionRefusal::Unsupported(code),
        other => AdmissionRefusal::UnclassifiedSource {
            source: format!("{other:?}"),
        },
    }
}

fn classify_invalid(code: Invalid) -> AdmissionRefusal {
    match code {
        // Left unresolved here: naming the axis needs B's own inventory and the
        // offer, which `admit_discriminating` supplies.
        Invalid::Selection => AdmissionRefusal::Selection(SelectionAxis::Undetermined),
        Invalid::Seal => AdmissionRefusal::Seal,
        Invalid::Name
        | Invalid::StructuralInteger
        | Invalid::WrongNumericKind
        | Invalid::NumericDomain
        | Invalid::Inventory
        | Invalid::Order
        | Invalid::Duplicate
        | Invalid::Dependency
        | Invalid::Definition
        | Invalid::Model
        | Invalid::ForeignLocus
        | Invalid::Locus
        | Invalid::Reference
        | Invalid::Owner
        | Invalid::Scope
        | Invalid::Type
        | Invalid::Profile
        | Invalid::Call
        | Invalid::Binding
        | Invalid::Control
        | Invalid::Cycle
        | Invalid::Feature
        | Invalid::Canonical
        | Invalid::Encoding => AdmissionRefusal::Invalid(code),
        other => AdmissionRefusal::UnclassifiedSource {
            source: format!("{other:?}"),
        },
    }
}

/// Resolve A's collapsed selection refusal against B's own selections.
///
/// The comparison is against B's independently selected values, member by
/// member, in a declared order. It never reads a value out of the offer to
/// decide what should have been selected.
fn selection_axis(
    inventory: &SelectedInventory,
    offer: Option<&Value>,
    expected: &Expected<'_>,
) -> SelectionAxis {
    let Some(offer) = offer else {
        return SelectionAxis::Undetermined;
    };
    if let Some(axis) = reference_axis(
        inventory,
        offer,
        &["contract"],
        expected.contract,
        DigestSlot::Contract,
        &ReferenceAxes {
            identity: SelectionAxis::ContractIdentity,
            revision: SelectionAxis::ContractRevision,
            digest: SelectionAxis::ContractDigest,
            reference: SelectionAxis::ContractReference,
        },
    ) {
        return axis;
    }
    if let Some(axis) = producer_axis(offer, expected) {
        return axis;
    }
    if let Some(axis) = reference_axis(
        inventory,
        offer,
        &["baseline"],
        expected.baseline,
        DigestSlot::Baseline,
        &ReferenceAxes {
            identity: SelectionAxis::BaselineIdentity,
            revision: SelectionAxis::BaselineSelectionState,
            digest: SelectionAxis::BaselineDigest,
            reference: SelectionAxis::BaselineReference,
        },
    ) {
        return axis;
    }
    SelectionAxis::Undetermined
}

/// The four axes one artifact reference can diverge on.
struct ReferenceAxes {
    identity: SelectionAxis,
    revision: SelectionAxis,
    digest: SelectionAxis,
    reference: SelectionAxis,
}

fn reference_axis(
    inventory: &SelectedInventory,
    offer: &Value,
    path: &[&str],
    selected: &wire::ArtifactRef,
    slot: DigestSlot,
    axes: &ReferenceAxes,
) -> Option<SelectionAxis> {
    let offered = member(offer, path)?;
    let selected_value = serde_json::to_value(selected).ok()?;
    if offered == &selected_value {
        return None;
    }
    if member(offered, &["identity"]) != member(&selected_value, &["identity"]) {
        return Some(axes.identity);
    }
    if member(offered, &["revision"]) != member(&selected_value, &["revision"]) {
        return Some(axes.revision);
    }
    let offered_digest = member_str(offered, &["digest"])?;
    match inventory.classify_offered(slot, offered_digest) {
        DigestClassification::InSelectedDomain | DigestClassification::UnselectedSlot => {
            Some(axes.reference)
        }
        // A cross-domain value is refused before admission, so reaching this
        // arm means the digest diverges inside the slot's own domain.
        DigestClassification::Divergent | DigestClassification::CrossDomain(_) => Some(axes.digest),
    }
}

fn producer_axis(offer: &Value, expected: &Expected<'_>) -> Option<SelectionAxis> {
    let offered = member(offer, &["producer"])?;
    let selected = serde_json::to_value(expected.producer).ok()?;
    if offered == &selected {
        return None;
    }
    if member(offered, &["implementation"]) != member(&selected, &["implementation"]) {
        return Some(SelectionAxis::ProducerIdentity);
    }
    if member(offered, &["revision"]) != member(&selected, &["revision"]) {
        return Some(SelectionAxis::ProducerRevision);
    }
    Some(SelectionAxis::ProducerBinary)
}

fn member<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(key)?;
    }
    Some(current)
}

fn member_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    member(value, path)?.as_str()
}

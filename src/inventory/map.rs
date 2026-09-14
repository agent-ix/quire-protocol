// SPDX-License-Identifier: AGPL-3.0-or-later
//! A-error to B-refusal mapping, and the discriminating admission entry point.
//!
//! A owns its own error enums and B cannot extend them. This module is B's
//! discriminating layer *over* what A returns: every A refusal is retained as
//! A's complete typed [`Error`], and the one A code that collapses several axes
//! — `Invalid::Selection` — is additionally resolved against B's own
//! independently selected inventory before it reaches the caller.
//!
//! A's error enum is `#[non_exhaustive]`, so a downstream match must have a
//! catch-all arm. That arm retains the original value rather than formatting it
//! into a string. A future A variant therefore remains typed, keeps A's stable
//! [`Error::code`], and cannot be folded into a neighbouring B classification.

use serde_json::Value;

use crate::bounds::inventory::{BudgetDimension, SourceSite};
use crate::bounds::{SiteId, SiteKind};
use crate::digest::producer::{CrossDomainDigest, DigestSlot};
use crate::inventory::axis::DivergedSelection;
use crate::inventory::{DigestClassification, InventoryError, SelectedInventory};
use crate::{admit_and_link, wire, AdmittedPackage, Error, Expected, Invalid, Limits};

pub(crate) const ADMISSION_INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("admission/offer/decode"),
    "inventory::map::admit_discriminating",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);

/// A B-owned admission refusal. Each variant is independently assertable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionRefusal {
    /// A digest was offered in a slot that does not accept its domain. Decided
    /// before A sees the offer, so a cross-domain value never reaches semantic
    /// evaluation and never degrades into a parse failure.
    CrossDomainDigest(CrossDomainDigest),
    /// A's collapsed selection refusal, resolved to the one axis that diverged
    /// while retaining A's exact typed refusal.
    Selection {
        /// The component B's independently selected inventory found divergent.
        axis: DivergedSelection,
        /// A's original `Invalid::Selection` refusal, unchanged.
        source: Error,
    },
    /// Any A refusal that needs no B-owned selection-axis refinement.
    ///
    /// The complete owner type is retained, not its `Debug` or `Display`
    /// rendering. Callers can match every published nested family directly and
    /// can always read A's stable machine code through [`Self::source_code`].
    Source(Error),
    /// B's own selections could not be reduced to an inventory. This names B's
    /// inputs, not the offer, which has not been read when it is raised.
    UnusableSelections(InventoryError),
}

impl AdmissionRefusal {
    /// A's exact typed refusal, when this outcome came from A's reader.
    ///
    /// Pre-admission digest-domain checks and unusable B selections have no A
    /// refusal because A was deliberately not called.
    #[must_use]
    pub const fn source_error(&self) -> Option<&Error> {
        match self {
            Self::Selection { source, .. } | Self::Source(source) => Some(source),
            Self::CrossDomainDigest(_) | Self::UnusableSelections(_) => None,
        }
    }

    /// A's stable machine-readable code, without formatting the error.
    #[must_use]
    pub fn source_code(&self) -> Option<&'static str> {
        self.source_error().map(Error::code)
    }
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
        Err(error) => Err(match classify_source(error) {
            // A's one collapsed code is the only classification that needs B's
            // inventory to become a single axis.
            AdmissionRefusal::Selection { source, .. } => AdmissionRefusal::Selection {
                axis: selection_axis(&inventory, offer.as_ref(), expected),
                source,
            },
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

/// Retain one A refusal and add the only classification that needs offer
/// context later.
///
/// `Error` is `#[non_exhaustive]`, so the final arm is compulsory. It keeps the
/// owner type intact: when A publishes another family, that value remains
/// inspectable through [`AdmissionRefusal::source_error`] and its stable code
/// through [`AdmissionRefusal::source_code`] without a B update first.
pub fn classify_source(error: Error) -> AdmissionRefusal {
    match error {
        Error::Invalid(Invalid::Selection) => AdmissionRefusal::Selection {
            axis: DivergedSelection::Undetermined,
            source: Error::Invalid(Invalid::Selection),
        },
        source => AdmissionRefusal::Source(source),
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
) -> DivergedSelection {
    let Some(offer) = offer else {
        return DivergedSelection::Undetermined;
    };
    if let Some(axis) = reference_axis(
        inventory,
        offer,
        &["contract"],
        expected.contract,
        DigestSlot::Contract,
        &ReferenceAxes {
            identity: DivergedSelection::ContractIdentity,
            revision: DivergedSelection::ContractRevision,
            digest: DivergedSelection::ContractDigest,
            reference: DivergedSelection::ContractReference,
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
            identity: DivergedSelection::BaselineIdentity,
            revision: DivergedSelection::BaselineSelectionState,
            digest: DivergedSelection::BaselineDigest,
            reference: DivergedSelection::BaselineReference,
        },
    ) {
        return axis;
    }
    DivergedSelection::Undetermined
}

/// The four axes one artifact reference can diverge on.
struct ReferenceAxes {
    identity: DivergedSelection,
    revision: DivergedSelection,
    digest: DivergedSelection,
    reference: DivergedSelection,
}

fn reference_axis(
    inventory: &SelectedInventory,
    offer: &Value,
    path: &[&str],
    selected: &wire::ArtifactRef,
    slot: DigestSlot,
    axes: &ReferenceAxes,
) -> Option<DivergedSelection> {
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

fn producer_axis(offer: &Value, expected: &Expected<'_>) -> Option<DivergedSelection> {
    let offered = member(offer, &["producer"])?;
    let selected = serde_json::to_value(expected.producer).ok()?;
    if offered == &selected {
        return None;
    }
    if member(offered, &["implementation"]) != member(&selected, &["implementation"]) {
        return Some(DivergedSelection::ProducerIdentity);
    }
    if member(offered, &["revision"]) != member(&selected, &["revision"]) {
        return Some(DivergedSelection::ProducerRevision);
    }
    Some(DivergedSelection::ProducerBinary)
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

// SPDX-License-Identifier: AGPL-3.0-or-later
//! A-error to B-refusal mapping, and the discriminating admission entry point.
//!
//! A owns its own error enums and B cannot extend them. This module is B's
//! discriminating layer *over* what A returns: every A refusal is retained as
//! A's complete typed [`crate::Error`], and the one A code that collapses several axes
//! — `Invalid::Selection` — is additionally resolved against B's own
//! independently selected reference values before it reaches the caller.
//!
//! A's error enum is `#[non_exhaustive]`, so a downstream match must have a
//! catch-all arm. That arm retains the original value rather than formatting it
//! into a string. A future A variant therefore remains typed, keeps A's stable
//! [`crate::Error::code`], and cannot be folded into a neighbouring B classification.

use serde_json::Value;

use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::inventory::axis::DivergedSelection;
use crate::{intake, wire, AdmittedPackage, Error, Expected, Invalid, Limits};

pub(crate) const ADMISSION_INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("admission/offer/decode"),
    "inventory::map::admit_discriminating",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
pub(crate) const ADMISSION_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("admission/package/allocate"),
    "inventory::map::admit_discriminating",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// A B-owned admission refusal. Each variant is independently assertable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionRefusal {
    /// A's collapsed selection refusal, resolved to the one axis that diverged
    /// while retaining A's exact typed refusal.
    Selection {
        /// The independently selected reference component that diverged.
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
        }
    }

    /// A's stable machine-readable code, without formatting the error.
    #[must_use]
    pub fn source_code(&self) -> Option<&'static str> {
        self.source_error().map(Error::code)
    }
}

/// Admit an offer and return a refusal B can discriminate.
pub fn admit_discriminating(
    bytes: &[u8],
    expected: &Expected<'_>,
    limits: Limits,
) -> Result<AdmittedPackage, AdmissionRefusal> {
    match intake::v1::admit(bytes, expected, limits).into_result() {
        Ok(package) => Ok(package),
        Err(error) => {
            // Only inspect the bounded offer after the owner reader has refused
            // it, to refine its collapsed selection outcome.
            let offer = (bytes.len() <= limits.bounded().payload_bytes)
                .then(|| serde_json::from_slice::<Value>(bytes).ok())
                .flatten();
            Err(match classify_source(error) {
                // A's one collapsed code is the only classification that needs B's
                // inventory to become a single axis.
                AdmissionRefusal::Selection { source, .. } => AdmissionRefusal::Selection {
                    axis: selection_axis(offer.as_ref(), expected),
                    source,
                },
                refused => refused,
            })
        }
    }
}

/// Admit an offer under the shared QProtocol caller budget.
///
/// The complete offered byte population is charged before domain inspection,
/// JSON decoding, or the producer-owned reader begins. The producer reader's
/// immutable limits remain independently enforced by [`admit_discriminating`].
pub fn admit_discriminating_bounded(
    bytes: &[u8],
    expected: &Expected<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<AdmittedPackage, AdmissionRefusal> {
    resources.run_unit(
        UnitLabel::new("inventory::map::admit_discriminating_bounded"),
        |resources| admit_discriminating_bounded_unit(bytes, expected, limits, resources),
    )
}

fn admit_discriminating_bounded_unit(
    bytes: &[u8],
    expected: &Expected<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<AdmittedPackage, AdmissionRefusal> {
    let effective = limits.bounded();
    if let Err(error) = resources.charge_all_mixed(&[
        (
            ADMISSION_INPUT_SOURCE.id(),
            bytes.len(),
            Some(effective.payload_bytes),
        ),
        (ADMISSION_ALLOCATION_SOURCE.id(), 1, None),
    ]) {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(admit_discriminating(bytes, expected, effective))
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

/// Resolve A's collapsed selection refusal against B's own selected references.
///
/// The comparison is against B's independently selected values, member by
/// member, in a declared order. It never reads a value out of the offer to
/// decide what should have been selected.
fn selection_axis(offer: Option<&Value>, expected: &Expected<'_>) -> DivergedSelection {
    let Some(offer) = offer else {
        return DivergedSelection::Undetermined;
    };
    if let Some(axis) = reference_axis(
        offer,
        &["contract"],
        expected.contract,
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
        offer,
        &["baseline"],
        expected.baseline,
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
    offer: &Value,
    path: &[&str],
    selected: &wire::ArtifactRef,
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
    if member(offered, &["digest"]) != member(&selected_value, &["digest"]) {
        Some(axes.digest)
    } else {
        Some(axes.reference)
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

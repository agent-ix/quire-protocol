// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independently selected `Expected` inventory. Owner chunk: **C-04**.
//!
//! C-04 lands B's own independently selected inventory so that a selection
//! refusal can be attributed to a single axis instead of inheriting A's
//! collapsed `Invalid::Selection`.

/// Per-axis selection discriminator. Owner chunk: **C-04**.
///
/// Will own one B-owned code per axis: contract version, producer identity,
/// source revision, baseline-selection state, baseline digest, dependency
/// closure, profile identity, required feature, resealed bytes.
pub mod axis;

/// `quire_spec_language` `Error` to `ProtocolRefusal` mapping. Owner chunk: **C-05**.
///
/// Will own a total (catch-all-free) mapping carrying A's stage, A's locus or an
/// explicit `Unavailable`, the C-04 axis code, and A's original value retained
/// uninterpreted.
pub mod map;

use serde_json::Value;

use crate::digest::jcs::{JcsError, ResultDigest};
use std::fmt;

use crate::digest::producer::{
    algorithm_label, CrossDomainDigest, DigestDomain, DigestSlot, OfferedDomain, ProducerDigestRef,
    PRODUCER_ARTIFACT_LABEL,
};
use crate::{wire, Expected};

/// B's own selections could not be reduced to an inventory.
///
/// This is a refusal about B's inputs, never about the offer: the offer has not
/// been read when it is raised.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InventoryError {
    /// A selected reference has no RFC 8785 canonical form.
    Canonicalization(JcsError),
    /// A selected reference did not serialise through A's own wire shape.
    Serialization(String),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonicalization(error) => write!(formatter, "{error}"),
            Self::Serialization(error) => {
                write!(formatter, "selected reference did not serialise: {error}")
            }
        }
    }
}

impl std::error::Error for InventoryError {}

impl From<JcsError> for InventoryError {
    fn from(value: JcsError) -> Self {
        Self::Canonicalization(value)
    }
}

/// What B's inventory can say about a digest offered in a given slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DigestClassification {
    /// The offered value is B's own selected value for that slot's domain.
    InSelectedDomain,
    /// The offered value belongs to a domain the slot does not accept.
    CrossDomain(CrossDomainDigest),
    /// The offered value is in the slot's own domain and simply diverges.
    Divergent,
    /// The slot is not one this inventory holds a producer selection for.
    ///
    /// Explicit rather than folded into `Divergent`: a slot B never selected is
    /// a different fact from a value B selected and did not match.
    UnselectedSlot,
}

/// Refusal of a digest offered where a protocol-result digest belongs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultDigestRefusal {
    /// The offered value belongs to a producer-owned domain.
    CrossDomain(CrossDomainDigest),
    /// The result itself has no canonical form, so no comparison is possible.
    Uncanonicalizable(JcsError),
    /// The offered value is in the result domain and diverges from B's own.
    Divergent {
        /// The value as offered, retained uninterpreted.
        offered: String,
        /// B's own canonical value for the same result.
        selected: String,
    },
}

/// One selected reference, held in both domains that can name it.
///
/// Holding both is what makes cross-domain substitution decidable: the two
/// domains are textually indistinguishable, so only an inventory that already
/// knows the value of the *other* domain can tell them apart.
#[derive(Clone, Debug)]
struct SlotInventory {
    slot: DigestSlot,
    producer: ProducerDigestRef,
    result: ResultDigest,
}

/// B's independently selected inventory of digest references.
///
/// Built from the caller's `Expected` before any offer is read. The offer has
/// no authority to add, replace or reinterpret an entry.
#[derive(Clone, Debug)]
pub struct SelectedInventory {
    slots: Vec<SlotInventory>,
}

impl SelectedInventory {
    /// Build the inventory from the caller's independent selections.
    pub fn from_expected(expected: &Expected<'_>) -> Result<Self, InventoryError> {
        let selections = [
            (DigestSlot::Contract, expected.contract),
            (DigestSlot::Baseline, expected.baseline),
            (DigestSlot::ProducerBinary, &expected.producer.binary),
        ];
        let mut slots = Vec::with_capacity(selections.len());
        for (slot, reference) in selections {
            slots.push(SlotInventory {
                slot,
                producer: ProducerDigestRef::artifact_bytes(&reference.digest.to_string()),
                result: ResultDigest::of_result(&reference_value(reference)?)?,
            });
        }
        Ok(Self { slots })
    }

    fn slot(&self, slot: DigestSlot) -> Option<&SlotInventory> {
        self.slots.iter().find(|entry| entry.slot == slot)
    }

    /// Classify a digest offered in a producer-reference slot.
    ///
    /// The domain question is settled here, before any semantic evaluation of
    /// the offer, because a value from the wrong domain is not a wrong value in
    /// the right domain: it answers a different question entirely.
    pub fn classify_offered(&self, slot: DigestSlot, offered: &str) -> DigestClassification {
        let Some(entry) = self.slot(slot) else {
            return DigestClassification::UnselectedSlot;
        };
        if offered == entry.producer.as_producer_text() {
            return DigestClassification::InSelectedDomain;
        }
        if offered == entry.result.text() {
            return DigestClassification::CrossDomain(CrossDomainDigest {
                slot,
                selected: slot.selected_domain(),
                offered: OfferedDomain::Identified(DigestDomain::ProtocolResultJcs),
                value: offered.to_owned(),
            });
        }
        match algorithm_label(offered) {
            Some(PRODUCER_ARTIFACT_LABEL) => DigestClassification::Divergent,
            Some(label) => DigestClassification::CrossDomain(CrossDomainDigest {
                slot,
                selected: slot.selected_domain(),
                offered: OfferedDomain::ForeignLabel(label.to_owned()),
                value: offered.to_owned(),
            }),
            None => DigestClassification::CrossDomain(CrossDomainDigest {
                slot,
                selected: slot.selected_domain(),
                offered: OfferedDomain::Unlabelled,
                value: offered.to_owned(),
            }),
        }
    }

    /// Accept a digest offered where a protocol-result digest belongs.
    ///
    /// The reverse direction of the same rule: a producer reference copied into
    /// the consumer's result domain is refused, not recanonicalised.
    pub fn accept_result_digest(
        &self,
        result: &Value,
        offered: &str,
    ) -> Result<ResultDigest, ResultDigestRefusal> {
        let selected =
            ResultDigest::of_result(result).map_err(ResultDigestRefusal::Uncanonicalizable)?;
        if offered == selected.text() {
            return Ok(selected);
        }
        if let Some(entry) = self
            .slots
            .iter()
            .find(|entry| entry.producer.as_producer_text() == offered)
        {
            return Err(ResultDigestRefusal::CrossDomain(CrossDomainDigest {
                slot: DigestSlot::ProtocolResult,
                selected: DigestSlot::ProtocolResult.selected_domain(),
                offered: OfferedDomain::Identified(entry.producer.domain()),
                value: offered.to_owned(),
            }));
        }
        Err(ResultDigestRefusal::Divergent {
            offered: offered.to_owned(),
            selected: selected.text(),
        })
    }

    /// B's own result-domain value for a selected reference slot.
    ///
    /// Exposed so a test can construct the well-formed cross-domain offer that
    /// the guard above must refuse.
    pub fn result_domain_text(&self, slot: DigestSlot) -> Option<String> {
        self.slot(slot).map(|entry| entry.result.text())
    }
}

/// Serialise a selected reference through A's own published wire shape.
///
/// B adds no schema here: the members and their spellings are A's, obtained
/// from A's `Serialize` impl for the type B was given.
fn reference_value(reference: &wire::ArtifactRef) -> Result<Value, InventoryError> {
    serde_json::to_value(reference)
        .map_err(|error| InventoryError::Serialization(error.to_string()))
}

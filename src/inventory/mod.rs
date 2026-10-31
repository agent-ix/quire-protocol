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

/// Discriminating `quire_spec_language::Error` retention. Owner chunk: **C-05**.
///
/// Retains A's complete typed error and stable code, and adds C-04's selection
/// axis where A deliberately collapses several selection failures. Converting
/// that value into `ProtocolRefusal` remains gated on the selected diagnostic
/// catalog and typed-cause contract owned upstream; this module does not invent
/// either one.
pub mod map;

use crate::digest::producer::{
    algorithm_label, CrossDomainDigest, DigestSlot, OfferedDomain, ProducerDigestRef,
    PRODUCER_ARTIFACT_LABEL,
};
use crate::Expected;

/// What B's inventory can say about a digest offered in a given slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DigestClassification {
    /// The offered value is B's own selected value for that slot's domain.
    InSelectedDomain,
    /// The offered value belongs to a domain the slot does not accept.
    CrossDomain(CrossDomainDigest),
    /// The offered value is in the slot's own domain and simply diverges.
    Divergent,
}

/// B's independently selected inventory of digest references.
///
/// Built from the caller's `Expected` before any offer is read. The offer has
/// no authority to add, replace or reinterpret an entry.
#[derive(Clone, Debug)]
pub struct SelectedInventory {
    contract: ProducerDigestRef,
    baseline: ProducerDigestRef,
    producer_binary: ProducerDigestRef,
}

impl SelectedInventory {
    /// Build the inventory from the caller's independent selections.
    pub fn from_expected(expected: &Expected<'_>) -> Self {
        Self {
            contract: ProducerDigestRef::artifact_bytes(&expected.contract.digest.to_string()),
            baseline: ProducerDigestRef::artifact_bytes(&expected.baseline.digest.to_string()),
            producer_binary: ProducerDigestRef::artifact_bytes(
                &expected.producer.binary.digest.to_string(),
            ),
        }
    }

    /// Classify a digest offered in a producer-reference slot.
    ///
    /// The domain question is settled here, before any semantic evaluation of
    /// the offer, because a value from the wrong domain is not a wrong value in
    /// the right domain: it answers a different question entirely.
    pub fn classify_offered(&self, slot: DigestSlot, offered: &str) -> DigestClassification {
        let selected = match slot {
            DigestSlot::Contract => &self.contract,
            DigestSlot::Baseline => &self.baseline,
            DigestSlot::ProducerBinary => &self.producer_binary,
        };
        if offered == selected.as_producer_text() {
            return DigestClassification::InSelectedDomain;
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
}

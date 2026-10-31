// SPDX-License-Identifier: AGPL-3.0-or-later
//! Opaque producer-domain artifact digest references and their cross-domain guard.

use std::fmt;

/// The producer artifact digest label selected by protocol FR-015.
///
/// Result identities use structural keys and have no digest domain. This
/// label applies only to supplied artifact bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DigestDomain {
    /// `raw-artifact-digest`. Over the complete supplied bytes of an artifact,
    /// including layout. Producer-owned, copied exactly, never recanonicalised.
    RawArtifact,
}

impl DigestDomain {
    /// The only digest domain this repository selects.
    pub const ALL: [Self; 1] = [Self::RawArtifact];

    /// The published member label, spelled exactly as the vocabulary publishes it.
    pub const fn as_str(self) -> &'static str {
        "raw-artifact-digest"
    }

    /// The algorithm label a value in this domain is spelled with.
    ///
    /// Values from other labels remain foreign and are never interpreted as
    /// raw artifact digests.
    pub const fn algorithm_label(self) -> &'static str {
        "sha256"
    }
}

impl fmt::Display for DigestDomain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A reference slot: the place in the protocol a digest was offered for.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DigestSlot {
    /// The accepted compiled-protocol contract artifact reference.
    Contract,
    /// The accepted semantic baseline artifact reference.
    Baseline,
    /// The accepted producer binary artifact reference.
    ProducerBinary,
}

impl DigestSlot {
    /// The one domain this slot accepts. A slot never accepts two domains.
    pub const fn selected_domain(self) -> DigestDomain {
        match self {
            Self::Contract | Self::Baseline | Self::ProducerBinary => DigestDomain::RawArtifact,
        }
    }

    /// Stable slot name, for diagnostics only.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::Baseline => "baseline",
            Self::ProducerBinary => "producer-binary",
        }
    }
}

impl fmt::Display for DigestSlot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// What was actually offered in a slot, as far as B's inventory can identify it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OfferedDomain {
    /// An algorithm or domain label B does not select, copied verbatim.
    ForeignLabel(String),
    /// The value carries no algorithm label at all.
    Unlabelled,
}

/// A digest offered in a slot that does not accept its domain.
///
/// This is refused before any semantic evaluation of the offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossDomainDigest {
    /// Where the value was offered.
    pub slot: DigestSlot,
    /// The one domain that slot accepts.
    pub selected: DigestDomain,
    /// What was offered instead.
    pub offered: OfferedDomain,
    /// The offending value, retained uninterpreted.
    pub value: String,
}

impl fmt::Display for CrossDomainDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} slot accepts the {} domain; offered {}",
            self.slot,
            self.selected,
            match &self.offered {
                OfferedDomain::ForeignLabel(label) => label.as_str(),
                OfferedDomain::Unlabelled => "an unlabelled value",
            }
        )
    }
}

impl std::error::Error for CrossDomainDigest {}

/// An opaque producer-domain digest reference, copied byte-for-byte.
///
/// The type exposes the producer's exact text and its declared domain, and
/// nothing else. It offers no accessor that yields a value the result-domain
/// layer will accept, so a producer reference cannot be recanonicalised.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProducerDigestRef {
    text: String,
}

impl ProducerDigestRef {
    /// Copy a producer reference exactly.
    pub fn copy_exact(text: &str) -> Self {
        Self {
            text: text.to_owned(),
        }
    }

    /// Copy an exact-bytes artifact reference, the one domain whose
    /// producer-ownership is a compile-time fact, so no caller has to handle an
    /// impossible `None`.
    pub(crate) fn artifact_bytes(text: &str) -> Self {
        Self::copy_exact(text)
    }

    /// The producer's exact bytes, unchanged.
    pub fn as_producer_text(&self) -> &str {
        &self.text
    }
}

/// The algorithm label of a digest text, if it carries one.
///
/// The label is a spelling, not a domain. It identifies a value whose *label*
/// B does not select at all; it can never establish that a well-formed value
/// belongs to the selected domain.
pub fn algorithm_label(text: &str) -> Option<&str> {
    text.split_once(':').map(|(label, _)| label)
}

/// The one algorithm label the producer's artifact references are spelled with.
pub const PRODUCER_ARTIFACT_LABEL: &str = DigestDomain::RawArtifact.algorithm_label();

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Opaque producer-domain digest references and the cross-domain guard.
//!
//! Producer-owned source, model, config and artifact digests are copied exactly
//! and stay opaque. Nothing in this module feeds a producer reference into the
//! result-domain canonicaliser in [`crate::digest::jcs`]: there is no `From`,
//! no `Into`, and no trait impl that connects the two. Recanonicalising a
//! producer reference as an RFC 8785 protocol-result digest is therefore not
//! merely discouraged, it is unrepresentable.

use std::fmt;

/// A digest domain: the question a digest value answers.
///
/// Two values may share an algorithm label and still belong to different
/// domains, because the domain is fixed by *what was hashed*, never by the
/// spelling of the value. Cross-domain substitution is consequently invisible
/// to any syntactic check and must be decided against an independently selected
/// inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DigestDomain {
    /// Producer-owned: over the exact bytes of a producer artifact.
    ProducerArtifactBytes,
    /// Producer-owned: over the exact bytes of a producer source document.
    ProducerSource,
    /// Producer-owned: over the exact bytes of an admitted producer model.
    ProducerModel,
    /// Producer-owned: over the exact bytes of a producer configuration.
    ProducerConfig,
    /// Consumer-owned: over the RFC 8785/JCS canonical form of a protocol result.
    ProtocolResultJcs,
}

impl DigestDomain {
    /// Whether the domain is owned by the producer and therefore copied exactly.
    pub const fn is_producer_owned(self) -> bool {
        match self {
            Self::ProducerArtifactBytes
            | Self::ProducerSource
            | Self::ProducerModel
            | Self::ProducerConfig => true,
            Self::ProtocolResultJcs => false,
        }
    }

    /// Stable domain name, for diagnostics only; it is never parsed back.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProducerArtifactBytes => "producer-artifact-bytes",
            Self::ProducerSource => "producer-source",
            Self::ProducerModel => "producer-model",
            Self::ProducerConfig => "producer-config",
            Self::ProtocolResultJcs => "protocol-result-jcs",
        }
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
    /// A B-owned protocol-result record.
    ProtocolResult,
}

impl DigestSlot {
    /// The one domain this slot accepts. A slot never accepts two domains.
    pub const fn selected_domain(self) -> DigestDomain {
        match self {
            Self::Contract | Self::Baseline | Self::ProducerBinary => {
                DigestDomain::ProducerArtifactBytes
            }
            Self::ProtocolResult => DigestDomain::ProtocolResultJcs,
        }
    }

    /// Stable slot name, for diagnostics only.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::Baseline => "baseline",
            Self::ProducerBinary => "producer-binary",
            Self::ProtocolResult => "protocol-result",
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
    /// Identified by value against B's own inventory for the same object.
    Identified(DigestDomain),
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
                OfferedDomain::Identified(domain) => domain.as_str(),
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
    domain: DigestDomain,
    text: String,
}

impl ProducerDigestRef {
    /// Copy a producer reference exactly.
    ///
    /// Returns `None` for a consumer-owned domain: a protocol-result digest is
    /// not a producer reference and this constructor will not launder one.
    pub fn copy_exact(domain: DigestDomain, text: &str) -> Option<Self> {
        domain.is_producer_owned().then(|| Self {
            domain,
            text: text.to_owned(),
        })
    }

    /// Copy an exact-bytes artifact reference, the one domain whose
    /// producer-ownership is a compile-time fact, so no caller has to handle an
    /// impossible `None`.
    pub(crate) fn artifact_bytes(text: &str) -> Self {
        Self {
            domain: DigestDomain::ProducerArtifactBytes,
            text: text.to_owned(),
        }
    }

    /// The producer's declared domain.
    pub const fn domain(&self) -> DigestDomain {
        self.domain
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

/// The one algorithm label A's producer-owned artifact references are spelled with.
pub const PRODUCER_ARTIFACT_LABEL: &str = "sha256";

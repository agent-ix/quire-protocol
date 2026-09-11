// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001/FR-002: strict native compiled-protocol admission and linking.
//!
//! This crate deliberately owns no native parser, source frontend, wire decoder,
//! numeric interpretation, or formal profile. It exposes the accepted public
//! Rust reader from the A-owned `quire.compiled-protocol/1` contract through a
//! named B-consumer boundary.

#![forbid(unsafe_code)]

pub use quire_spec_language::protocol_artifact::wire;
pub use quire_spec_language::protocol_artifact::{
    AdmittedModel, AdmittedPackage, Candidate, Error, Expected, ExpectedDeclaration,
    ExpectedForeignSource, ExpectedSource, Invalid, Limits, Report, SuppliedDependency,
    Unsupported,
};
pub use quire_spec_language::ByteDigest;

/// Admit the exact bytes selected by the caller and validate their finite
/// references through A's public `quire.compiled-protocol/1` reader.
///
/// `expected` must be built before this call from independently selected source,
/// model, definition/dependency, contract, producer, and output identities. The
/// offered payload has no authority to select or repair any of those values.
/// Feature identifiers are checked by A's accepted in-contract allowlist: the
/// public reader does not expose a separate caller-selected feature-set input.
///
/// A successful result contains A's linked typed package. A refusal is returned
/// before this crate can expose a package for any failed seal, identity, feature,
/// numeric, or structural check.
pub fn admit_and_link(
    bytes: &[u8],
    expected: &Expected<'_>,
    limits: Limits,
) -> Report<AdmittedPackage> {
    quire_spec_language::protocol_artifact::read(bytes, expected, limits)
}

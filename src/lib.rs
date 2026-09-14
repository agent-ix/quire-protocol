// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001/FR-002: strict native compiled-protocol admission and linking.
//!
//! This crate deliberately owns no native parser, source frontend, wire decoder,
//! numeric interpretation, or formal profile. It exposes the accepted public
//! Rust reader from the A-owned `quire.compiled-protocol/1` contract through a
//! named B-consumer boundary.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Facade reservation (chunk C-00).
//
// Every module the planned chunk set needs is declared here in one commit so
// that no later PR has to edit this file to introduce its module (COLLISION-1).
// Each module below documents the chunk that owns its body; a module was
// declared here only so that its owner could fill it without touching the
// facade. Several owners have since done so, so this is a reservation record,
// not a claim that the modules are empty. After the reserving commit
// `src/lib.rs` changes only to append a re-export, in the module order
// established here.
// ---------------------------------------------------------------------------

/// Shared assessment inputs retained from producer-owned facts.
pub mod assessment;
/// Count, allocation and deadline guards. Owner chunk: C-29 (submodule: C-30).
pub mod bounds;
/// Claim disposition product type and registry. Owner chunk: C-22 (submodules: C-23, C-24).
pub mod claims;
/// Five-axis closure and outcome lattice. Owner chunk: C-15.
pub mod closure;
/// Compensation algebra, purity, activation and recovery. Owner chunks: C-16..C-19.
pub mod compensation;
/// Batch and settled-incremental conformance evaluation. Owner chunks: C-20, C-21.
pub mod conformance;
/// Producer-domain and result-domain digests. Owner chunks: C-10, C-11.
pub mod digest;
/// Protocol identity newtypes. Owner chunk: C-07.
pub mod ids;
/// Independently selected inventory and axis discrimination. Owner chunks: C-04, C-05.
pub mod inventory;
/// Finite linked-protocol graph and its located diagnostics. Owner chunks: C-12, C-13.
pub mod link;
/// B-owned typed refusal envelope. Owner chunk: C-01 (submodules: C-02, C-03).
pub mod refusal;
/// Seed, toolchain and corpus reproduction record. Owner chunk: C-31.
pub mod repro;
/// Canonical protocol result record. Owner chunk: C-26 (submodules: C-27, C-28).
pub mod result;

pub use quire_spec_language::protocol_artifact::v2;
pub use quire_spec_language::protocol_artifact::wire;
pub use quire_spec_language::protocol_artifact::{
    AdmittedModel, AdmittedPackage, Candidate, Error, Expected, ExpectedDeclaration,
    ExpectedForeignSource, ExpectedSource, Invalid, Limits, NumberComponent, NumberError, Report,
    SuppliedDependency, Unsupported,
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

/// Reads `bytes` as a `quire.compiled-protocol/2` package, checking it against
/// the selections in `expected` under `limits`.
///
/// Returns the admitted package, or a refusal naming the first check that
/// failed.
pub fn admit_and_link_v2(
    bytes: &[u8],
    expected: &v2::Expected<'_>,
    limits: Limits,
) -> Report<v2::AdmittedPackage> {
    v2::read(bytes, expected, limits)
}

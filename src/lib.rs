// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001/FR-002: strict native compiled-protocol admission and linking.
//!
//! This crate deliberately owns no native parser, source frontend, wire decoder,
//! numeric interpretation, or formal profile. It exposes the accepted public
//! Rust reader from the A-owned `quire.compiled-protocol/1` contract through a
//! named B-consumer boundary.

#![forbid(unsafe_code)]

// Lets in-crate semantic test modules retain the same absolute paths as their
// former integration-test location without exporting a second crate surface.
extern crate self as quire_protocol;

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
/// Consumer-authored evidence index for one pinned producer delivery.
pub mod delivery_index;
/// Canonical JCS encoding for structured protocol values.
pub mod digest;
/// Protocol identity newtypes. Owner chunk: C-07.
pub mod ids;
/// Version-explicit compiled-protocol admission boundaries.
pub mod intake;
/// Independently selected inventory and axis discrimination. Owner chunks: C-04, C-05.
pub mod inventory;
/// Finite linked-protocol graph and its located diagnostics. Owner chunks: C-12, C-13.
pub mod link;
/// Version-explicit full-assessment model capabilities.
pub mod model;
/// Closed public operation and claim outcome algebras.
pub mod outcome;
/// Exact request-bound producer-contract inventory for IT-002.
pub mod producer_contracts;
/// Source-bound projection of canonical protocol results to qverify E02.
pub mod projection;
/// B-owned typed refusal envelope. Owner chunk: C-01 (submodules: C-02, C-03).
pub mod refusal;
/// Seed, toolchain and corpus reproduction record. Owner chunk: C-31.
pub mod repro;
/// Canonical protocol result record. Owner chunk: C-26 (submodules: C-27, C-28).
pub mod result;
/// B-owned version-lock manifest resolution and exact-byte delivery guard.
pub mod version_lock;

pub use outcome::{ClaimOutcome, OperationOutcome};

pub use quire_spec_language::protocol_artifact::v2;
pub use quire_spec_language::protocol_artifact::wire;
pub use quire_spec_language::protocol_artifact::{
    AdmittedModel, AdmittedPackage, Candidate, Error, Expected, ExpectedDeclaration,
    ExpectedForeignSource, ExpectedSource, Invalid, Limits, NumberComponent, NumberError, Report,
    SuppliedDependency, Unsupported,
};
// ADR-011 §6.1 layer F (QSL-177): ByteDigest moved out of quire-spec-language's
// root crate into the qsl-foundation workspace crate it was extracted to.
pub use qsl_foundation::ByteDigest;

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
    intake::v1::admit(bytes, expected, limits)
}

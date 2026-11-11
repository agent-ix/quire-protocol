// SPDX-License-Identifier: AGPL-3.0-or-later
//! The protocol consumer of the checked package.
//!
//! This crate deliberately owns no native parser, source frontend, wire decoder,
//! numeric interpretation, or formal profile. It admits the checked-package
//! handoff through Contract IR's reader (FR-043), links a protocol clause over
//! QSL's read view (FR-044), evaluates compensation slots and temporal evidence
//! through QSL's checked-package evaluation entries (FR-045, FR-046), and keeps
//! the compensation state machine and the claim and closure vocabulary.

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

/// Count, allocation and deadline guards. Owner chunk: C-29 (submodule: C-30).
pub mod bounds;
pub mod checked_package;
/// Five-axis closure and outcome lattice. Owner chunk: C-15.
pub mod closure;
/// Compensation algebra, purity, activation and recovery. Owner chunks: C-16..C-19.
pub mod compensation;
/// Authored role, channel, join and loop declarations of a linked protocol.
pub mod declarations;
/// Consumer-authored evidence index for one pinned producer delivery.
pub mod delivery_index;
/// Exact-byte SHA-256 digest carriers.
pub mod digest;
pub mod evidence;
/// Protocol identity newtypes. Owner chunk: C-07.
pub mod ids;
pub mod link;
/// Closed public operation and claim outcome algebras.
pub mod outcome;
/// B-owned typed refusal envelope. Owner chunk: C-01 (submodules: C-02, C-03).
pub mod refusal;
/// Seed, toolchain and corpus reproduction record. Owner chunk: C-31.
pub mod repro;
pub mod slot;
/// B-owned version-lock manifest resolution and exact-byte delivery guard.
pub mod version_lock;

pub use digest::byte::ByteDigest;
pub use outcome::{ClaimOutcome, OperationOutcome};

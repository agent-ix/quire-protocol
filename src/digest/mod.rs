// SPDX-License-Identifier: AGPL-3.0-or-later
//! Digest-domain facade. Owner chunk: **C-00** (facade only).
//!
//! Two digest domains meet here and must never merge: producer-owned references
//! are copied byte-for-byte and stay opaque (G-DIGEST), while the single RFC
//! 8785 canonicalisation in this crate covers protocol results only.

/// Opaque producer-domain digest reference. Owner chunk: **C-10**.
///
/// Will expose the producer's exact bytes and declared domain tag, and will
/// offer no path — no `From`, `Into`, or trait impl — that feeds them into the
/// result-domain JCS layer.
pub mod producer;

/// The single RFC 8785 result-domain digest implementation. Owner chunk: **C-11**.
///
/// Will own the crate's only JCS canonicaliser, plus refusal of an unknown
/// result-contract version rather than a nearest-known interpretation.
pub mod jcs;

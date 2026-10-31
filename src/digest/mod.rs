// SPDX-License-Identifier: AGPL-3.0-or-later
//! Digest and canonical-encoding utilities. Producer-owned artifact digests
//! remain opaque; JCS is used only to encode structured protocol values.

/// Opaque producer-domain digest reference. Owner chunk: **C-10**.
///
/// Will expose the producer's exact bytes and declared domain tag, and will
/// offer no path — no `From`, `Into`, or trait impl — that feeds them into the
/// result-domain JCS layer.
pub mod producer;

/// RFC 8785 canonical encoding utilities.
pub mod jcs;

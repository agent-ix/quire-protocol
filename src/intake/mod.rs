// SPDX-License-Identifier: AGPL-3.0-or-later
//! Version-explicit compiled-protocol admission boundaries.
//!
//! The frozen root facade remains the narrow `/1` compatibility exception.
//! Full-assessment callers select a versioned module explicitly.

/// Frozen compiled-protocol version-1 compatibility admission.
pub mod v1;
/// Compiled-protocol version-2 admission for full assessment.
pub mod v2;

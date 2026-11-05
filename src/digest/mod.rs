// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical-encoding utilities for structured protocol values.

/// Exact-byte SHA-256 carrier for the FR-009 and FR-015 content digests.
pub mod byte;
/// RFC 8785 canonical encoding utilities.
pub mod jcs;

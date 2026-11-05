// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact-byte SHA-256 for the content digests FR-009 and FR-015 keep.
//!
//! Quire Spec Language no longer publishes a digest type (QSL-1261), so the
//! consumer holds the one carrier its own version-lock and result digests need.
//! It is not an integrity digest of a checked package (FR-047).

use std::fmt;

use sha2::{Digest, Sha256};

/// SHA-256 value with canonical lowercase, algorithm-prefixed text encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteDigest([u8; 32]);

impl ByteDigest {
    /// Hash precisely the supplied bytes, without parsing or normalization.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}

impl fmt::LowerHex for ByteDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

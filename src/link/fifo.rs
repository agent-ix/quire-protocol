// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact runtime FIFO partition keys.

use super::definition::{DefinitionIdentity, StaticChannelKey};

/// One runtime FIFO partition.
///
/// `V` stays the value owner's exact type. Protocol neither serializes nor
/// interprets it, so equality cannot silently approximate an evaluated value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FifoPartition<V> {
    channel: StaticChannelKey,
    value: V,
}

impl<V> FifoPartition<V> {
    /// Pair one exact static channel with one exact owner-typed value.
    #[must_use]
    pub(super) const fn new(channel: StaticChannelKey, value: V) -> Self {
        Self { channel, value }
    }

    /// Complete FR-288 definition identity owning the static channel.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.channel.definition
    }

    /// Declaration-local static channel index.
    #[must_use]
    pub const fn channel(&self) -> u32 {
        self.channel.channel
    }

    /// Borrow the exact owner-typed evaluated value.
    #[must_use]
    pub const fn value(&self) -> &V {
        &self.value
    }
}

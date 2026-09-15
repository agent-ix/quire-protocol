// SPDX-License-Identifier: AGPL-3.0-or-later
//! One immutable conformance input assembled from linked static and compensation facts.

use crate::compensation::registration::{Registration, Snapshot as CompensationSnapshot};
use crate::model::v2::{DefinitionIdentity, LinkedProtocol};

/// Why linked static and derived compensation state cannot form one kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// A registration names another complete linked definition.
    ForeignDefinition,
    /// A registration names no compensation definition in the linked graph.
    UnknownCompensation,
}

/// Non-retargetable conformance input for one assessment request.
///
/// This kernel retains the exact linked definition and derived registration
/// population. Batch and incremental frontends must consume this same value;
/// neither receives a separately assembled compensation model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kernel {
    definition: DefinitionIdentity,
    registrations: Vec<Registration>,
}

impl Kernel {
    /// Validates and retains one linked graph with its derived registrations.
    pub fn new(
        linked: &LinkedProtocol<'_>,
        compensation: &CompensationSnapshot,
    ) -> Result<Self, Refusal> {
        for registration in compensation.registrations() {
            if registration.key().definition() != linked.identity() {
                return Err(Refusal::ForeignDefinition);
            }
            if !linked
                .compensations()
                .iter()
                .any(|definition| definition.handle() == registration.key().compensation_handle())
            {
                return Err(Refusal::UnknownCompensation);
            }
        }
        Ok(Self {
            definition: linked.identity().clone(),
            registrations: compensation.registrations().to_vec(),
        })
    }

    /// Returns the exact linked static identity governing this kernel.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.definition
    }

    /// Returns replay-idempotent registrations in QSL declaration order.
    #[must_use]
    pub fn registrations(&self) -> &[Registration] {
        &self.registrations
    }
}

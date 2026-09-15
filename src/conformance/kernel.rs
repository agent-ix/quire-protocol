// SPDX-License-Identifier: AGPL-3.0-or-later
//! One immutable conformance input assembled from linked static and compensation facts.

use crate::compensation::registration::{Registration, Snapshot as CompensationSnapshot};
use crate::model::v2::{DefinitionIdentity, LinkedProtocol};
use quire_spec_language::protocol_artifact::wire::{Body, ControlOperation};

/// Why linked static and derived compensation state cannot form one kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// A registration names another complete linked definition.
    ForeignDefinition,
    /// A registration names no compensation definition in the linked graph.
    UnknownCompensation,
    /// A selected QSL control table is too large for its published `u32` handle domain.
    UnrepresentableControl,
}

/// One exact QSL control retained as a conformance obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedObligation {
    handle: u32,
    operation: ControlOperation,
}

impl LinkedObligation {
    /// Returns the exact declaration-local QSL control coordinate.
    #[must_use]
    pub const fn handle(&self) -> u32 {
        self.handle
    }

    /// Returns QSL's complete typed control operation.
    #[must_use]
    pub const fn operation(&self) -> &ControlOperation {
        &self.operation
    }
}

/// Non-retargetable conformance input for one assessment request.
///
/// This kernel retains the exact linked definition and derived registration
/// population. Batch and incremental frontends must consume this same value;
/// neither receives a separately assembled compensation model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kernel {
    definition: DefinitionIdentity,
    obligations: Vec<LinkedObligation>,
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
        let Body::Protocol { controls, .. } = &linked.declaration().body else {
            unreachable!("LinkedProtocol only retains a QSL protocol declaration");
        };
        let mut obligations = Vec::with_capacity(controls.len());
        for (index, control) in controls.iter().enumerate() {
            let handle = u32::try_from(index).map_err(|_| Refusal::UnrepresentableControl)?;
            obligations.push(LinkedObligation {
                handle,
                operation: control.operation.clone(),
            });
        }
        Ok(Self {
            definition: linked.identity().clone(),
            obligations,
            registrations: compensation.registrations().to_vec(),
        })
    }

    /// Returns the exact linked static identity governing this kernel.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.definition
    }

    /// Returns every QSL control in exact declaration-table order.
    #[must_use]
    pub fn obligations(&self) -> &[LinkedObligation] {
        &self.obligations
    }

    /// Returns replay-idempotent registrations in QSL declaration order.
    #[must_use]
    pub fn registrations(&self) -> &[Registration] {
        &self.registrations
    }
}

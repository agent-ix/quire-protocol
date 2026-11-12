// SPDX-License-Identifier: AGPL-3.0-or-later
//! The link of an admitted QObs observation record to a checked node and a
//! workflow instance (QSpec FR-260-AC-5..9, FR-261).
//!
//! The record carries neither field. Both are derived from members it
//! carries: the declared binding identity, resolved through the assessment's
//! declared role mapping, gives the checked nodes the record observes, and
//! the FR-287 subject gives the workflow instance. Nothing here accepts a node
//! or an instance from the caller in place of the derivation.

use std::collections::BTreeMap;

use quire_contract_model::CheckedNodeId;
use quire_observation::authority::observation;

use crate::ids::WorkflowInstanceId;

/// The assessment's declared role mapping: each declared binding identity and
/// the checked nodes (events, compensations or responses) it resolves to. One
/// binding may be referenced by several compatible role mappings, so it
/// resolves to a set.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RoleMapping {
    nodes: BTreeMap<Box<str>, Vec<CheckedNodeId>>,
}

impl RoleMapping {
    /// A mapping that resolves no binding.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare that `binding` resolves to `nodes`, in addition to any nodes
    /// already declared for it.
    #[must_use]
    pub fn resolving(
        mut self,
        binding: &str,
        nodes: impl IntoIterator<Item = CheckedNodeId>,
    ) -> Self {
        self.nodes.entry(binding.into()).or_default().extend(nodes);
        self
    }

    fn resolve(&self, binding: &str) -> &[CheckedNodeId] {
        self.nodes.get(binding).map_or(&[], Vec::as_slice)
    }
}

/// Why an observation is not linked to the event node and workflow instance a
/// wait needs. The observation is then unavailable as that wait's `after`
/// observation; it is not repaired or guessed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unlinked {
    /// The record's declared binding identity resolves to no node.
    BindingUnresolved,
    /// The record observes other nodes than the wait's anchor.
    NotAnchorNode,
    /// The record's subject is another workflow instance.
    OtherInstance,
}

/// What an observation record offers for the link: its declared binding
/// identity and its FR-287 subject identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Basis {
    binding: Box<str>,
    subject: Box<[u8]>,
}

impl Basis {
    /// The members of `view` the link is derived from.
    #[must_use]
    pub fn of(view: &observation::View) -> Self {
        let payload = view.payload();
        Self {
            binding: payload.binding_identity().into(),
            subject: payload.subject().identity.into(),
        }
    }

    /// Verify that the record is an observation of `anchor` for `instance`.
    ///
    /// # Errors
    ///
    /// [`Unlinked`] naming the first member that does not link.
    pub fn verify(
        &self,
        mapping: &RoleMapping,
        anchor: &CheckedNodeId,
        instance: &WorkflowInstanceId,
    ) -> Result<(), Unlinked> {
        let nodes = mapping.resolve(&self.binding);
        if nodes.is_empty() {
            return Err(Unlinked::BindingUnresolved);
        }
        if !nodes.contains(anchor) {
            return Err(Unlinked::NotAnchorNode);
        }
        if *self.subject != *instance.identity_bytes() {
            return Err(Unlinked::OtherInstance);
        }
        Ok(())
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! The link of an admitted QObs observation record to a checked node and a
//! workflow instance (QSpec FR-260-AC-5, FR-260-AC-6, FR-260-AC-9, FR-261).
//!
//! The record carries neither field. Both are derived from members it
//! carries: the declared binding identity, resolved through the assessment's
//! declared role mapping, gives the checked nodes the record observes, and
//! the FR-287 subject gives the workflow instance. Nothing here accepts a node
//! or an instance from the caller in place of the derivation.

use std::collections::{BTreeMap, BTreeSet};

use quire_contract_model::CheckedNodeId;
use quire_observation::authority::observation;
use quire_observation::{AdmittedBundleKey, QualifiedSubject};

/// The assessment's declared role mapping: each declared binding identity and
/// the checked nodes (events, compensations or responses) it resolves to. One
/// binding may be referenced by several compatible role mappings, so it
/// resolves to a set.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RoleMapping {
    nodes: BTreeMap<Box<str>, BTreeSet<CheckedNodeId>>,
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

    fn resolve(&self, binding: &str) -> Option<&BTreeSet<CheckedNodeId>> {
        self.nodes.get(binding).filter(|nodes| !nodes.is_empty())
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
    /// The record's FR-287 subject key (authority, kind and identity) is not
    /// the waiting instance's.
    OtherInstance,
}

/// What an observation record offers for the link: its declared binding
/// identity and its complete FR-287 subject key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Basis {
    binding: Box<str>,
    authority: AdmittedBundleKey,
    kind: Box<[u8]>,
    identity: Box<[u8]>,
}

impl Basis {
    /// The members of `view` the link is derived from.
    #[must_use]
    pub fn of(view: &observation::View) -> Self {
        let payload = view.payload();
        let subject = payload.subject();
        Self {
            binding: payload.binding_identity().into(),
            authority: AdmittedBundleKey {
                bundle_identity: subject.authority.identity.to_owned(),
                bundle_revision: subject.authority.revision.clone(),
                digest: subject.authority.digest.clone(),
            },
            kind: subject.kind.into(),
            identity: subject.identity.into(),
        }
    }

    /// Verify that the record is an observation of `anchor` for the workflow
    /// instance whose FR-287 subject key is `instance`. The whole key must
    /// match: equal identity bytes under another authority or kind are another
    /// instance.
    ///
    /// # Errors
    ///
    /// [`Unlinked`] naming the first member that does not link.
    pub fn verify(&self, link: &Link, anchor: &CheckedNodeId) -> Result<(), Unlinked> {
        let instance = &link.subject;
        let nodes = link
            .mapping
            .resolve(&self.binding)
            .ok_or(Unlinked::BindingUnresolved)?;
        if !nodes.contains(anchor) {
            return Err(Unlinked::NotAnchorNode);
        }
        if self.authority != *instance.authority()
            || *self.kind != *instance.kind().as_bytes()
            || *self.identity != *instance.identity().as_bytes()
        {
            return Err(Unlinked::OtherInstance);
        }
        Ok(())
    }
}

/// What links observations to one wait: the assessment's declared role
/// mapping and the FR-287 subject key of the waiting workflow instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Link {
    mapping: RoleMapping,
    subject: QualifiedSubject,
}

impl Link {
    /// The link of observations resolved through `mapping` to the instance
    /// whose subject key is `subject`.
    #[must_use]
    pub const fn new(mapping: RoleMapping, subject: QualifiedSubject) -> Self {
        Self { mapping, subject }
    }

    /// The waiting instance's FR-287 subject key.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }
}

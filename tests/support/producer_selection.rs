// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent fixture oracle for producer request-selection metadata.
//!
//! Production callers provide this metadata before loading producer values;
//! these helpers deliberately inspect already-built test fixtures only to
//! create a discriminating expected digest for the post-resolution verifier.

use quire_observation::QualifiedSubject;
use quire_protocol::model::v2::RuntimeBindingSet;
use quire_protocol::producer_contracts::ProducerValueSelection;
use sha2::{Digest as _, Sha256};

struct FixtureHasher {
    digest: Sha256,
    consumed: usize,
    limit: usize,
}

impl FixtureHasher {
    fn new(limit: usize) -> Self {
        Self {
            digest: Sha256::new(),
            consumed: 0,
            limit,
        }
    }

    fn component(&mut self, bytes: &[u8]) -> Result<(), ()> {
        let next = 16usize
            .checked_add(bytes.len())
            .and_then(|value| self.consumed.checked_add(value))
            .ok_or(())?;
        if next > self.limit {
            return Err(());
        }
        self.digest.update(
            u128::try_from(bytes.len())
                .expect("Rust slice lengths fit u128")
                .to_be_bytes(),
        );
        self.digest.update(bytes);
        self.consumed = next;
        Ok(())
    }

    fn finish(self) -> ([u8; 32], usize) {
        (self.digest.finalize().into(), self.consumed)
    }
}

fn authority_subject(hasher: &mut FixtureHasher, subject: &QualifiedSubject) -> Result<(), ()> {
    let authority = subject.authority();
    for component in [
        authority.bundle_identity.as_bytes(),
        authority.bundle_revision.namespace.as_bytes(),
        authority.bundle_revision.value.as_bytes(),
        authority.digest.label.as_bytes(),
        authority.digest.value.as_bytes(),
        subject.kind().as_bytes(),
        subject.identity().as_bytes(),
    ] {
        hasher.component(component)?;
    }
    Ok(())
}

#[allow(
    dead_code,
    reason = "not every integration fixture selects a standalone subject"
)]
pub(crate) fn subject(
    subject: &QualifiedSubject,
    limit: usize,
) -> Result<ProducerValueSelection, ()> {
    let mut hasher = FixtureHasher::new(limit);
    authority_subject(&mut hasher, subject)?;
    let (digest, framed_bytes) = hasher.finish();
    Ok(ProducerValueSelection::new(digest, framed_bytes))
}

pub(crate) fn runtime(
    runtime: &RuntimeBindingSet,
    limit: usize,
) -> Result<ProducerValueSelection, ()> {
    runtime_parts(runtime, limit)
        .map(|(digest, framed_bytes)| ProducerValueSelection::new(digest, framed_bytes))
}

pub(crate) fn runtime_parts(
    runtime: &RuntimeBindingSet,
    limit: usize,
) -> Result<([u8; 32], usize), ()> {
    let mut hasher = FixtureHasher::new(limit);
    hasher.component(runtime.target().identity.as_bytes())?;
    hasher.component(runtime.target().revision.as_bytes())?;
    hasher.component(&runtime.target().digest)?;
    for binding in runtime.bindings() {
        hasher.component(&binding.slot().get().to_be_bytes())?;
        if let Some(value) = binding.subject_value() {
            hasher.component(b"subject")?;
            authority_subject(&mut hasher, value)?;
        } else if let Some(value) = binding.relationship_value() {
            hasher.component(b"relationship")?;
            hasher.component(value.declaration_identity().as_bytes())?;
            let authority = value.authority();
            for component in [
                authority.bundle_identity.as_bytes(),
                authority.bundle_revision.namespace.as_bytes(),
                authority.bundle_revision.value.as_bytes(),
                authority.digest.label.as_bytes(),
                authority.digest.value.as_bytes(),
                value.interface_version().as_bytes(),
                value.identity().as_bytes(),
            ] {
                hasher.component(component)?;
            }
            authority_subject(&mut hasher, value.source())?;
            authority_subject(&mut hasher, value.target())?;
        } else {
            panic!("closed runtime fixture has one owner variant")
        }
    }
    Ok(hasher.finish())
}

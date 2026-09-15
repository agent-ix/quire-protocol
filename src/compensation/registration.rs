// SPDX-License-Identifier: AGPL-3.0-or-later
//! Derived compensation-registration keys over owner-published identities.
//!
//! A registration has no standalone protocol-generated identity. Its key is
//! the complete linked static definition, QSL's declaration-local compensation
//! handle, and QObs's exact established forward-effect subject. This module
//! deliberately retains those values instead of serialising, hashing, or
//! replacing them with the legacy standalone identity newtypes.

use quire_observation::QualifiedSubject;

use crate::model::v2::{CompensationRegistrationInput, DefinitionIdentity};

/// The complete FR-233 registration key for one admitted compensation pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationKey {
    definition: DefinitionIdentity,
    compensation_handle: u32,
    forward_effect: QualifiedSubject,
}

impl RegistrationKey {
    /// Returns the complete linked FR-288 definition identity.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.definition
    }

    /// Returns QSL's exact declaration-local compensation handle.
    #[must_use]
    pub const fn compensation_handle(&self) -> u32 {
        self.compensation_handle
    }

    /// Returns QObs's exact authority-qualified established forward effect.
    #[must_use]
    pub const fn forward_effect(&self) -> &QualifiedSubject {
        &self.forward_effect
    }
}

/// One replay-idempotent derived compensation registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    key: RegistrationKey,
}

impl Registration {
    /// Returns the complete derived registration key.
    #[must_use]
    pub const fn key(&self) -> &RegistrationKey {
        &self.key
    }

    pub(crate) fn from_input(
        definition: &DefinitionIdentity,
        input: &CompensationRegistrationInput<'_>,
    ) -> Self {
        Self {
            key: RegistrationKey {
                definition: definition.clone(),
                compensation_handle: input.compensation_handle(),
                forward_effect: input.effect().subject().clone(),
            },
        }
    }
}

/// The registration state derived for one non-retargetable assessment request.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Snapshot {
    registrations: Vec<Registration>,
}

impl Snapshot {
    /// Derives registrations from exact QSL/QObs pairs.
    ///
    /// Equal complete keys are exact replay of one registration. A distinct
    /// compensation handle or a distinct QObs effect remains a distinct key.
    #[must_use]
    pub fn from_inputs(
        definition: &DefinitionIdentity,
        inputs: &[CompensationRegistrationInput<'_>],
    ) -> Self {
        let mut registrations = Vec::with_capacity(inputs.len());
        for input in inputs {
            let registration = Registration::from_input(definition, input);
            if !registrations.contains(&registration) {
                registrations.push(registration);
            }
        }
        Self { registrations }
    }

    /// Returns every distinct derived registration in QSL declaration order.
    #[must_use]
    pub fn registrations(&self) -> &[Registration] {
        &self.registrations
    }

    pub(crate) fn ensure(
        &mut self,
        definition: &DefinitionIdentity,
        input: &CompensationRegistrationInput<'_>,
    ) -> (Registration, bool) {
        let candidate = Registration::from_input(definition, input);
        if let Some(existing) = self
            .registrations
            .iter()
            .find(|registration| **registration == candidate)
        {
            return (existing.clone(), false);
        }
        self.registrations.push(candidate.clone());
        (candidate, true)
    }

    pub(crate) fn find(
        &self,
        definition: &DefinitionIdentity,
        compensation_handle: u32,
    ) -> Option<&Registration> {
        self.registrations.iter().find(|registration| {
            registration.key.definition == *definition
                && registration.key.compensation_handle == compensation_handle
        })
    }
}

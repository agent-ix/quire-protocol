// SPDX-License-Identifier: AGPL-3.0-or-later
//! Constructor-checked capability boundary for pure compensation assessment.

/// I/O capability forbidden from every compensation assessment path.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Capability {
    /// Network access.
    Network,
    /// Storage access.
    Storage,
    /// Telemetry emission.
    Telemetry,
    /// Business-operation invocation.
    BusinessOperation,
    /// Compensating-operation invocation.
    CompensationOperation,
}

/// Read-only declared capability inventory.
///
/// Implementations may also expose action methods to their test harness, but
/// QProtocol receives only this inspection interface and cannot invoke them.
pub trait CapabilityInventory {
    /// Complete capabilities admitted by the assessment surface.
    fn capabilities(&self) -> &[Capability];
}

/// One forbidden capability at the pure-assessment boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Refusal {
    capability: Capability,
}

impl Refusal {
    /// Exact forbidden capability.
    #[must_use]
    pub const fn capability(self) -> Capability {
        self.capability
    }
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "forbidden compensation capability: {:?}",
            self.capability
        )
    }
}

impl std::error::Error for Refusal {}

/// Constructor-private witness that an assessment surface admits no I/O.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PureAssessment {
    private: (),
}

impl PureAssessment {
    /// Inspect the complete declared surface before any assessment runs.
    pub fn checked(inventory: &dyn CapabilityInventory) -> Result<Self, Refusal> {
        if let Some(capability) = inventory.capabilities().first().copied() {
            return Err(Refusal { capability });
        }
        Ok(Self { private: () })
    }
}

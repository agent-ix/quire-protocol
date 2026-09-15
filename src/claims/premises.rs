// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact producer-byte and premise retention for claim dispatch.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::bounds::inventory::{BudgetDimension, SourceSite};
use crate::bounds::{SiteId, SiteKind};

pub(crate) const INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("claims/premise/retain"),
    "claims::premises::PremiseSet",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);

/// One producer-authored component, retained byte-for-byte.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Component {
    /// Exact producer identity bytes.
    pub identity: Box<[u8]>,
    /// Exact admitted producer bytes.
    pub bytes: Box<[u8]>,
}

impl Component {
    /// Retain identity and document bytes without normalization.
    #[must_use]
    pub fn retained(identity: impl Into<Box<[u8]>>, bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            identity: identity.into(),
            bytes: bytes.into(),
        }
    }
}

/// Exact selected analysis backend.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BackendSelection {
    /// Backend identity bytes.
    pub identity: Box<[u8]>,
    /// Backend contract/version bytes.
    pub version: Box<[u8]>,
}

impl BackendSelection {
    /// Retain an independently selected backend and version.
    #[must_use]
    pub fn retained(identity: impl Into<Box<[u8]>>, version: impl Into<Box<[u8]>>) -> Self {
        Self {
            identity: identity.into(),
            version: version.into(),
        }
    }
}

/// Adopted FR-280 assumption state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AssumptionState {
    /// `met`.
    Met,
    /// `not-met`.
    NotMet,
    /// `unknown`.
    Unknown,
    /// `not-assessed`.
    NotAssessed,
}

/// One exact assumption plus its declared dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assumption {
    /// Exact assumption identity and source bytes.
    pub component: Component,
    /// Explicit state. Absence is a defect, never a default.
    pub state: Option<AssumptionState>,
    /// Exact assumption identities this assumption depends upon.
    pub dependencies: Vec<Box<[u8]>>,
}

/// Derived-artifact evidence checked by its producer-owner authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedArtifact {
    /// Exact derived artifact identity and bytes.
    pub artifact: Component,
    /// Exact selected source tuple in owner-declared order.
    pub source_tuple: Vec<Component>,
}

/// Owner seam for validating a derived-artifact identity against its tuple.
///
/// QProtocol never reconstructs a producer's identity algorithm. It asks the
/// selected owner implementation whether the retained tuple and identity agree.
pub trait CorrespondenceAuthority {
    /// Whether `artifact` is the exact owner-derived artifact for its tuple.
    fn validates(&self, artifact: &DerivedArtifact) -> bool;
}

/// Every premise retained by one claim request and its returned disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiseSet {
    /// Exact admitted fragment.
    pub admitted_fragment: Component,
    /// Exact selected backend.
    pub backend: BackendSelection,
    /// Exact assumptions in authored order.
    pub assumptions: Vec<Assumption>,
    /// Exact communication premises in authored order.
    pub communication: Vec<Component>,
    /// Exact visibility premises in authored order.
    pub visibility: Vec<Component>,
    /// Exact environment premises in authored order.
    pub environment: Vec<Component>,
    /// Exact source identities and source bytes in authored order.
    pub sources: Vec<Component>,
    /// Optional derived-artifact correspondence.
    pub derived_artifact: Option<DerivedArtifact>,
}

/// Premise axis carrying a defect.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PremiseAxis {
    /// Admitted fragment.
    AdmittedFragment,
    /// Selected backend identity/version.
    Backend,
    /// Assumption identity, state, or dependency graph.
    Assumption,
    /// Communication premise.
    Communication,
    /// Visibility premise.
    Visibility,
    /// Environment premise.
    Environment,
    /// Source identity or bytes.
    Source,
    /// Derived-artifact correspondence.
    DerivedArtifact,
}

/// Exact premise defect kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PremiseDefectKind {
    /// Required component is absent.
    Missing,
    /// Identity or producer bytes are empty.
    Empty,
    /// An identity is repeated within one ordered population.
    Duplicate,
    /// An assumption dependency names no retained assumption.
    DanglingDependency,
    /// Assumption dependencies are circular.
    CircularDependency,
    /// Backend-returned evidence differs from the exact request premises.
    /// Derived identity does not validate against the exact selected tuple.
    CorrespondenceMismatch,
}

/// One independently assertable premise defect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiseDefect {
    /// Defective premise axis.
    pub axis: PremiseAxis,
    /// Exact defect kind.
    pub kind: PremiseDefectKind,
    /// Exact affected identity bytes when present.
    pub identity: Option<Box<[u8]>>,
}

/// Validate every exact premise without defaulting or reconstructing one.
pub(crate) fn validate(
    premises: &PremiseSet,
    correspondence: Option<&dyn CorrespondenceAuthority>,
) -> Vec<PremiseDefect> {
    let mut defects = Vec::new();
    validate_component(
        &premises.admitted_fragment,
        PremiseAxis::AdmittedFragment,
        &mut defects,
    );
    if premises.backend.identity.is_empty() || premises.backend.version.is_empty() {
        defects.push(PremiseDefect {
            axis: PremiseAxis::Backend,
            kind: PremiseDefectKind::Empty,
            identity: (!premises.backend.identity.is_empty())
                .then(|| premises.backend.identity.clone()),
        });
    }
    validate_assumptions(&premises.assumptions, &mut defects);
    validate_population(
        &premises.communication,
        PremiseAxis::Communication,
        &mut defects,
    );
    validate_population(&premises.visibility, PremiseAxis::Visibility, &mut defects);
    validate_population(
        &premises.environment,
        PremiseAxis::Environment,
        &mut defects,
    );
    validate_population(&premises.sources, PremiseAxis::Source, &mut defects);
    if let Some(derived) = &premises.derived_artifact {
        validate_component(
            &derived.artifact,
            PremiseAxis::DerivedArtifact,
            &mut defects,
        );
        validate_population(
            &derived.source_tuple,
            PremiseAxis::DerivedArtifact,
            &mut defects,
        );
        if correspondence.is_none_or(|authority| !authority.validates(derived)) {
            defects.push(PremiseDefect {
                axis: PremiseAxis::DerivedArtifact,
                kind: PremiseDefectKind::CorrespondenceMismatch,
                identity: Some(derived.artifact.identity.clone()),
            });
        }
    }
    defects
}

fn validate_component(component: &Component, axis: PremiseAxis, defects: &mut Vec<PremiseDefect>) {
    if component.identity.is_empty() || component.bytes.is_empty() {
        defects.push(PremiseDefect {
            axis,
            kind: PremiseDefectKind::Empty,
            identity: (!component.identity.is_empty()).then(|| component.identity.clone()),
        });
    }
}

fn validate_population(values: &[Component], axis: PremiseAxis, defects: &mut Vec<PremiseDefect>) {
    if values.is_empty() {
        defects.push(PremiseDefect {
            axis,
            kind: PremiseDefectKind::Missing,
            identity: None,
        });
        return;
    }
    let mut identities = BTreeSet::new();
    for value in values {
        validate_component(value, axis, defects);
        if !identities.insert(value.identity.as_ref()) {
            defects.push(PremiseDefect {
                axis,
                kind: PremiseDefectKind::Duplicate,
                identity: Some(value.identity.clone()),
            });
        }
    }
}

fn validate_assumptions(assumptions: &[Assumption], defects: &mut Vec<PremiseDefect>) {
    let mut by_identity = BTreeMap::new();
    for (index, assumption) in assumptions.iter().enumerate() {
        validate_component(&assumption.component, PremiseAxis::Assumption, defects);
        if assumption.state.is_none() {
            defects.push(PremiseDefect {
                axis: PremiseAxis::Assumption,
                kind: PremiseDefectKind::Missing,
                identity: Some(assumption.component.identity.clone()),
            });
        }
        if by_identity
            .insert(assumption.component.identity.as_ref(), index)
            .is_some()
        {
            defects.push(PremiseDefect {
                axis: PremiseAxis::Assumption,
                kind: PremiseDefectKind::Duplicate,
                identity: Some(assumption.component.identity.clone()),
            });
        }
    }
    let mut indegree = vec![0usize; assumptions.len()];
    let mut successors = vec![Vec::new(); assumptions.len()];
    for (dependent, assumption) in assumptions.iter().enumerate() {
        let mut local = BTreeSet::new();
        for dependency in &assumption.dependencies {
            if !local.insert(dependency.as_ref()) {
                defects.push(PremiseDefect {
                    axis: PremiseAxis::Assumption,
                    kind: PremiseDefectKind::Duplicate,
                    identity: Some(dependency.clone()),
                });
                continue;
            }
            let Some(predecessor) = by_identity.get(dependency.as_ref()).copied() else {
                defects.push(PremiseDefect {
                    axis: PremiseAxis::Assumption,
                    kind: PremiseDefectKind::DanglingDependency,
                    identity: Some(dependency.clone()),
                });
                continue;
            };
            let Some(degree) = indegree[dependent].checked_add(1) else {
                defects.push(PremiseDefect {
                    axis: PremiseAxis::Assumption,
                    kind: PremiseDefectKind::CircularDependency,
                    identity: None,
                });
                return;
            };
            indegree[dependent] = degree;
            successors[predecessor].push(dependent);
        }
    }
    let mut ready: VecDeque<_> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect();
    let mut visited = 0usize;
    while let Some(index) = ready.pop_front() {
        let Some(next_visited) = visited.checked_add(1) else {
            defects.push(PremiseDefect {
                axis: PremiseAxis::Assumption,
                kind: PremiseDefectKind::CircularDependency,
                identity: None,
            });
            return;
        };
        visited = next_visited;
        for successor in &successors[index] {
            let Some(degree) = indegree[*successor].checked_sub(1) else {
                defects.push(PremiseDefect {
                    axis: PremiseAxis::Assumption,
                    kind: PremiseDefectKind::CircularDependency,
                    identity: None,
                });
                return;
            };
            indegree[*successor] = degree;
            if indegree[*successor] == 0 {
                ready.push_back(*successor);
            }
        }
    }
    if visited != assumptions.len() {
        defects.push(PremiseDefect {
            axis: PremiseAxis::Assumption,
            kind: PremiseDefectKind::CircularDependency,
            identity: None,
        });
    }
}

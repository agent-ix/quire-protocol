// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded decision-support and direct-correction lineage validation.

use std::collections::{BTreeMap, BTreeSet};

use crate::bounds::inventory::{BudgetDimension, SourceSite};
use crate::bounds::{SiteId, SiteKind};

use super::record::{CorrectionInput, DecisionSupport, DependenceRelation, Limits, Usage};
use super::{ErrorCode, Refusal, Result};

pub(crate) const LINEAGE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/lineage/traverse"),
    "result::lineage::validate",
    BudgetDimension::Lineage,
    SiteKind::Traversal,
);

pub(crate) fn validate_support(records: &[DecisionSupport], limits: Limits) -> Result<()> {
    let usage = Usage {
        support_records: records.len(),
        lineage_nodes: records.len(),
        ..Usage::default()
    };
    if records.len() > limits.max_support_records || records.len() > limits.max_lineage_nodes {
        return Err(Refusal::assembly(
            ErrorCode::ResourceIncomplete,
            "decision-support lineage exceeds its effective ceiling",
            usage,
            None,
        ));
    }
    let mut by_key = BTreeMap::new();
    for (index, record) in records.iter().enumerate() {
        if record.oracle_identity.is_empty() || record.observation_identity.is_empty() {
            return invalid("decision support contains an empty exact identity", usage);
        }
        let key = (
            record.oracle_identity.as_str(),
            record.observation_identity.as_str(),
        );
        if by_key.insert(key, index).is_some() {
            return invalid("decision support contains a duplicate record key", usage);
        }
        match record.relation {
            DependenceRelation::Independent | DependenceRelation::Unknown
                if record.predecessor.is_none() && record.dependence_group.is_none() => {}
            DependenceRelation::Shared
                if record.predecessor.is_none()
                    && record
                        .dependence_group
                        .as_deref()
                        .is_some_and(|value| !value.is_empty()) => {}
            DependenceRelation::Derived
                if record.predecessor.is_some() && record.dependence_group.is_none() => {}
            _ => {
                return invalid(
                    "dependence relation has missing or foreign relation fields",
                    usage,
                )
            }
        }
    }
    for (start, record) in records.iter().enumerate() {
        if record.relation != DependenceRelation::Derived {
            continue;
        }
        let mut visited = BTreeSet::new();
        let mut current = start;
        loop {
            if !visited.insert(current) {
                return invalid("decision-support lineage contains a cycle", usage);
            }
            if visited.len() > limits.max_lineage_nodes {
                return Err(Refusal::assembly(
                    ErrorCode::ResourceIncomplete,
                    "decision-support walk exceeds its effective ceiling",
                    Usage {
                        lineage_nodes: visited.len(),
                        ..usage
                    },
                    None,
                ));
            }
            let current_record = &records[current];
            if current_record.relation != DependenceRelation::Derived {
                break;
            }
            let Some(predecessor) = &current_record.predecessor else {
                return invalid("derived decision support has no predecessor", usage);
            };
            let key = (
                predecessor.oracle_identity.as_str(),
                predecessor.observation_identity.as_str(),
            );
            let Some(next) = by_key.get(&key).copied() else {
                return invalid(
                    "derived decision support names a missing predecessor",
                    usage,
                );
            };
            if next == current {
                return invalid("decision-support lineage contains a self edge", usage);
            }
            current = next;
        }
    }
    Ok(())
}

pub(crate) fn validate_correction(
    correction: Option<CorrectionInput<'_>>,
    limits: Limits,
) -> Result<()> {
    let Some(correction) = correction else {
        return Ok(());
    };
    let usage = Usage {
        lineage_nodes: 1,
        ..Usage::default()
    };
    if limits.max_lineage_nodes < 1 {
        return Err(Refusal::assembly(
            ErrorCode::ResourceIncomplete,
            "correction lineage exceeds its effective ceiling",
            usage,
            None,
        ));
    }
    if correction.contradicted_premise_identity.is_empty()
        || correction.corrected_input_identity.is_empty()
        || correction.corrected_input_identity
            == correction
                .predecessor
                .document
                .snapshot
                .provenance
                .input_identity
    {
        return invalid(
            "correction must name a retained premise and a different exact input",
            usage,
        );
    }
    let snapshot = &correction.predecessor.document.snapshot;
    let retained = snapshot
        .settlement
        .supporting_fact_identities
        .iter()
        .chain(snapshot.settlement.supporting_progress_identities.iter())
        .chain(
            snapshot
                .global_premises
                .iter()
                .map(|premise| &premise.identity),
        )
        .any(|identity| identity == correction.contradicted_premise_identity);
    if !retained {
        return invalid(
            "correction names no premise retained by its direct predecessor",
            usage,
        );
    }
    Ok(())
}

fn invalid<T>(detail: &'static str, usage: Usage) -> Result<T> {
    Err(Refusal::assembly(
        ErrorCode::LineageInvalid,
        detail,
        usage,
        None,
    ))
}

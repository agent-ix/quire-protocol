// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact execution/branch/workflow premise-set validation for FR-247.

use std::collections::BTreeSet;

use crate::assessment::{GlobalConformanceInput, GlobalPremise};
use crate::closure::GlobalConformanceClosure;

use super::record::{Claim, ClaimKind, Limits, Usage};
use super::{ErrorCode, Refusal, Result};

pub(crate) fn validate(
    claim: &Claim,
    global: &GlobalConformanceInput,
    limits: Limits,
) -> Result<()> {
    let usage = Usage {
        premises: global.premises.len().max(global.required_premises.len()),
        ..Usage::default()
    };
    if usage.premises > limits.max_premises {
        return Err(Refusal::new(
            ErrorCode::ResourceIncomplete,
            "global premise population exceeds its effective ceiling",
            usage,
            None,
        ));
    }
    let is_global = claim.kind == ClaimKind::GlobalConformance;
    match (is_global, global.state) {
        (false, GlobalConformanceClosure::NotRequired)
            if global.required_premises.is_empty() && global.premises.is_empty() =>
        {
            return Ok(())
        }
        (false, GlobalConformanceClosure::NotRequired) => {
            return Err(Refusal::new(
                ErrorCode::GlobalPremiseMismatch,
                "not-required cannot carry a global premise",
                usage,
                None,
            ));
        }
        (false, _) => {
            return Err(Refusal::new(
                ErrorCode::GlobalPremiseMismatch,
                "a non-global claim must carry not-required",
                usage,
                None,
            ));
        }
        (true, GlobalConformanceClosure::NotRequired) => {
            return Err(Refusal::new(
                ErrorCode::GlobalPremiseMismatch,
                "a global claim cannot carry not-required",
                usage,
                None,
            ));
        }
        (true, _) => {}
    }
    if global.required_premises.is_empty()
        || global.required_premises != global.premises
        || !unique(&global.required_premises)
    {
        return Err(Refusal::new(
            ErrorCode::GlobalPremiseMismatch,
            "global closure premises are missing, duplicated, reordered, or cross-wired",
            usage,
            None,
        ));
    }
    let kinds: BTreeSet<_> = global.premises.iter().map(|premise| premise.kind).collect();
    if kinds.len() != 3 {
        return Err(Refusal::new(
            ErrorCode::GlobalPremiseMismatch,
            "global closure requires execution, branch, and workflow premises",
            usage,
            None,
        ));
    }
    Ok(())
}

fn unique(premises: &[GlobalPremise]) -> bool {
    let mut keys = BTreeSet::new();
    premises.iter().all(|premise| {
        !premise.identity.is_empty()
            && !premise.scope_identity.is_empty()
            && !premise.authority_identity.is_empty()
            && keys.insert((
                premise.kind,
                premise.identity.as_str(),
                premise.scope_identity.as_str(),
                premise.authority_identity.as_str(),
            ))
    })
}

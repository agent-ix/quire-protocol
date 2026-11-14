// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-171: a choice whose owner, or a channel whose sender, is no reference
//! never reaches the link: Contract IR's reader refuses the package at
//! admission (FR-043), so the view refusals the link carries for them are
//! unreachable from an admitted package (FR-051, FR-052).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod projection;
    pub mod realizability;
}

use ix_trace_rs::trace;
use quire_contract_model::{CheckedPackageDispatchResult, CheckedPackageRefusalCode};
use serde_json::Value;
use support::projection::{CANCELLED, PICK};
use support::realizability::Realizability;

/// Replace the argument `name`'s value with a text literal, in every node
/// that has all of `with`. The wire stays valid; the member is no reference.
fn spoil_argument(wire: &mut Value, name: &str, with: &[&str]) {
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut dropped = 0;
    for node in nodes {
        let Some(arguments) = node["body"]["arguments"].as_array_mut() else {
            continue;
        };
        let has = |arguments: &[Value], wanted: &str| {
            arguments
                .iter()
                .any(|argument| argument["name"].as_str() == Some(wanted))
        };
        if with.iter().all(|wanted| has(arguments, wanted)) {
            let mut gone = Vec::new();
            for argument in arguments
                .iter_mut()
                .filter(|argument| argument["name"].as_str() == Some(name))
            {
                gone.push(argument["value"]["target"].clone());
                argument["value"] = serde_json::json!({
                    "term": "literal",
                    "type": {"ordinal": 0, "package": "package", "path": ["scalar_type", "text"]},
                    "value": "nobody",
                    "value_kind": "text"
                });
            }
            if let Some(dependencies) = node["dependencies"].as_array_mut() {
                dependencies.retain(|dependency| !gone.contains(dependency));
            }
            dropped += 1;
        }
    }
    assert!(dropped > 0, "no node carries {with:?}");
}

/// What Contract IR's reader says of the package with `mutate` applied.
fn read(fixture: &Realizability, mutate: impl FnOnce(&mut Value)) -> CheckedPackageDispatchResult {
    fixture.read_mutated(mutate)
}

#[test]
#[trace("TC-171", "FR-052-AC-5")]
fn a_choice_whose_owner_is_no_reference_is_refused_before_it_can_be_linked() {
    let fixture = Realizability::worked(PICK, CANCELLED);
    let outcome = read(&fixture, |wire| {
        spoil_argument(wire, "owner", &["owner", "visible", "cases"]);
    });
    assert!(
        matches!(
            &outcome,
            CheckedPackageDispatchResult::Refused(refusal)
                if refusal.code == CheckedPackageRefusalCode::MalformedWire
        ),
        "{outcome:?}"
    );
}

#[test]
#[trace("TC-171", "FR-051-AC-5")]
fn a_channel_whose_sender_is_no_reference_is_refused_before_it_can_be_linked() {
    let fixture = Realizability::worked(PICK, CANCELLED);
    let outcome = read(&fixture, |wire| {
        spoil_argument(wire, "from", &["from", "to", "carries"]);
    });
    assert!(
        matches!(
            &outcome,
            CheckedPackageDispatchResult::Refused(refusal)
                if refusal.code == CheckedPackageRefusalCode::MalformedWire
        ),
        "{outcome:?}"
    );
}

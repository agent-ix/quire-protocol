// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-150: linking over a typed read view of the checked package (FR-044),
//! the stage-2 subset (AC-1, AC-2, AC-4, AC-5's signature and compile-fail
//! half). AC-3, AC-6, AC-7 and AC-5's `BindingKind::Window` clause are QSL-1740.

mod support {
    pub mod handoff;
}

use std::collections::BTreeMap;
use std::path::Path;

use ix_trace_rs::trace;
use qsl_package::protocol_view::{causal_relation, protocol_view, ViewRefusal};
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedSemanticNodeV2,
};
use quire_protocol::checked_package::{admit, HandoffFiles};
use quire_protocol::link::{link, LinkedProtocol};
use quire_spec_language::checked_handoff::Corpus;
use serde_json::Value;
use support::handoff::{join_budget, written, Handoff};

fn admitted(handoff: &Handoff, bytes: &[u8]) -> Box<CheckedPackageV2> {
    let files = HandoffFiles {
        checked_package: bytes,
        selection: &handoff.files["selection.json"],
        identities: &handoff.files["identities.json"],
        equality: &handoff.files["equality.json"],
    };
    match admit(
        &files,
        &handoff.evidence,
        CheckedPackageReadLimits::bounded(),
        &mut join_budget(),
    ) {
        Ok(admitted) => Box::new(admitted.package().clone()),
        Err(refusal) => panic!("recipe R refused: {refusal}"),
    }
}

fn recipe(kind: &str) -> (Handoff, Box<CheckedPackageV2>) {
    let handoff = written(kind, Corpus::None);
    let package = admitted(&handoff, &handoff.files["checked-package.json"]);
    (handoff, package)
}

fn form_node<'p>(package: &'p CheckedPackageV2, form: &str) -> &'p CheckedSemanticNodeV2 {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| &*node.semantic_form == form)
        .unwrap_or_else(|| panic!("no {form} node"))
}

fn ids(nodes: &[&CheckedNodeId]) -> Vec<CheckedNodeId> {
    nodes.iter().map(|id| (*id).clone()).collect()
}

/// FR-044-AC-1: over recipe R's protocol the linked lists and edge set are
/// `protocol_view`'s node ids and `causal_relation`'s edges, value for value.
#[test]
#[trace("FR-044-AC-1", "TC-150")]
fn linked_lists_and_edges_equal_the_owners_view_and_relation() {
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    let linked = link(&package, &clause).expect("R's protocol links");
    let view = protocol_view(&package, &clause).expect("owner view");
    let relation = causal_relation(&package, &clause).expect("owner relation");

    assert_eq!(linked.clause(), view.clause());
    assert_eq!(linked.roles(), ids(view.roles()));
    assert_eq!(linked.channels(), ids(view.channels()));
    assert_eq!(linked.queues(), ids(view.queues()));
    assert_eq!(linked.controls(), ids(view.controls()));
    assert_eq!(linked.compensations(), ids(view.compensations()));
    assert_eq!(linked.requirements(), ids(view.requirements()));
    assert_eq!(linked.run(), view.run());
    assert_eq!(linked.finish(), view.finish());
    assert_eq!(linked.terminal(), view.terminal());
    assert_eq!(linked.scheduling(), view.scheduling());
    assert_eq!(linked.activation(), view.activation());
    assert_eq!(linked.edges(), relation.edges());

    // The owner's values for recipe R, so an empty pair of sides cannot agree.
    assert_eq!(
        (
            linked.roles().len(),
            linked.channels().len(),
            linked.queues().len(),
            linked.controls().len(),
            linked.compensations().len(),
            linked.requirements().len(),
            linked.activation().len(),
            linked.edges().len(),
        ),
        (2, 1, 1, 16, 2, 1, 1, 25)
    );
    assert!(linked.terminal().is_none());
    assert!(linked.scheduling().is_none());

    let mut kinds = BTreeMap::new();
    for edge in linked.edges() {
        *kinds.entry(format!("{:?}", edge.kind)).or_insert(0usize) += 1;
    }
    let kinds: Vec<_> = kinds.into_iter().collect();
    assert_eq!(
        kinds,
        [
            ("Branch".to_owned(), 6),
            ("Effect".to_owned(), 1),
            ("Join".to_owned(), 2),
            ("Message".to_owned(), 2),
            ("Sequence".to_owned(), 14),
        ]
    );
    assert!(
        linked.edges().windows(2).all(|pair| pair[0] < pair[1]),
        "the owner's strictly ascending order is kept"
    );
}

fn sources(dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).expect("src directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let text = std::fs::read_to_string(&path).expect("source");
            found.push((path.display().to_string(), text));
        }
    }
}

/// FR-044-AC-2: no source of the crate builds an `EdgeKind`, `CausalEdge` or
/// `PortRef`, and the edges the link layer returns come from
/// `causal_relation`.
#[test]
#[trace("FR-044-AC-2", "TC-150")]
fn no_source_constructs_a_causal_edge() {
    let mut found = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    assert!(found.len() > 10, "the scan reads the crate's sources");
    for (path, text) in &found {
        for line in text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
        {
            // `compensation/algebra.rs` owns a different `CausalEdge`: an
            // authored order over milestones, unrelated to QSL's relation.
            let own_milestone_edge = path.ends_with("src/compensation/algebra.rs");
            if own_milestone_edge {
                assert!(
                    !text.contains("protocol_view"),
                    "{path} imports the owner's edge"
                );
            }
            for forbidden in ["EdgeKind::", "CausalEdge {", "PortRef {", "Port::"] {
                if own_milestone_edge && forbidden == "CausalEdge {" {
                    continue;
                }
                assert!(
                    !line.contains(forbidden),
                    "{path} builds or names an edge piece: {line}"
                );
            }
        }
    }
    let link = found
        .iter()
        .find(|(path, _)| path.ends_with("src/link.rs"))
        .map(|(_, text)| text.as_str())
        .expect("the link source");
    assert!(link.contains("causal_relation(package, protocol)?"));
    assert!(link.contains("relation.edges()"));
}

/// FR-044-AC-4: an absent node, a `protocol_reference` carrier and a
/// non-protocol node each return the owner's typed refusal.
#[test]
#[trace("FR-044-AC-4", "TC-150")]
fn absent_carrier_and_non_protocol_nodes_return_the_owners_refusal() {
    let (handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();

    // Absent: a node of another recipe package that this package does not hold.
    let (_other_handoff, other) = recipe("fixed_sample");
    let absent = other
        .graph()
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .find(|id| {
            !package
                .graph()
                .nodes
                .iter()
                .any(|node| &node.node_id == *id)
        })
        .expect("a node only fixed_sample holds")
        .clone();
    let refused = link(&package, &absent).expect_err("absent node");
    assert_eq!(
        refused,
        ViewRefusal::UnknownProtocol {
            node: absent.clone()
        }
    );
    assert_eq!(Some(refused), protocol_view(&package, &absent).err());

    // Non-protocol: the model node that heads the graph.
    let model = package
        .graph()
        .nodes
        .iter()
        .find(|node| &*node.node_tag == "model")
        .expect("a model node")
        .node_id
        .clone();
    let refused = link(&package, &model).expect_err("non-protocol node");
    assert_eq!(
        refused,
        ViewRefusal::UnknownProtocol {
            node: model.clone()
        }
    );
    assert_eq!(Some(refused), protocol_view(&package, &model).err());

    // A protocol node of the carrier form: R's clause respelled as a
    // `protocol_reference`.
    let mut json: Value =
        serde_json::from_slice(&handoff.files["checked-package.json"]).expect("package JSON");
    let nodes = json["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let index = nodes
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause node");
    let source = nodes[index].clone();
    let mut carrier = source.clone();
    let target = source["node_id"].clone();
    carrier["node_id"] = serde_json::json!({
        "package": source["node_id"]["package"],
        "path": ["protocol", "protocol_reference", {"node": index}],
        "ordinal": 0
    });
    carrier["semantic_form"] = Value::from("protocol_reference");
    carrier["body"] = serde_json::json!({"term": "reference", "target": target});
    carrier["dependencies"] = serde_json::json!([target]);
    carrier["occurrences"] = serde_json::json!([{"ordinal": 0, "role": "expression"}]);
    for forbidden in ["declaration", "owner", "owner_model", "nominal_structure"] {
        carrier.as_object_mut().expect("object").remove(forbidden);
    }
    nodes.push(carrier.clone());
    let map = json["source_map"].as_array_mut().expect("source map");
    let mut entry = map
        .iter()
        .find(|entry| entry["node_id"] == target)
        .expect("the clause's source entry")
        .clone();
    entry["node_id"] = carrier["node_id"].clone();
    entry["ordinal"] = Value::from(0);
    entry["role"] = Value::from("expression");
    map.push(entry);
    let bytes = serde_json::to_vec(&json).expect("respelled package");
    let result = read_checked_package(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &handoff.evidence,
    );
    let CheckedPackageDispatchResult::AdmittedV2(carrier) = result else {
        panic!("the reader must admit the respelled package: {result:?}");
    };
    let carrier_id = form_node(&carrier, "protocol_reference").node_id.clone();
    let refused = link(&carrier, &carrier_id).expect_err("carrier");
    assert_eq!(
        refused,
        ViewRefusal::NotAProtocolClause {
            node: carrier_id.clone()
        }
    );
    assert_eq!(Some(refused), protocol_view(&carrier, &carrier_id).err());

    // The control: the unedited package still links the same node.
    assert!(link(&package, &clause).is_ok());
}

/// FR-044-AC-5: the link entry takes `CheckedPackageV2` and `CheckedNodeId`
/// and answers `LinkedProtocol` or the owner's `ViewRefusal`; the retired wire
/// types are compile-fail doctests of `quire_protocol::link`.
#[test]
#[trace("FR-044-AC-5", "TC-150")]
fn the_link_entry_names_the_checked_package_types() {
    let entry: fn(&CheckedPackageV2, &CheckedNodeId) -> Result<LinkedProtocol, ViewRefusal> = link;
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    assert_eq!(entry(&package, &clause).expect("links").clause(), &clause);
}

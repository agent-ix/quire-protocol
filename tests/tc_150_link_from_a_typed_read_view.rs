// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-150: linking over a typed read view of the checked package (FR-044),
//! the stage-2 subset (AC-1, AC-2, AC-4, AC-5's signature and compile-fail
//! half). AC-3, AC-6, AC-7 and AC-5's `BindingKind::Window` clause are QSL-1740.

mod support {
    pub mod handoff;
}

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ix_trace_rs::trace;
use qsl_package::protocol_view::{
    causal_relation, control_operation, protocol_view, temporal_binding, CausalEdge,
    ControlOperation, EdgeKind, ViewRefusal,
};
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm, ClockConfiguration,
};
use quire_protocol::checked_package::{admit, HandoffFiles};
use quire_protocol::link::{link, LinkRefusal, LinkedProtocol};
use quire_spec_language::checked_handoff::Corpus;
use serde_json::Value;
use support::handoff::{join_budget, request, written, written_request, written_source, Handoff};

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
#[trace("TC-150", "FR-044-AC-1")]
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
    assert_eq!(linked.requirement_terms().len(), view.requirements().len());
    assert_eq!(linked.temporal_bindings().len(), view.requirements().len());
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
#[trace("TC-150", "FR-044-AC-2")]
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
    assert!(link.contains("causal_relation(package, protocol)"));
    assert!(link.contains("relation.edges()"));
}

/// FR-044-AC-4: an absent node, a `protocol_reference` carrier and a
/// non-protocol node each return the owner's typed refusal.
#[test]
#[trace("TC-150", "FR-044-AC-4")]
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
        LinkRefusal::View(ViewRefusal::UnknownProtocol {
            node: absent.clone()
        })
    );
    assert_eq!(
        Some(refused),
        protocol_view(&package, &absent)
            .err()
            .map(LinkRefusal::View)
    );

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
        LinkRefusal::View(ViewRefusal::UnknownProtocol {
            node: model.clone()
        })
    );
    assert_eq!(
        Some(refused),
        protocol_view(&package, &model).err().map(LinkRefusal::View)
    );

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
        LinkRefusal::View(ViewRefusal::NotAProtocolClause {
            node: carrier_id.clone()
        })
    );
    assert_eq!(
        Some(refused),
        protocol_view(&carrier, &carrier_id)
            .err()
            .map(LinkRefusal::View)
    );

    // The control: the unedited package still links the same node.
    assert!(link(&package, &clause).is_ok());
}

/// FR-044-AC-6: recipe R links without a package window binding, and the
/// linker module does not import or look up a window view.
#[test]
#[trace("TC-150", "FR-044-AC-6")]
fn window_is_an_evaluation_input_not_a_package_binding() {
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    assert!(link(&package, &clause).is_ok());
    assert!(package
        .graph()
        .nodes
        .iter()
        .all(|node| node.semantic_form.as_ref() != "window"));

    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/link.rs"))
        .expect("link module source");
    assert!(
        !names_a_window_view(&source),
        "the linker names no window view or binding"
    );
}

/// Whether any code line of `source` (comments excluded) holds an identifier
/// that names a window: a path import, a braced import or a call alike.
fn names_a_window_view(source: &str) -> bool {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(|line| line.split(|c: char| !(c.is_alphanumeric() || c == '_')))
        .any(|token| token.to_ascii_lowercase().contains("window"))
}

/// FR-044-AC-6: the source scan sees a window view however it is imported.
#[test]
#[trace("TC-150", "FR-044-AC-6")]
fn the_window_scan_covers_path_and_braced_imports() {
    assert!(names_a_window_view(
        "use qsl_package::protocol_view::Window;"
    ));
    assert!(names_a_window_view(
        "use qsl_package::protocol_view::{protocol_view, Window, ViewRefusal};"
    ));
    assert!(names_a_window_view(
        "use qsl_package::protocol_view::{\n    causal_relation,\n    window_binding,\n};"
    ));
    assert!(!names_a_window_view(
        "// a window is an evaluation input\nuse qsl_package::protocol_view::{causal_relation, locus};"
    ));
}

/// FR-044 no-relabel: the linker never constructs one of the owner's
/// `ViewRefusal` values; it carries only the refusal QSL itself returned.
#[test]
#[trace("TC-150", "FR-044-AC-3")]
fn the_linker_constructs_no_owner_refusal() {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/link.rs"))
        .expect("link module source");
    for line in source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
    {
        assert!(
            !line.contains("ViewRefusal::"),
            "the linker builds or names an owner refusal: {line}"
        );
    }
}

/// FR-044-AC-7: recipe R carries the selected temporal content through the
/// link result, while its causal edges remain exactly QSL's authored relation.
#[test]
#[trace("TC-150", "FR-044-AC-7")]
fn recipe_r_retains_selected_clock_and_owner_causal_edges() {
    let (_event_handoff, event_package) = recipe("event_position");
    let event_clause = form_node(&event_package, "protocol_clause").node_id.clone();
    let event_link = link(&event_package, &event_clause).expect("event-position recipe links");
    let event_relation = causal_relation(&event_package, &event_clause).expect("owner relation");
    assert_eq!(event_link.edges(), event_relation.edges());
    assert!(!event_link.temporal_bindings().is_empty());
    assert_eq!(
        event_link.temporal_bindings().len(),
        event_link.requirement_terms().len()
    );
    let rows = event_package.temporal_bindings().expect("clock rows");
    for (requirement, selected) in event_link
        .requirements()
        .iter()
        .zip(event_link.temporal_bindings())
    {
        assert_eq!(
            temporal_binding(rows, requirement).expect("owner-selected clock"),
            selected
        );
    }
}

fn operation(package: &CheckedPackageV2, id: &CheckedNodeId) -> Option<ControlOperation> {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == id)
        .and_then(|node| control_operation(node).ok())
}

/// The nodes reachable from `start` along `edges`, `start` included, never
/// entering `stop`.
fn reach(
    edges: &[CausalEdge],
    start: &CheckedNodeId,
    stop: Option<&CheckedNodeId>,
) -> BTreeSet<CheckedNodeId> {
    let mut seen = BTreeSet::from([start.clone()]);
    let mut frontier = vec![start.clone()];
    while let Some(node) = frontier.pop() {
        for edge in edges.iter().filter(|edge| edge.from.node == node) {
            if Some(&edge.to.node) != stop && seen.insert(edge.to.node.clone()) {
                frontier.push(edge.to.node.clone());
            }
        }
    }
    seen
}

/// FR-044-AC-7 (FR-002-AC-1, AC-2): over recipe R's `parallel`, the two
/// concurrent branches keep distinct node identities, share no node, and each
/// has its own owner-supplied join edge; every listed node id is unique.
#[test]
#[trace("TC-150", "FR-044-AC-7")]
fn concurrent_branches_keep_distinct_identities_and_their_own_join_edge() {
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    let linked = link(&package, &clause).expect("recipe R links");

    for list in [
        linked.roles(),
        linked.channels(),
        linked.queues(),
        linked.controls(),
    ] {
        let unique: BTreeSet<_> = list.iter().collect();
        assert_eq!(unique.len(), list.len(), "no node id is listed twice");
    }

    let parallels: Vec<_> = linked
        .controls()
        .iter()
        .filter(|id| operation(&package, id) == Some(ControlOperation::Parallel))
        .collect();
    assert_eq!(parallels.len(), 1, "recipe R has one parallel control");
    let parallel = parallels[0];
    let bodies: Vec<_> = linked
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Branch && &edge.from.node == parallel)
        .map(|edge| edge.to.node.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(bodies.len(), 2, "two branch bodies enter the parallel");

    let first = reach(linked.edges(), &bodies[0], Some(parallel));
    let second = reach(linked.edges(), &bodies[1], Some(parallel));
    assert!(first.is_disjoint(&second), "branches share no node");

    let joins: Vec<_> = linked
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Join)
        .collect();
    assert_eq!(joins.len(), 2, "`join all` names both branches");
    assert_eq!(joins[0].to, joins[1].to, "both joins land on one port");
    assert_ne!(joins[0].from, joins[1].from);
    assert!(
        (first.contains(&joins[0].from.node) && second.contains(&joins[1].from.node))
            || (second.contains(&joins[0].from.node) && first.contains(&joins[1].from.node)),
        "one join edge leaves each branch"
    );
}

/// FR-044-AC-7 (FR-002-AC-2): the owner-supplied edges leave the two branches
/// unordered against each other, so every interleaving of their controls is
/// permitted, keep each branch's own message order, and reach the join.
#[test]
#[trace("TC-150", "FR-044-AC-7")]
fn parallel_branches_permit_every_interleaving_and_reach_the_join() {
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    let linked = link(&package, &clause).expect("recipe R links");
    let edges = linked.edges();
    let parallel = linked
        .controls()
        .iter()
        .find(|id| operation(&package, id) == Some(ControlOperation::Parallel))
        .expect("parallel control");
    let bodies: Vec<_> = edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Branch && &edge.from.node == parallel)
        .map(|edge| edge.to.node.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let sides = [
        reach(edges, &bodies[0], Some(parallel)),
        reach(edges, &bodies[1], Some(parallel)),
    ];
    let of_kind =
        |side: &BTreeSet<CheckedNodeId>, wanted: ControlOperation| -> Vec<CheckedNodeId> {
            side.iter()
                .filter(|id| operation(&package, id) == Some(wanted))
                .cloned()
                .collect()
        };
    let join_target = edges
        .iter()
        .find(|edge| edge.kind == EdgeKind::Join)
        .map(|edge| edge.to.node.clone())
        .expect("a join edge");

    let mut sends = Vec::new();
    for (index, side) in sides.iter().enumerate() {
        let send = of_kind(side, ControlOperation::Send);
        let receive = of_kind(side, ControlOperation::Receive);
        assert_eq!((send.len(), receive.len()), (1, 1), "branch {index}");
        assert!(
            edges.iter().any(|edge| edge.kind == EdgeKind::Message
                && edge.from.node == send[0]
                && edge.to.node == receive[0]),
            "the branch keeps its authored send-to-receive edge"
        );
        assert!(
            reach(edges, &send[0], None).contains(&join_target),
            "the branch's controls precede the join"
        );
        sends.push(send[0].clone());
    }
    // Neither branch's first control is ordered before the other's, so both
    // interleavings are permitted by the relation.
    assert!(!reach(edges, &sends[0], Some(parallel)).contains(&sends[1]));
    assert!(!reach(edges, &sends[1], Some(parallel)).contains(&sends[0]));
    for edge in edges {
        let from = sides.iter().position(|side| side.contains(&edge.from.node));
        let to = sides.iter().position(|side| side.contains(&edge.to.node));
        assert!(
            from.is_none() || to.is_none() || from == to,
            "no edge crosses between the concurrent branches: {edge:?}"
        );
    }
}

/// FR-044-AC-7 (FR-002-AC-4): reordering the equal-time input of the handoff
/// (the temporal clock selections) adds no causal edge: the edge set is the
/// same, and so is the selected binding of each required clause.
#[test]
#[trace("TC-150", "FR-044-AC-7")]
fn reordering_equal_time_input_adds_no_edge() {
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    let baseline = link(&package, &clause).expect("recipe R links");

    let mut asked = request("event_position", Corpus::None);
    assert!(
        asked.emit.temporal_selections.len() >= 2,
        "recipe R states several clock selections, so reversing them reorders"
    );
    asked.emit.temporal_selections.reverse();
    let reordered_handoff = written_request(asked);
    let reordered_package = admitted(
        &reordered_handoff,
        &reordered_handoff.files["checked-package.json"],
    );
    let reordered_clause = form_node(&reordered_package, "protocol_clause")
        .node_id
        .clone();
    let reordered = link(&reordered_package, &reordered_clause).expect("reordered input links");

    assert_eq!(baseline.edges(), reordered.edges());
    assert_eq!(baseline.requirements(), reordered.requirements());
    assert_eq!(baseline.temporal_bindings(), reordered.temporal_bindings());
}

/// FR-044-AC-7: changing the selected clock authority changes the linked
/// temporal binding while leaving the owner-supplied causal relation intact.
#[test]
#[trace("TC-150", "FR-044-AC-7")]
fn changing_the_selected_clock_changes_the_linked_temporal_binding() {
    let (_event_handoff, event_package) = recipe("event_position");
    let event_clause = form_node(&event_package, "protocol_clause").node_id.clone();
    let event_link = link(&event_package, &event_clause).expect("event-position recipe links");

    let mut asked = request("event_position", Corpus::None);
    for selection in &mut asked.emit.temporal_selections {
        if let ClockConfiguration::EventPosition { sequence_authority } = &mut selection.clock {
            *sequence_authority = "events-authority-alternative".into();
        }
    }
    let alternate_handoff = written_request(asked);
    let alternate_package = admitted(
        &alternate_handoff,
        &alternate_handoff.files["checked-package.json"],
    );
    let alternate_clause = form_node(&alternate_package, "protocol_clause")
        .node_id
        .clone();
    let alternate_link =
        link(&alternate_package, &alternate_clause).expect("alternate clock selection links");
    let alternate_relation =
        causal_relation(&alternate_package, &alternate_clause).expect("owner relation");

    assert_ne!(
        event_link.temporal_bindings(),
        alternate_link.temporal_bindings(),
        "the selected clock authority is retained in the linked result"
    );
    assert_eq!(event_link.edges(), alternate_link.edges());
    assert_eq!(alternate_link.edges(), alternate_relation.edges());
}

/// FR-044-AC-3: duplicate role authorities are detected from typed role terms.
#[test]
#[trace("TC-150", "FR-044-AC-3")]
fn duplicate_role_authority_is_refused_at_the_later_role() {
    let (handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    let view = protocol_view(&package, &clause).expect("owner view");
    let [first, second, ..] = view.roles() else {
        panic!("recipe R has at least two roles");
    };
    let mut json: Value =
        serde_json::from_slice(&handoff.files["checked-package.json"]).expect("package JSON");
    let nodes = json["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let first_wire = nodes
        .iter()
        .find(|node| node["node_id"] == serde_json::to_value(first).expect("role id"))
        .expect("first role wire node");
    let first_model = first_wire["body"]["arguments"]
        .as_array()
        .expect("role arguments")
        .iter()
        .find(|argument| argument["name"] == "model")
        .expect("model argument")["value"]["target"]
        .clone();
    let second_wire = nodes
        .iter_mut()
        .find(|node| node["node_id"] == serde_json::to_value(second).expect("role id"))
        .expect("second role wire node");
    let arguments = second_wire["body"]["arguments"]
        .as_array_mut()
        .expect("role arguments");
    let model_argument = arguments
        .iter_mut()
        .find(|argument| argument["name"] == "model")
        .expect("model argument");
    let old_model = model_argument["value"]["target"].clone();
    model_argument["value"]["target"] = first_model.clone();
    let dependencies = second_wire["dependencies"]
        .as_array_mut()
        .expect("role dependencies");
    dependencies.retain(|dependency| *dependency != old_model);
    if !dependencies.contains(&first_model) {
        dependencies.push(first_model.clone());
    }
    dependencies.sort_by_key(|dependency| serde_json::to_string(dependency).expect("node id"));

    let bytes = serde_json::to_vec(&json).expect("mutated package");
    let CheckedPackageDispatchResult::AdmittedV2(mutated) = read_checked_package(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &handoff.evidence,
    ) else {
        panic!("the checked reader admits two roles over one authority");
    };
    let refusal = link(&mutated, &clause).expect_err("duplicate authority must be refused");
    let LinkRefusal::DuplicateRoleAuthority {
        role,
        authority,
        locus,
    } = refusal
    else {
        panic!("expected duplicate authority refusal, got {refusal:?}");
    };
    assert_eq!(role, (*second).clone());
    assert_eq!(
        authority,
        serde_json::from_value(first_model).expect("authority id")
    );
    assert!(!locus.is_empty(), "the refusal retains available locations");
}

/// Recipe R with its first choice control respelled as a `repeat` whose
/// `maximum` argument is the given term value; admitted by the checked reader.
fn repeat_package(maximum: Value) -> (Box<CheckedPackageV2>, CheckedNodeId) {
    use qsl_package::protocol_view::{control_operation, ControlOperation};

    let (handoff, package) = recipe("event_position");
    let sequence = package
        .graph()
        .nodes
        .iter()
        .find(|node| control_operation(node) == Ok(ControlOperation::Choice))
        .expect("recipe R includes a choice control");
    let CheckedTerm::Application { arguments, .. } =
        package.term(&sequence.node_id).expect("sequence term")
    else {
        panic!("sequence has an application body");
    };
    let argument = |name: &str| {
        arguments
            .iter()
            .find_map(|term| match term {
                CheckedTerm::Binding { name: found, value } if found.as_ref() == name => {
                    Some(value.as_ref())
                }
                _ => None,
            })
            .expect("sequence argument")
    };
    let owner = serde_json::to_value(argument("owner")).expect("owner term");
    let visible = serde_json::to_value(argument("visible")).expect("visible term");
    let CheckedTerm::Aggregate { members } = argument("cases") else {
        panic!("choice cases are an aggregate");
    };
    let case_body = |index: usize| {
        let Some(CheckedTerm::Aggregate { members: case }) = members.get(index) else {
            panic!("choice has case {index}");
        };
        let Some(CheckedTerm::Binding { value, .. }) = case.iter().find(
            |member| matches!(member, CheckedTerm::Binding { name, .. } if name.as_ref() == "body"),
        ) else {
            panic!("choice case has a body");
        };
        let CheckedTerm::Reference { target } = value.as_ref() else {
            panic!("choice case body is a reference");
        };
        target
    };
    let target = case_body(0);
    let exhausted_target = case_body(1);
    let body = serde_json::json!({"term": "reference", "target": target});
    let type_id = |form: &str| {
        package
            .graph()
            .nodes
            .iter()
            .find(|node| &*node.node_tag == "scalar_type" && &*node.semantic_form == form)
            .map(|node| serde_json::to_value(&node.node_id).expect("type id"))
            .expect("built-in scalar type")
    };
    let integer_type = type_id("integer");
    let boolean_type = type_id("boolean");
    let sequence_id = serde_json::to_value(&sequence.node_id).expect("choice id");
    let mut json: Value =
        serde_json::from_slice(&handoff.files["checked-package.json"]).expect("package JSON");
    let nodes = json["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let repeat = nodes
        .iter_mut()
        .find(|node| node["node_id"] == sequence_id)
        .expect("choice wire node");
    repeat["body"]["operation"]["member"]["operator"] = Value::from("repeat");
    repeat["body"]["arguments"] = serde_json::json!([
        {"term": "binding", "name": "owner", "value": owner},
        {"term": "binding", "name": "visible", "value": visible},
        {"term": "binding", "name": "maximum", "value": {
            "term": "aggregate", "members": [{
                "term": "literal", "type": integer_type, "value_kind": "integer", "value": maximum
            }]
        }},
        {"term": "binding", "name": "proof", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "guard", "value": {
            "term": "literal", "type": boolean_type, "value_kind": "boolean", "value": true
        }},
        {"term": "binding", "name": "body", "value": body},
        {"term": "binding", "name": "exhausted", "value": {
            "term": "aggregate", "members": [{"term": "reference", "target": exhausted_target}]
        }}
    ]);
    fn collect_term_dependencies(term: &Value, found: &mut Vec<Value>) {
        if term["term"] == "reference" {
            found.push(term["target"].clone());
        }
        match term {
            Value::Array(items) => {
                for item in items {
                    collect_term_dependencies(item, found);
                }
            }
            Value::Object(items) => {
                for item in items.values() {
                    collect_term_dependencies(item, found);
                }
            }
            _ => {}
        }
    }
    let mut dependencies = Vec::new();
    collect_term_dependencies(&repeat["body"], &mut dependencies);
    dependencies.sort_by(|left, right| {
        let left: CheckedNodeId = serde_json::from_value(left.clone()).expect("node id");
        let right: CheckedNodeId = serde_json::from_value(right.clone()).expect("node id");
        left.cmp(&right)
    });
    dependencies.dedup();
    repeat["dependencies"] = Value::Array(dependencies);

    let bytes = serde_json::to_vec(&json).expect("mutated package");
    let result = read_checked_package(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &handoff.evidence,
    );
    let CheckedPackageDispatchResult::AdmittedV2(mutated) = result else {
        panic!("the checked reader admits a progressing repeat with maximum zero: {result:?}");
    };
    let sequence_id = sequence.node_id.clone();
    (mutated, sequence_id)
}

/// FR-044-AC-3: a zero maximum repeat receives a consumer refusal with its
/// source locus and no linked representation.
#[test]
#[trace("TC-150", "FR-044-AC-3")]
fn zero_maximum_repeat_is_refused_at_its_control() {
    let (mutated, control_id) = repeat_package(Value::from("0"));
    let clause = form_node(&mutated, "protocol_clause").node_id.clone();
    let refusal = link(&mutated, &clause).expect_err("zero maximum must be refused");
    let LinkRefusal::ZeroProgressIteration { control, locus } = refusal else {
        panic!("expected zero-progress refusal, got {refusal:?}");
    };
    assert_eq!(control, control_id);
    assert!(
        !locus.is_empty(),
        "the refusal retains source-map locations"
    );
}

/// FR-044-AC-3: two admitted channels with one route and payload but different
/// delivery premises are refused with both source loci.
#[test]
#[trace("TC-150", "FR-044-AC-3")]
fn conflicting_channel_premises_are_refused_at_both_nodes() {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/checked-handoff/unit-event-position.native");
    let source = std::fs::read_to_string(source_path).expect("recipe R source");
    let declaration = "channel Decisions from Provider to Service carries M::Notice\n        ordering unordered delivery at-least-once capacity 4 overflow reject;";
    let source = source.replace(
        declaration,
        &format!(
            "{declaration}\n    channel DecisionsAlternative from Provider to Service carries M::Notice\n        ordering unordered delivery unknown capacity 4 overflow reject;"
        ),
    );
    assert!(
        source.contains("channel DecisionsAlternative"),
        "the fixture receives a second channel declaration"
    );
    let handoff = written_source("event_position", &source, Corpus::None);
    let package = admitted(&handoff, &handoff.files["checked-package.json"]);
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    let view = protocol_view(&package, &clause).expect("owner view");
    assert_eq!(view.channels().len(), 2, "both declarations are admitted");

    let refusal = link(&package, &clause).expect_err("conflicting delivery premises");
    let LinkRefusal::ConflictingChannelPremise {
        channel,
        other,
        locus,
    } = refusal
    else {
        panic!("expected channel-premise refusal, got {refusal:?}");
    };
    assert!(view.channels().contains(&&channel));
    assert!(view.channels().contains(&&other));
    assert!(
        locus.len() >= 2,
        "both channels' available loci are retained"
    );
}

/// FR-044-AC-5: the link entry takes `CheckedPackageV2` and `CheckedNodeId`
/// and answers `LinkedProtocol` or `LinkRefusal`; the retired wire
/// types are compile-fail doctests of `quire_protocol::link`.
#[test]
#[trace("TC-150", "FR-044-AC-5")]
fn the_link_entry_names_the_checked_package_types() {
    let entry: fn(&CheckedPackageV2, &CheckedNodeId) -> Result<LinkedProtocol, LinkRefusal> = link;
    let (_handoff, package) = recipe("event_position");
    let clause = form_node(&package, "protocol_clause").node_id.clone();
    assert_eq!(entry(&package, &clause).expect("links").clause(), &clause);
}

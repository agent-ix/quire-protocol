// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked packages for the dynamic-roles, queues, joins and loops tests
//! (TC-050 to TC-055): one QSL unit compiled by QSL's writer (QSL FR-517) and
//! admitted by Contract IR's reader. The population `workers` the replicated
//! role draws from is added to the recipe model in memory.

// Each test binary uses a different part of these helpers.
#![allow(dead_code)]

use std::path::PathBuf;

use qsl_semantics::check::wire_id::WireGraph;
use quire_contract_model::{CheckedNodeId, CheckedPackageReadLimits, CheckedPackageV2};
use quire_exact::NodeKey;
use quire_protocol::checked_package::{admit, HandoffFiles};
use quire_protocol::disposition::Construct;
use quire_protocol::dynamics::ObservationRef;
use quire_protocol::ids::{MessageId, RoleInstanceId, WorkflowInstanceId};
use quire_protocol::link::{link, LinkedProtocol};
use quire_spec_language::checked_handoff::Corpus;

use super::handoff::{join_budget, request, written_request};

/// The protocol: a static role `S`, one replicated role `R` of lifetime
/// `{lifetime}`, eight channels over distinct routes and payloads, a join of
/// every policy and a loop with and without a maximum.
fn source(lifetime: &str, max: u64) -> String {
    let branches = |prefix: &str| {
        ["a", "b", "c"]
            .iter()
            .map(|label| format!("branch {label} sequence {prefix}{label} {{ }}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let parallel = |name: &str, join: &str| {
        format!(
            "parallel {name} memory sc {{ {} }} join {join};",
            branches(name)
        )
    };
    format!(
        r#"language "ix:native" edition "1-draft";
profile v = "quire.model.complete/v1";
profile p = "quire.protocol.finite-global/v1";
model M = "agent-ix/checked-handoff";
function both using v(a: Boolean, b: Boolean, c: Boolean): Boolean pure {{ a and b and c }}
protocol Flow using p over (input: M::Workflow) on origin {{
  role S on M::Workflow;
  role R each M::Provider from workers max {max} lifetime {lifetime};
  channel Head from S to S carries M::Notice ordering fifo by (m: M::Notice) {{ m.ready }} delivery at-least-once capacity 4 overflow loss head;
  channel Reject from S to R carries M::Notice ordering fifo by (m: M::Notice) {{ m.ready }} delivery exactly-once-premise capacity 2 overflow reject;
  channel Tail from R to S carries M::Notice ordering fifo by (m: M::Notice) {{ m.ready }} delivery at-most-once capacity 2 overflow loss tail;
  channel New from R to R carries M::Notice ordering fifo by (m: M::Notice) {{ m.ready }} delivery unknown capacity 2 overflow loss new;
  channel Block from S to S carries Boolean ordering unordered delivery at-least-once capacity 3 overflow block;
  channel BagNew from S to R carries Boolean ordering unordered delivery unknown capacity 2 overflow loss new;
  channel BagHead from R to S carries Boolean ordering unordered delivery unknown capacity 2 overflow loss head;
  channel Symbolic from R to R carries Boolean ordering unordered delivery unknown capacity symbolic(Bound) overflow reject;
  run sequence Main {{
    {all}
    {any_continue}
    {quorum_cancel}
    {any_unruled}
    {quorum_unruled}
    {all_unruled}
    {predicate}
    repeat Bounded by S visible () max 3 invariant {{ true }} variant {{ 1 }} while {{ false }} sequence BoundedBody {{ event Tick by S as (tick: M::Notice) {{ true }}; }} exhausted sequence BoundedOut {{ }}
    repeat Free by S visible () while {{ false }} sequence FreeBody {{ event Tock by S as (tock: M::Notice) {{ true }}; }}
  }}
  finish End as (outcome: Boolean) {{ true }};
}}
"#,
        all = parallel("JoinAll", "all [a,b,c] outstanding continue"),
        any_continue = parallel("AnyContinue", "any [a,b,c] outstanding continue"),
        quorum_cancel = parallel("QuorumCancel", "quorum(2) [a,b,c] outstanding cancel"),
        any_unruled = parallel("AnyUnruled", "any [a,b,c]"),
        quorum_unruled = parallel("QuorumUnruled", "quorum(2) [a,b,c]"),
        all_unruled = parallel("AllUnruled", "all [a,b,c]"),
        predicate = parallel("Predicate", "predicate(both) [a,b,c]"),
    )
}

/// The recipe model with the population `workers` of `Provider`.
fn model() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/checked-handoff/model.semantic-ir.json"
    );
    let mut model: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).expect("model fixture")).expect("model JSON");
    let package = "ix://agent-ix/checked-handoff";
    model["populations"] = serde_json::json!([{
        "identity": format!("{package}/workers"),
        "displayName": format!("{package}/workers"),
        "kind": {"module": "agent-ix/checked-handoff", "name": "population"},
        "members": [format!("{package}/Provider")],
        "extent": "closed",
        "origin": {"generated": {
            "generatorIdentity": format!("{package}/workers"),
            "generatorVersion": "1.0.0",
            "inputIdentities": [format!("{package}/workers")],
        }},
    }]);
    serde_json::to_vec(&model).expect("model bytes")
}

/// One admitted package and its linked protocol.
pub struct Dynamic {
    package: Box<CheckedPackageV2>,
    clause: CheckedNodeId,
    linked: LinkedProtocol,
}

impl Dynamic {
    /// The package declaring the replicated role `R` with `lifetime` and `max`.
    pub fn with(lifetime: &str, max: u64) -> Self {
        let holder = tempfile::tempdir().expect("temporary directory");
        let model_path: PathBuf = holder.path().join("model.semantic-ir.json");
        std::fs::write(&model_path, model()).expect("model file");
        let source_path = holder.path().join("unit.native");
        std::fs::write(&source_path, source(lifetime, max)).expect("source file");
        let mut asked = request("event_position", Corpus::None);
        asked.emit.source = source_path;
        asked.emit.models = vec![model_path];
        asked.emit.temporal_selections = Vec::new();
        let handoff = written_request(asked);
        let files = HandoffFiles {
            checked_package: &handoff.files["checked-package.json"],
            selection: &handoff.files["selection.json"],
            identities: &handoff.files["identities.json"],
            equality: &handoff.files["equality.json"],
        };
        let package = match admit(
            &files,
            &handoff.evidence,
            CheckedPackageReadLimits::bounded(),
            &mut join_budget(),
        ) {
            Ok(admitted) => Box::new(admitted.package().clone()),
            Err(refusal) => panic!("the dynamic package is refused: {refusal}"),
        };
        let clause = package
            .graph()
            .nodes
            .iter()
            .find(|node| &*node.semantic_form == "protocol_clause")
            .expect("a protocol clause")
            .node_id
            .clone();
        let linked = link(&package, &clause).expect("the dynamic protocol links");
        Self {
            package,
            clause,
            linked,
        }
    }

    /// The admitted package.
    pub fn package(&self) -> &CheckedPackageV2 {
        &self.package
    }

    /// The protocol clause.
    pub fn clause(&self) -> &CheckedNodeId {
        &self.clause
    }

    /// The linked protocol.
    pub fn linked(&self) -> &LinkedProtocol {
        &self.linked
    }

    /// The node of form `form` whose declared name ends in `name`.
    pub fn named(&self, form: &str, name: &str) -> CheckedNodeId {
        self.package
            .graph()
            .nodes
            .iter()
            .find(|node| {
                &*node.semantic_form == form
                    && node
                        .declaration
                        .as_ref()
                        .and_then(|declaration| declaration.qualified_name.last())
                        .is_some_and(|last| &**last == name)
            })
            .unwrap_or_else(|| panic!("no {form} named {name}"))
            .node_id
            .clone()
    }

    /// The construct (node and locus) at `node`.
    pub fn construct(&self, node: &CheckedNodeId) -> Construct {
        Construct::of(&self.package, node)
    }

    /// QSL's key for `node`.
    pub fn key(&self, node: &CheckedNodeId) -> NodeKey {
        WireGraph::new(
            self.package
                .resolved_graph()
                .nodes
                .iter()
                .map(|n| &n.node_id),
        )
        .expect("wire ids")
        .key(node)
        .expect("key")
    }
}

/// The observation at history position `position`.
pub fn at(position: u64) -> ObservationRef {
    ObservationRef {
        position,
        identity: format!("o{position}").into_bytes().into(),
    }
}

/// The role instance the observation names `name`.
pub fn instance(name: &str) -> RoleInstanceId {
    RoleInstanceId::new(name.as_bytes(), name)
}

/// The message named `name`.
pub fn message(name: &str) -> MessageId {
    MessageId::new(name.as_bytes(), name)
}

/// The workflow instance named `name`.
pub fn workflow(name: &str) -> WorkflowInstanceId {
    WorkflowInstanceId::new(name.as_bytes(), name)
}

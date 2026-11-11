// SPDX-License-Identifier: AGPL-3.0-or-later
//! A checked package whose protocol declares an `await` (TC-154): recipe R's
//! unit with an armed event, an `await ... within [0,29]` matching a `Seen`
//! event, and the clock the await selects. Written by QSL's writer and
//! admitted by Contract IR's reader.

use quire_contract_model::{CheckedNodeId, CheckedPackageReadLimits};
use quire_protocol::checked_package::{admit, HandoffFiles};
use quire_protocol::link::{link, LinkedProtocol};
use quire_spec_language::checked_handoff::Corpus;

use super::handoff::{join_budget, request, written_request};

const ANCHOR: &str =
    "        attempt Tried by Service on M::Workflow::apply contracts [BeforeApply,AfterApply]";

/// Where an `await` after the compensation template `Full` goes: past the
/// forward effect it compensates, so the causal relation stays acyclic.
const AFTER_FORWARD: &str =
    "        event Recovered by Service for Full as (recoveryEvent: M::Notice) { recoveryEvent.ready };";

/// The awaiting protocol text with the `await` anchored `after` the named
/// control or compensation template.
fn awaiting_source(after: &str) -> String {
    format!(
        r#"        event Armed by Service as (armed: M::Notice) {{ true }};
        await Wait after {after} using t clock "wait" within [0,29]
            match event Seen by Service as (seen: M::Notice) {{ true }};
            then sequence WaitThen {{ }} timeout sequence WaitTimeout {{ }}
"#
    )
}

/// The admitted package and its linked protocol.
pub struct Awaiting {
    clause: CheckedNodeId,
    linked: LinkedProtocol,
}

impl Awaiting {
    /// The `await` anchored after the event `Main::Armed`.
    pub fn new() -> Self {
        Self::after("Main::Armed")
    }

    /// The `await` anchored after the compensation template `Full`.
    pub fn after_compensation() -> Self {
        Self::after("Full")
    }

    fn after(anchor: &str) -> Self {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/checked-handoff"
        );
        let unit = std::fs::read_to_string(format!("{dir}/unit-event-position.native"))
            .expect("recipe unit");
        let place = if anchor == "Full" {
            AFTER_FORWARD
        } else {
            ANCHOR
        };
        assert!(unit.contains(place), "the recipe's anchor line moved");
        let source = unit.replacen(place, &format!("{}{place}", awaiting_source(anchor)), 1);
        let holder = tempfile::tempdir().expect("temporary directory");
        let path = holder.path().join("unit.native");
        std::fs::write(&path, source).expect("source file");
        let mut asked = request("event_position", Corpus::None);
        asked.emit.source = path;
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
            Ok(admitted) => admitted.package().clone(),
            Err(refusal) => panic!("the awaiting package is refused: {refusal}"),
        };
        let clause = package
            .graph()
            .nodes
            .iter()
            .find(|node| &*node.semantic_form == "protocol_clause")
            .expect("a protocol clause")
            .node_id
            .clone();
        let linked = link(&package, &clause).expect("the protocol links");
        Self { clause, linked }
    }

    pub fn linked(&self) -> &LinkedProtocol {
        &self.linked
    }

    pub fn clause(&self) -> &CheckedNodeId {
        &self.clause
    }
}

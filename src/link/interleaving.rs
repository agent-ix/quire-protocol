// SPDX-License-Identifier: AGPL-3.0-or-later
//! Permitted interleavings of the retained causal graph. Owner chunk: **C-14**.
//!
//! `FR-002-AC-2` requires that "every permitted interleaving of bounded
//! parallel branches preserves authored causal edges and satisfies its declared
//! join". That is a proof obligation over what [`LinkedProtocol`] retained, not
//! a refusal the linker makes: FR-002's Behavior lists what the linker refuses
//! and an interleaving is not on it. So this module ships no public surface and
//! is compiled only under `cfg(test)` — a `pub` enumerator with no caller in
//! the shipped crate would be API debt, and standing one up for the conformance
//! chunks (C-20, C-21) to find later is the placeholder `CLAUDE.md` forbids.
//!
//! # What "permitted" means, and whose rule it is
//!
//! Upstream's, in both halves. `quire-specification` FR-052 states "Parallel
//! branches SHALL admit every interleaving that preserves each branch's causal
//! edges", and `FR-228` names the admitted join rule as the "all-branch join".
//! Nothing here decides which orders are permitted; [`interleavings`] computes
//! the linear extensions of the relation A already derived, validated and
//! published in the artifact's `causal_edges`.
//!
//! Two consequences follow from taking A's edge set *exactly*:
//!
//! - **No edge is added.** FR-002 requires causal edges to be derived "only
//!   from authored control and channel premises", so a control's `enter` is not
//!   silently ordered before its own `exit` here. A emits that edge itself for
//!   every control that has it — `sequence` for a leaf check, commit or event —
//!   and where A emits none, none is assumed.
//! - **Repeat progress is excluded from the order.** A documents
//!   `repeat_progress` as "bounded repeat progress as its explicit cyclic
//!   edge", and FR-052's output is "an acyclic finite control representation".
//!   A relation carrying the back edge has no linear extension at all, so
//!   including it would report zero permitted interleavings for every loop.
//!
//! # Bound
//!
//! There is none, and none is invented. The count of linear extensions is
//! factorial in the width of the graph, so this enumerates the whole set and an
//! artifact-sized input would not finish. That is acceptable *here* and nowhere
//! else: the only inputs are fixtures the tests author. `FR-016` Absence rules
//! out a locally chosen ceiling, and the resource inventory that would grant
//! one is `NFR-002`'s, owner chunk C-30 — the same ruling C-13 recorded for its
//! accumulator.
//!
//! [`LinkedProtocol`]: super::LinkedProtocol

use std::collections::BTreeSet;

use super::EdgeShape;

/// One control endpoint: A's node index and its port.
///
/// The same pair [`EdgeShape`] carries, not a second spelling of it.
pub(super) type Endpoint = (u32, &'static str);

/// A's own spelling of the cyclic back edge.
const REPEAT_PROGRESS: &str = "repeat_progress";

/// Every permitted interleaving of the endpoints these edges relate.
///
/// One linear extension per element, each a total order over every endpoint the
/// edge set mentions. The outer order is deterministic: candidates are tried in
/// endpoint order, so two runs and two hosts produce the same list.
///
/// An empty result means the acyclic relation has a cycle. A refuses a cyclic
/// edge set inside `read`, so an admitted artifact cannot produce one; a
/// hand-built fixture can, and this says so by returning nothing rather than
/// by looping.
pub(super) fn interleavings(edges: &[EdgeShape]) -> Vec<Vec<Endpoint>> {
    let mut endpoints: BTreeSet<Endpoint> = BTreeSet::new();
    for edge in edges {
        endpoints.insert(edge.from);
        endpoints.insert(edge.to);
    }
    let endpoints: Vec<Endpoint> = endpoints.into_iter().collect();
    let relation = precedence(edges);

    let mut found = Vec::new();
    let mut placed: Vec<Endpoint> = Vec::new();
    extend(&endpoints, &relation, &mut placed, &mut found);
    found
}

/// The acyclic half of A's edge set, as ordered pairs.
pub(super) fn precedence(edges: &[EdgeShape]) -> Vec<(Endpoint, Endpoint)> {
    edges
        .iter()
        .filter(|edge| edge.kind != REPEAT_PROGRESS)
        .map(|edge| (edge.from, edge.to))
        .collect()
}

/// Place every endpoint whose predecessors are already placed, and recurse.
fn extend(
    endpoints: &[Endpoint],
    relation: &[(Endpoint, Endpoint)],
    placed: &mut Vec<Endpoint>,
    found: &mut Vec<Vec<Endpoint>>,
) {
    if placed.len() == endpoints.len() {
        found.push(placed.clone());
        return;
    }
    for endpoint in endpoints {
        if placed.contains(endpoint) {
            continue;
        }
        let ready = relation
            .iter()
            .all(|(from, to)| to != endpoint || placed.contains(from));
        if !ready {
            continue;
        }
        placed.push(*endpoint);
        extend(endpoints, relation, placed, found);
        placed.pop();
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    /// One edge, with the fields the order is computed from.
    fn edge(kind: &'static str, from: Endpoint, to: Endpoint) -> EdgeShape {
        EdgeShape {
            kind,
            owner: 0,
            from,
            to,
            maximum: None,
        }
    }

    /// Two independent two-step chains interleave in exactly six orders.
    ///
    /// `C(4, 2) = 6`, which is the count of ways to merge two chains of two
    /// while keeping each chain's own order. The expected list is written out
    /// rather than computed, so an enumerator that dropped an order or invented
    /// one fails against it instead of against its own arithmetic.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_two_chains_merge_in_every_order_that_keeps_both() {
        let left = ((1, "enter"), (1, "exit"));
        let right = ((2, "enter"), (2, "exit"));
        let edges = [
            edge("sequence", left.0, left.1),
            edge("sequence", right.0, right.1),
        ];

        assert_eq!(
            interleavings(&edges),
            [
                vec![left.0, left.1, right.0, right.1],
                vec![left.0, right.0, left.1, right.1],
                vec![left.0, right.0, right.1, left.1],
                vec![right.0, left.0, left.1, right.1],
                vec![right.0, left.0, right.1, left.1],
                vec![right.0, right.1, left.0, left.1],
            ]
        );
    }

    /// A cyclic relation admits nothing, and terminates saying so.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_a_cycle_admits_no_interleaving() {
        let edges = [
            edge("sequence", (1, "enter"), (1, "exit")),
            edge("sequence", (1, "exit"), (1, "enter")),
        ];

        assert!(interleavings(&edges).is_empty());
    }

    /// The repeat back edge is excluded, so a bounded loop still orders.
    ///
    /// With `repeat_progress` in the relation this graph is the cycle above and
    /// reports nothing; A publishes that edge as the explicit cyclic one, so it
    /// is the one edge the acyclic order leaves out.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_the_repeat_back_edge_is_not_an_ordering() {
        let edges = [
            edge("branch", (0, "enter"), (1, "enter")),
            edge("sequence", (1, "enter"), (1, "exit")),
            edge("repeat_progress", (1, "exit"), (0, "enter")),
        ];

        assert_eq!(
            interleavings(&edges),
            [vec![(0, "enter"), (1, "enter"), (1, "exit")]]
        );
        assert_eq!(precedence(&edges).len(), 2);
    }
}

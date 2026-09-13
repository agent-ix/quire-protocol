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
//! # Concurrency, not alternation
//!
//! **A linear extension is an interleaving only when every branch edge means
//! concurrency.** That is the case for a parallel and for nothing else, and the
//! distinction is not visible in the edges: A derives a `branch` edge from a
//! control's `enter` to each sub-body and a `join` edge back, identically for
//! `Parallel` and for `Choice`, and `Repeat` adds `branch` edges to its body
//! and its exhausted arm. Those are **alternatives**. Ordering a choice's two
//! cases against each other would report an order in which the protocol takes
//! `ship` *and* `cancel`, which no run does.
//!
//! An earlier revision filtered `repeat_progress` and treated every other edge
//! as precedence, with the domain restriction stated only in this prose. It
//! computed exactly that false answer for any artifact carrying a choice,
//! repeat or await, and no test called it on one, so the prose was never
//! contradicted by a run. [`interleavings`] now takes the control shapes and
//! refuses such an edge set through [`Exclusive`]; the unrestricted graph
//! walk is [`linear_extensions`], which claims nothing about runs.
//!
//! Enumerating the runs of a choice or a bounded repeat is a different
//! obligation with a different owner — `FR-053` and `FR-052`'s bounded
//! progress — and this module does not quietly stand in for it.
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

use quire_spec_language::protocol_artifact::wire::EdgeKind;

use super::{ControlShape, EdgeShape};

/// One control endpoint: A's node index and its port.
///
/// The same pair [`EdgeShape`] carries, not a second spelling of it.
pub(super) type Endpoint = (u32, &'static str);

/// The edge set spans a control whose branches are alternatives.
///
/// Returned rather than filtered away: dropping the control's edges would
/// answer a question about a *different* graph, and answering it silently is
/// how the earlier revision was wrong.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Exclusive {
    /// The control's index in A's control table.
    pub control: u32,
    /// Its operation kind, as [`ControlShape`] spells it.
    pub kind: &'static str,
}

/// Every permitted interleaving of a purely concurrent edge set.
///
/// One linear extension per element, each a total order over every endpoint the
/// edge set mentions. The outer order is deterministic: candidates are tried in
/// endpoint order, so two runs and two hosts produce the same list.
///
/// Refuses with [`Exclusive`] when any control the edges reach admits
/// alternatives rather than concurrency — see the module documentation.
///
/// An empty `Ok` means the acyclic relation has a cycle. A refuses a cyclic
/// edge set inside `read`, so an admitted artifact cannot produce one; a
/// hand-built fixture can, and this says so by returning nothing rather than
/// by looping.
pub(super) fn interleavings(
    controls: &[ControlShape],
    edges: &[EdgeShape],
) -> Result<Vec<Vec<Endpoint>>, Exclusive> {
    for (index, control) in controls.iter().enumerate() {
        let index = u32::try_from(index).expect("A's control table is indexed by u32");
        if !control.admits_alternatives() {
            continue;
        }
        if edges
            .iter()
            .any(|edge| edge.from.0 == index || edge.to.0 == index || edge.owner == index)
        {
            return Err(Exclusive {
                control: index,
                kind: control.kind(),
            });
        }
    }
    Ok(linear_extensions(edges))
}

/// Every linear extension of the acyclic half of an edge set.
///
/// The graph walk with no claim attached: a linear extension of a relation
/// containing a choice's branch edges is a well-defined order over that
/// relation and is **not** a run of the protocol. [`interleavings`] is the
/// entry point that asserts the difference.
pub(super) fn linear_extensions(edges: &[EdgeShape]) -> Vec<Vec<Endpoint>> {
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
///
/// The excluded kind is A's own [`EdgeKind::RepeatProgress`], asked for by its
/// published spelling rather than by a literal repeated here.
pub(super) fn precedence(edges: &[EdgeShape]) -> Vec<(Endpoint, Endpoint)> {
    edges
        .iter()
        .filter(|edge| edge.kind != EdgeKind::RepeatProgress.as_str())
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

    /// One control shape, with the field the guard reads.
    fn control(kind: &'static str) -> ControlShape {
        ControlShape {
            kind,
            indices: Vec::new(),
        }
    }

    /// One edge, with the fields the order is computed from.
    ///
    /// The kind is A's [`EdgeKind`], asked for its own spelling here rather than
    /// written out: a rename upstream moves these fixtures with it instead of
    /// leaving four tests asserting against a literal no artifact carries.
    fn edge(kind: EdgeKind, from: Endpoint, to: Endpoint) -> EdgeShape {
        EdgeShape {
            kind: kind.as_str(),
            owner: from.0,
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
            edge(EdgeKind::Sequence, left.0, left.1),
            edge(EdgeKind::Sequence, right.0, right.1),
        ];

        assert_eq!(
            linear_extensions(&edges),
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
            edge(EdgeKind::Sequence, (1, "enter"), (1, "exit")),
            edge(EdgeKind::Sequence, (1, "exit"), (1, "enter")),
        ];

        assert!(linear_extensions(&edges).is_empty());
    }

    /// The repeat back edge is excluded, so a bounded loop still orders.
    ///
    /// With `repeat_progress` in the relation this graph is the cycle above and
    /// reports nothing; A publishes that edge as the explicit cyclic one, so it
    /// is the one edge the acyclic order leaves out. The kind is asked for by
    /// `EdgeKind::RepeatProgress.as_str()` on both sides, so a rename upstream
    /// moves the filter and this fixture together instead of silently
    /// un-filtering the edge.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_the_repeat_back_edge_is_not_an_ordering() {
        let edges = [
            edge(EdgeKind::Branch, (0, "enter"), (1, "enter")),
            edge(EdgeKind::Sequence, (1, "enter"), (1, "exit")),
            edge(EdgeKind::RepeatProgress, (1, "exit"), (0, "enter")),
        ];

        assert_eq!(
            linear_extensions(&edges),
            [vec![(0, "enter"), (1, "enter"), (1, "exit")]]
        );
        assert_eq!(precedence(&edges).len(), 2);
    }

    /// A choice's two cases are alternatives, and ordering them is refused.
    ///
    /// This is the smallest graph that shows why the guard exists. The edges
    /// are exactly what A derives for a two-case choice, and they are
    /// indistinguishable from a two-branch parallel's: `branch` out to each
    /// case, `join` back from each. Every linear extension of that relation
    /// places both `(1, "exit")` and `(2, "exit")`, so every one of them
    /// describes a run taking `ship` **and** `cancel`.
    ///
    /// So the enumerator is not asked. [`interleavings`] reads the control kind
    /// and returns [`Exclusive`]. The same six edges under a `parallel` are then
    /// enumerated and written out in full: that list is what the guard prevents
    /// being claimed of a choice, and reading it shows both `exit`s in every
    /// order and both relative orders of them across the six.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_a_choice_is_refused_and_its_orders_would_run_both_cases() {
        let edges = [
            edge(EdgeKind::Branch, (0, "enter"), (1, "enter")),
            edge(EdgeKind::Branch, (0, "enter"), (2, "enter")),
            edge(EdgeKind::Sequence, (1, "enter"), (1, "exit")),
            edge(EdgeKind::Sequence, (2, "enter"), (2, "exit")),
            edge(EdgeKind::Join, (1, "exit"), (0, "exit")),
            edge(EdgeKind::Join, (2, "exit"), (0, "exit")),
        ];
        let choice = [control("choice"), control("check"), control("check")];
        let parallel = [control("parallel"), control("check"), control("check")];

        assert_eq!(
            interleavings(&choice, &edges),
            Err(Exclusive {
                control: 0,
                kind: "choice",
            })
        );

        // The identical edge set under a parallel is enumerated: the edges did
        // not change, the control kind did, and that is the whole distinction.
        let orders = interleavings(&parallel, &edges).expect("a parallel admits interleavings");
        let (enter, exit) = ((0, "enter"), (0, "exit"));
        assert_eq!(
            orders,
            [
                vec![
                    enter,
                    (1, "enter"),
                    (1, "exit"),
                    (2, "enter"),
                    (2, "exit"),
                    exit
                ],
                vec![
                    enter,
                    (1, "enter"),
                    (2, "enter"),
                    (1, "exit"),
                    (2, "exit"),
                    exit
                ],
                vec![
                    enter,
                    (1, "enter"),
                    (2, "enter"),
                    (2, "exit"),
                    (1, "exit"),
                    exit
                ],
                vec![
                    enter,
                    (2, "enter"),
                    (1, "enter"),
                    (1, "exit"),
                    (2, "exit"),
                    exit
                ],
                vec![
                    enter,
                    (2, "enter"),
                    (1, "enter"),
                    (2, "exit"),
                    (1, "exit"),
                    exit
                ],
                vec![
                    enter,
                    (2, "enter"),
                    (2, "exit"),
                    (1, "enter"),
                    (1, "exit"),
                    exit
                ],
            ],
            "C(4, 2) merges of two two-step chains, each bracketed by the control"
        );
    }
}

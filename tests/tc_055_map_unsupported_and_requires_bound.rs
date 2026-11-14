// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-055: QSL's `Unsupported` causes and requires-bound outcomes carried as
//! typed non-success dispositions (FR-057, FR-058).
//!
//! The stand-in answers at the call boundary into QSL (`QslBoundary`) and
//! records every request it receives. A request carries the claim and nothing
//! else, so a test over the recorded requests shows no bound reached QSL.

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use qsl_eval::simulation::protocol::Unsupported;
use qsl_eval::simulation::RequiresBound;
use qsl_foundation::bound::{DomainKey, DomainKind};
use qsl_semantics::family::ClaimExtent;
use quire_protocol::disposition::{
    settle_claims, AssessmentUnsupported, ClaimSettlement, Construct, Disposition, QslBoundary,
    QslReport, UnsupportedCause, UnsupportedDisposition,
};
use support::dynamic::Dynamic;

/// A requested claim: an identity and nothing a bound could ride on.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Claim(&'static str);

/// The premises the request carried.
type Premises = (&'static str, &'static str);

/// What the stand-in answers per claim.
struct Standin {
    answers: BTreeMap<&'static str, QslReport<&'static str>>,
    calls: Vec<Claim>,
}

impl QslBoundary<Claim, &'static str> for Standin {
    fn report(&mut self, claim: &Claim) -> QslReport<&'static str> {
        self.calls.push(claim.clone());
        self.answers.get(claim.0).cloned().expect("an answer")
    }
}

fn premises(claim: &Claim) -> (Claim, Premises) {
    (claim.clone(), (claim.0, "exact-fragment"))
}

fn unsupported(cause: UnsupportedCause, construct: &Construct) -> QslReport<&'static str> {
    QslReport::Unsupported {
        cause,
        construct: Some(construct.clone()),
    }
}

#[test]
#[trace("TC-055", "FR-057-AC-1")]
fn each_adr_listed_cause_is_carried_by_its_qsl_name_with_the_constructs_node() {
    let dynamic = Dynamic::with("workflow", 3);
    let construct = dynamic.construct(&dynamic.named("channel", "Symbolic"));
    let causes = [
        Unsupported::SymbolicCapacity,
        Unsupported::UnenumerableRecord,
    ];
    for cause in causes {
        let claim = Claim("conformance");
        let mut standin = Standin {
            answers: BTreeMap::from([(
                claim.0,
                unsupported(UnsupportedCause::Construct(cause), &construct),
            )]),
            calls: Vec::new(),
        };
        let settled = settle_claims(&mut standin, &[premises(&claim)]);
        let [ClaimSettlement::Disposition(Disposition::Unsupported(UnsupportedDisposition {
            cause: carried,
            construct: carried_construct,
            premises,
            ..
        }))] = settled.as_slice()
        else {
            panic!("{cause:?}: {settled:?}");
        };
        // QSL's own typed value, carried and compared as that value.
        assert_eq!(*carried, UnsupportedCause::Construct(cause));
        assert_eq!(format!("{carried:?}"), format!("Construct({cause:?})"));
        assert_eq!(
            carried_construct.as_ref().expect("a construct").node,
            dynamic.named("channel", "Symbolic")
        );
        assert_eq!(*premises, (claim.0, "exact-fragment"));
    }
}

#[test]
#[trace("TC-055", "FR-057-AC-2")]
fn an_unsupported_claim_leaves_every_other_claim_exactly_as_it_was() {
    let dynamic = Dynamic::with("workflow", 3);
    let construct = dynamic.construct(&dynamic.named("control", "Predicate"));
    let answers = |with_unsupported: bool| {
        let mut answers = BTreeMap::from([
            ("first", QslReport::Result("proved")),
            ("third", QslReport::Result("violated")),
        ]);
        if with_unsupported {
            answers.insert(
                "second",
                unsupported(
                    UnsupportedCause::Assessment(AssessmentUnsupported::JoinPredicate),
                    &construct,
                ),
            );
        }
        answers
    };
    let settle = |claims: &[&'static str], with_unsupported: bool| {
        let mut standin = Standin {
            answers: answers(with_unsupported),
            calls: Vec::new(),
        };
        let requests: Vec<_> = claims.iter().map(|name| premises(&Claim(name))).collect();
        (settle_claims(&mut standin, &requests), standin.calls)
    };
    let (with, calls) = settle(&["first", "second", "third"], true);
    let (without, _) = settle(&["first", "third"], false);
    assert_eq!(with.len(), 3);
    assert_eq!(calls, [Claim("first"), Claim("second"), Claim("third")]);
    assert_eq!(with[0], without[0]);
    assert_eq!(with[2], without[1]);
    assert!(matches!(
        with[1],
        ClaimSettlement::Disposition(Disposition::Unsupported(_))
    ));
}

#[test]
#[trace("TC-055", "FR-057-AC-3")]
fn symbolic_capacity_and_predicate_joins_hold_no_derived_number_or_truth_value() {
    use quire_protocol::dynamics::{
        assess_channels, assess_joins, ChannelEvent, ChannelFact, ChannelOutcome, JoinEvent,
        JoinFact, JoinOutcome,
    };
    use support::dynamic::{at, message, workflow};
    let dynamic = Dynamic::with("workflow", 3);
    let channels = assess_channels(
        dynamic.linked().declarations(),
        &[ChannelFact {
            at: at(1),
            channel: dynamic.named("channel", "Symbolic"),
            event: ChannelEvent::Enqueue {
                message: message("m1"),
                key: b"k".as_slice().into(),
            },
        }],
        |node| dynamic.construct(node),
    )
    .expect("declared");
    let symbolic = dynamic.named("channel", "Symbolic");
    // The unsupported outcome has no occupancy, capacity or violation field:
    // a derived number could not be held in it.
    assert!(channels.iter().any(|outcome| matches!(
        outcome,
        ChannelOutcome::Unsupported { channel, cause: UnsupportedCause::Construct(Unsupported::SymbolicCapacity), .. }
            if *channel == symbolic
    )));
    let joins = assess_joins(
        dynamic.linked().declarations(),
        &[JoinFact {
            at: at(1),
            join: dynamic.named("control", "Predicate"),
            workflow: workflow("w1"),
            repeat_ordinals: Vec::new(),
            event: JoinEvent::Settled,
        }],
        |node| dynamic.construct(node),
    )
    .expect("declared");
    assert!(matches!(
        joins.as_slice(),
        [JoinOutcome::Unsupported {
            cause: UnsupportedCause::Assessment(AssessmentUnsupported::JoinPredicate),
            ..
        }]
    ));
}

#[test]
#[trace("TC-055", "FR-057-AC-4")]
fn an_unrecognised_cause_is_retained_unchanged_and_mapped_to_no_known_cause() {
    let offered: &[u8] = b"QuantumEntangledLoop \xff";
    let cause = UnsupportedCause::unrecognised(offered);
    let UnsupportedCause::Unrecognised(retained) = &cause else {
        panic!("retained as unrecognised");
    };
    assert_eq!(retained.as_bytes(), offered);
    // It equals no cause QSL defines, including the requested-capability one.
    for known in [
        UnsupportedCause::Construct(Unsupported::SymbolicCapacity),
        UnsupportedCause::Assessment(AssessmentUnsupported::JoinPredicate),
        UnsupportedCause::requested_capability(),
    ] {
        assert_ne!(cause, known);
    }
    // Carried through a claim's settlement it stays what it was.
    let dynamic = Dynamic::with("workflow", 3);
    let construct = dynamic.construct(&dynamic.named("channel", "Symbolic"));
    let claim = Claim("conformance");
    let mut standin = Standin {
        answers: BTreeMap::from([(claim.0, unsupported(cause.clone(), &construct))]),
        calls: Vec::new(),
    };
    let settled = settle_claims(&mut standin, &[premises(&claim)]);
    let [ClaimSettlement::Disposition(Disposition::Unsupported(carried))] = settled.as_slice()
    else {
        panic!("{settled:?}");
    };
    assert_eq!(carried.cause, cause);
}

/// A requires-bound outcome of QSL's own type over the three kinds the ticket
/// names, keyed by a real checked node.
fn requires_bound(dynamic: &Dynamic) -> (RequiresBound, [(DomainKey, DomainKind); 3]) {
    let node = dynamic.key(&dynamic.named("control", "Bounded"));
    let domains = [
        (
            DomainKey::Node {
                node: node.clone(),
                path: vec![0],
            },
            DomainKind::Population,
        ),
        (
            DomainKey::Node {
                node: node.clone(),
                path: vec![1],
            },
            DomainKind::Collection,
        ),
        (DomainKey::Node { node, path: vec![] }, DomainKind::Loop),
    ];
    let ClaimExtent::Unbounded(domains_of) =
        ClaimExtent::from_domains(domains.iter().cloned().collect())
    else {
        panic!("three domains are unbounded");
    };
    (
        RequiresBound {
            domains: domains_of,
        },
        domains,
    )
}

#[test]
#[trace("TC-055", "FR-058-AC-1")]
fn a_requires_bound_outcome_is_carried_unchanged_with_each_domain_kind_and_key() {
    let dynamic = Dynamic::with("workflow", 3);
    let (outcome, domains) = requires_bound(&dynamic);
    let construct = dynamic.construct(&dynamic.named("control", "Bounded"));
    let claim = Claim("safety");
    let mut standin = Standin {
        answers: BTreeMap::from([(
            claim.0,
            QslReport::RequiresBound {
                outcome: outcome.clone(),
                construct: Some(construct.clone()),
            },
        )]),
        calls: Vec::new(),
    };
    let settled = settle_claims(&mut standin, &[premises(&claim)]);
    let [ClaimSettlement::Disposition(Disposition::RequiresBound(carried))] = settled.as_slice()
    else {
        panic!("{settled:?}");
    };
    assert_eq!(carried.outcome, outcome);
    assert_eq!(carried.construct, Some(construct));
    let carried_domains: Vec<_> = carried
        .domains()
        .map(|(key, kind)| (key.clone(), kind))
        .collect();
    let mut expected = domains.to_vec();
    expected.sort();
    assert_eq!(carried_domains, expected);
    let kinds: Vec<_> = carried_domains.iter().map(|(_, kind)| *kind).collect();
    assert!(kinds.contains(&DomainKind::Population));
    assert!(kinds.contains(&DomainKind::Collection));
    assert!(kinds.contains(&DomainKind::Loop));
}

#[test]
#[trace("TC-055", "FR-058-AC-2")]
fn no_call_into_qsl_carries_a_bound_and_no_disposition_field_holds_one() {
    let dynamic = Dynamic::with("workflow", 3);
    let (outcome, _) = requires_bound(&dynamic);
    let claim = Claim("safety");
    let mut standin = Standin {
        answers: BTreeMap::from([(
            claim.0,
            QslReport::RequiresBound {
                outcome,
                construct: Some(dynamic.construct(&dynamic.named("control", "Bounded"))),
            },
        )]),
        calls: Vec::new(),
    };
    let settled = settle_claims(&mut standin, &[premises(&claim)]);
    // Exactly one call, and what it received is the claim: `Claim` has no
    // field a bound could ride on, and `QslBoundary::report` has no other
    // parameter.
    assert_eq!(standin.calls, [claim]);
    let [ClaimSettlement::Disposition(Disposition::RequiresBound(carried))] = settled.as_slice()
    else {
        panic!("{settled:?}");
    };
    // The disposition is destructured with no rest pattern: its fields are
    // QSL's outcome, the construct and the premises, and nothing chosen here.
    let quire_protocol::disposition::RequiresBoundDisposition {
        outcome: _,
        construct: _,
        premises,
    } = carried;
    assert_eq!(*premises, (claim_name(), "exact-fragment"));
}

fn claim_name() -> &'static str {
    "safety"
}

#[test]
#[trace("TC-055", "FR-058-AC-3")]
fn a_requires_bound_claim_leaves_other_claims_and_the_closure_axes_unchanged() {
    use quire_protocol::closure::{
        ClosureState, DecisionScopeClosure, DecisionScopeProgress, GlobalConformanceClosure,
        ScopeAxis, ScopeState, SurroundingExecutionClosure, SurroundingExecutionProgress,
    };
    let dynamic = Dynamic::with("workflow", 3);
    let (outcome, _) = requires_bound(&dynamic);
    let construct = dynamic.construct(&dynamic.named("control", "Bounded"));
    let closure = || {
        ClosureState::new(
            Some(DecisionScopeProgress::of(ScopeState::Closed)),
            Some(DecisionScopeClosure::of(ScopeState::Closed)),
            Some(SurroundingExecutionProgress::of(ScopeState::Open)),
            Some(SurroundingExecutionClosure::of(ScopeState::Open)),
            Some(GlobalConformanceClosure::Open),
        )
        .expect("five axes")
    };
    let before = closure();
    let settle = |with_bound: bool| {
        let mut answers = BTreeMap::from([
            ("conformance", QslReport::Result("conformant")),
            ("progress", QslReport::Result("progressing")),
        ]);
        if with_bound {
            answers.insert(
                "safety",
                QslReport::RequiresBound {
                    outcome: outcome.clone(),
                    construct: Some(construct.clone()),
                },
            );
        }
        let claims: Vec<_> = answers.keys().map(|name| premises(&Claim(name))).collect();
        settle_claims(
            &mut Standin {
                answers,
                calls: Vec::new(),
            },
            &claims,
        )
    };
    let with = settle(true);
    let without = settle(false);
    assert_eq!(with.len(), 3);
    // Settlements come back in the order the claims were given (by name).
    let results = |settled: &[ClaimSettlement<&'static str, Premises>]| {
        settled
            .iter()
            .filter(|settlement| matches!(settlement, ClaimSettlement::Result { .. }))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(results(&with), results(&without));
    assert_eq!(closure(), before);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! The stand-in backend of TC-160 to TC-163. It is registered at the provider
//! boundary under a descriptor QSL's registry accepts, records every call it
//! receives, and answers from a function of the call. It decides nothing.

// Each test binary uses a different part of these helpers.
#![allow(dead_code)]

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

use qsl_foundation::source::provenance::OccurrenceKey;
use qsl_foundation::witness::{
    ObservationIdentity, RuntimeValuePath, ValuePathStep, ValuePathSubject,
};
use qsl_foundation::BackendId;
use qsl_replay::spine::ProtocolAnalysisRecord;
use qsl_replay::{
    InconclusiveCause, SeparatingWitnessRecord, SettlementBasis, TracePosition, Truth,
};
use qsl_route::{BackendDescriptor, Candidate, Mode, ProviderOrigin};
use qsl_semantics::check::Capability;
use quire_protocol::dispatch::{
    Answer, BackendResult, Binding, ChannelPremise, Claim, Fragment, Premises, ProtocolClaimKind,
    ProtocolRequest, ProviderBoundary,
};
use quire_protocol::disposition::{QslBoundary, QslReport};

use super::dynamic::Dynamic;

pub type Assumptions = &'static str;
pub type Req = ProtocolRequest<Assumptions, &'static str>;
pub type Bind = Binding<Assumptions, &'static str>;
/// The unified claim a provider is asked about; these tests ask no
/// composition or refinement, whose entries are therefore `()`.
pub type Asked = Claim<Assumptions, &'static str, ()>;
pub type Boundary = ProviderBoundary<'static, Asked, Answer<Sentinel, Infallible>>;

/// The result a stand-in returns, with the parts the dispatcher reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sentinel {
    pub label: &'static str,
    pub truth: Truth,
    pub basis: SettlementBasis,
    pub witness: Option<SeparatingWitnessRecord>,
    pub record: Option<ProtocolAnalysisRecord>,
    pub inconclusive: Option<InconclusiveCause>,
    pub artifact: Option<&'static str>,
}

impl Sentinel {
    pub fn new(label: &'static str, truth: Truth, basis: SettlementBasis) -> Self {
        Self {
            label,
            truth,
            basis,
            witness: None,
            record: None,
            inconclusive: None,
            artifact: None,
        }
    }
}

impl BackendResult for Sentinel {
    fn truth(&self) -> Truth {
        self.truth
    }
    fn basis(&self) -> SettlementBasis {
        self.basis
    }
    fn witness(&self) -> Option<&SeparatingWitnessRecord> {
        self.witness.as_ref()
    }
    fn run_record(&self) -> Option<&ProtocolAnalysisRecord> {
        self.record.as_ref()
    }
    fn inconclusive(&self) -> Option<&InconclusiveCause> {
        self.inconclusive.as_ref()
    }
}

pub type Reply = Box<dyn Fn(&Bind) -> QslReport<Sentinel>>;

/// The calls a stand-in received.
#[derive(Clone, Default)]
pub struct Calls(Rc<RefCell<Vec<Bind>>>);

impl Calls {
    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }
    pub fn all(&self) -> Vec<Bind> {
        self.0.borrow().clone()
    }
}

pub struct Standin {
    answer: Reply,
    calls: Calls,
}

impl Standin {
    pub fn new(answer: Reply, calls: Calls) -> Self {
        Self { answer, calls }
    }
}

impl QslBoundary<Asked, Answer<Sentinel, Infallible>> for Standin {
    fn report(&mut self, claim: &Asked) -> QslReport<Answer<Sentinel, Infallible>> {
        let Claim::Protocol(bound) = claim else {
            return QslReport::Failed(None);
        };
        self.calls.0.borrow_mut().push(bound.clone());
        (self.answer)(bound).map(Answer::Protocol)
    }
}

pub fn id(name: &str) -> BackendId {
    BackendId::new(name)
}

pub fn candidate(name: &str) -> Candidate {
    Candidate::new(id(name))
}

/// Register a stand-in named `name` serving `kinds`; its calls are the
/// returned record.
pub fn serve(boundary: &mut Boundary, name: &str, kinds: &[Capability], answer: Reply) -> Calls {
    let calls = Calls::default();
    let descriptor = BackendDescriptor::new(
        candidate(name),
        ProviderOrigin::Linked,
        kinds.iter().map(|kind| (*kind, Mode::Unbounded)),
        None,
        None,
    )
    .expect("a descriptor QSL's registry accepts");
    boundary
        .register(
            descriptor,
            Standin {
                answer,
                calls: calls.clone(),
            },
        )
        .expect("the registration is accepted");
    calls
}

/// An answer that returns `result` for every call.
pub fn always(result: Sentinel) -> Reply {
    Box::new(move |_| QslReport::Result(result.clone()))
}

/// A request over the fragment of the channels `channels` and roles `S`, `R`,
/// with every premise supplied.
pub fn request(dynamic: &Dynamic, kind: ProtocolClaimKind, channels: &[&str]) -> Req {
    let channel_nodes: Vec<_> = channels
        .iter()
        .map(|name| dynamic.named("channel", name))
        .collect();
    let roles = vec![dynamic.named("role", "S"), dynamic.named("role", "R")];
    Req {
        kind,
        fragment: Fragment {
            channels: channel_nodes.clone(),
            roles: roles.clone(),
        },
        backend: None,
        assumptions: "met",
        premises: Premises {
            channels: channel_nodes
                .into_iter()
                .map(|node| {
                    let declared = dynamic
                        .linked()
                        .declarations()
                        .channel(&node)
                        .expect("a declared channel");
                    let premise = ChannelPremise {
                        ordering: declared.ordering.clone(),
                        delivery: declared.delivery,
                    };
                    (node, premise)
                })
                .collect(),
            roles: roles.into_iter().map(|node| (node, "internal")).collect(),
        },
        sources: vec![dynamic.clause().clone()],
    }
}

/// A separating-witness record over `dynamic`, with `steps` in its value path.
pub fn witness(dynamic: &Dynamic, steps: Vec<ValuePathStep>) -> SeparatingWitnessRecord {
    let occurrence = OccurrenceKey::generated(dynamic.key(dynamic.clause()));
    SeparatingWitnessRecord {
        quantifier: Some(occurrence.clone()),
        deciding_element: qsl_replay::DecidingElement::Value(quire_exact::Value::Integer(
            quire_exact::Integer::from(600_i64),
        )),
        index: Some(2),
        value_path: RuntimeValuePath {
            observation: ObservationIdentity {
                authority: "fixture".to_owned(),
                identity: "o1".to_owned(),
            },
            subject: ValuePathSubject::Built(occurrence),
            steps,
        },
        trace_position: Some(TracePosition::new("frame-7".to_owned())),
    }
}

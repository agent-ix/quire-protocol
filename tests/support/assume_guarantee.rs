// SPDX-License-Identifier: AGPL-3.0-or-later
//! The stand-in backend of TC-200 to TC-204. It is registered at the provider
//! boundary under a descriptor QSL's registry accepts, records every call and
//! answers from a function of the call. It decides nothing.

// Each test binary uses a different part of these helpers.
#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use qsl_foundation::BackendId;
use qsl_replay::{InconclusiveCause, ItemLabel, SeparatingWitnessRecord, SettlementBasis, Truth};
use qsl_route::{BackendDescriptor, Candidate, Mode, ProviderOrigin};
use qsl_semantics::check::Capability;
use quire_contract_model::CheckedNodeId;
use quire_protocol::assume_guarantee::{
    AssumeGuaranteeBinding, AssumeGuaranteeKind, AssumeGuaranteeRequest, AssumeGuaranteeResult,
    PremiseLists,
};
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::dispatch::{
    BackendResult, Claim, ClaimDisposition, ClaimRequest, Fragment, ProviderBoundary,
};
use quire_protocol::disposition::{QslBoundary, QslReport};

use super::dynamic::Dynamic;

pub type Entry = &'static str;
pub type Bind = AssumeGuaranteeBinding<Entry>;
pub type Req = AssumeGuaranteeRequest<Entry>;
/// The unified claim a provider is asked about; these tests ask no
/// projection or realizability, whose components are therefore `()`.
pub type Asked = Claim<(), (), (), Entry>;
pub type Boundary = ProviderBoundary<'static, Asked, Out>;
pub type Claims = ClaimRequest<&'static str, (), (), (), Entry>;
pub type Settled = ClaimDisposition<&'static str, (), (), (), Entry, Out>;

/// The result a stand-in returns: opaque artifacts the dispatcher must carry
/// and the parts it reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Out {
    pub label: ItemLabel,
    pub execution: Option<AssessmentExecution>,
    pub subjects: Vec<CheckedNodeId>,
    pub premises: Option<PremiseLists<Entry>>,
    pub assumptions: Option<Vec<CheckedNodeId>>,
    pub certificate: Option<&'static str>,
    pub implementation_half: Option<&'static str>,
    pub specification_half: Option<&'static str>,
    pub discharge: Vec<(&'static str, &'static str)>,
}

impl Out {
    pub fn new(label: ItemLabel) -> Self {
        Self {
            label,
            execution: None,
            subjects: Vec::new(),
            premises: None,
            assumptions: Some(Vec::new()),
            certificate: None,
            implementation_half: None,
            specification_half: None,
            discharge: Vec::new(),
        }
    }
}

impl AssumeGuaranteeResult<Entry> for Out {
    fn label(&self) -> ItemLabel {
        self.label
    }
    fn stated_execution(&self) -> Option<AssessmentExecution> {
        self.execution
    }
    fn subjects(&self) -> &[CheckedNodeId] {
        &self.subjects
    }
    fn retained_premises(&self) -> Option<&PremiseLists<Entry>> {
        self.premises.as_ref()
    }
    fn retained_assumptions(&self) -> Option<&[CheckedNodeId]> {
        self.assumptions.as_deref()
    }
}

pub type Answer = Box<dyn Fn(&Bind) -> QslReport<Out>>;

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
    answer: Answer,
    calls: Calls,
}

impl QslBoundary<Asked, Out> for Standin {
    fn report(&mut self, claim: &Asked) -> QslReport<Out> {
        let Claim::AssumeGuarantee(bound) = claim else {
            return QslReport::Failed(None);
        };
        self.calls.0.borrow_mut().push(bound.clone());
        (self.answer)(bound)
    }
}

/// The projection and realizability parts of a result are not asked of these
/// stand-ins.
impl BackendResult for Out {
    fn truth(&self) -> Truth {
        Truth::Unavailable
    }
    fn basis(&self) -> SettlementBasis {
        SettlementBasis::Unavailable
    }
    fn witness(&self) -> Option<&SeparatingWitnessRecord> {
        None
    }
    fn inconclusive(&self) -> Option<&InconclusiveCause> {
        None
    }
}

pub fn serve(boundary: &mut Boundary, name: &str, kinds: &[Capability], answer: Answer) -> Calls {
    let calls = Calls::default();
    let descriptor = BackendDescriptor::new(
        Candidate::new(BackendId::new(name)),
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

pub fn always(result: Out) -> Answer {
    Box::new(move |_| QslReport::Result(result.clone()))
}

/// A request over the roles `S` and `R` as component subjects, with the
/// channel `Head` as an assumption identity and a distinct entry in each of
/// the four premise lists.
pub fn request(dynamic: &Dynamic, kind: AssumeGuaranteeKind) -> Req {
    Req {
        kind,
        fragment: Fragment {
            channels: vec![dynamic.named("channel", "Head")],
            roles: vec![dynamic.named("role", "S")],
        },
        subjects: vec![dynamic.named("role", "S"), dynamic.named("role", "R")],
        backend: None,
        assumptions: vec![
            dynamic.named("channel", "Head"),
            dynamic.named("channel", "Tail"),
        ],
        premises: PremiseLists {
            environment: vec!["env-1"],
            relies: vec!["rely-1", "rely-2"],
            liveness_dependencies: vec!["live-1"],
            fairness: vec!["fair-1"],
        },
        cycles: Vec::new(),
        sources: vec![dynamic.clause().clone()],
    }
}

/// `out` naming the request's subjects and retaining its premises and
/// assumptions, as an honest backend does.
pub fn honest(request: &Req, mut out: Out) -> Out {
    out.subjects.clone_from(&request.subjects);
    out.premises = Some(request.premises.clone());
    out.assumptions = Some(request.assumptions.clone());
    out
}

/// A node of the checked package that its protocol does not declare.
pub fn foreign(dynamic: &Dynamic) -> CheckedNodeId {
    let declarations = dynamic.linked().declarations();
    dynamic
        .package()
        .graph()
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .find(|node| !declarations.declares(node))
        .expect("a node the protocol does not declare")
}

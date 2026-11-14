// SPDX-License-Identifier: AGPL-3.0-or-later
//! A realizability fixture built from QSL's public front end: a three-role
//! protocol over a one-object model, both package forms (QSL's in-process
//! `CheckedPackage` the subject needs, and the admitted `CheckedPackageV2` the
//! request names), the model's snapshot, and the request QSL's engine decides.

// Each test binary uses a different part of these helpers.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use qsl_analyze::protocol_analysis::RealizabilityRequest;
use qsl_eval::model_subject::{ModelInstance, ModelSubject, ProtocolSubject, RoleBinding};
use qsl_eval::simulation::model::{ExpansionLimits, ModelSimulationRequest, PopulationUniverse};
use qsl_eval::simulation::protocol::ResolvedMemory;
use qsl_foundation::SourceIdentity;
use qsl_package::{CheckedPackage, EmitLimits};
use qsl_replay::protocol_analysis::{BoundsDraft, RunBudget};
use qsl_semantics::check::PackageDeclarations;
use qsl_semantics::model::accounting::ModelNormalizationLimits;
use qsl_semantics::model::intake::{admit_unit, meaning, package_input, PackageInput};
use qsl_semantics::model::observation::{
    document_reference, DocumentProvision, DocumentRef, ObservationLimits,
};
use quire_checked::key::DeclarationKey;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageReadLimits, CheckedPackageV2,
};
use quire_exact::ScalarLimits;
use quire_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value as Json};

use quire_protocol::link::{link, LinkedProtocol};

use super::projection::{assessed_root, RECORDS};

const PACKAGE: &str = "test/realizability";

fn ident(suffix: &str) -> String {
    format!("ix://{PACKAGE}/{suffix}")
}

fn origin(identity: &str) -> Json {
    json!({"generated": {"generatorIdentity": identity, "generatorVersion": "1.0.0",
        "inputIdentities": [identity]}})
}

/// The domain package: one object type `Cell` of one integer field `v`, and
/// its closed population `cells`.
fn document() -> Vec<u8> {
    let constructs: Vec<Json> = [
        ("object_type", meaning::OBJECT_TYPE),
        ("population", meaning::POPULATION),
        ("value_type", meaning::VALUE_TYPE),
    ]
    .into_iter()
    .map(|(name, meaning)| {
        json!({"kind": {"module": PACKAGE, "name": name}, "moduleVersion": "1.0.0",
            "construct": {"identity": "none", "shape": "record", "members": {},
                "meaning": meaning}})
    })
    .collect();
    let node = |identity: &str, kind: &str, extra: Json| {
        let mut node = json!({
            "identity": identity, "displayName": identity,
            "kind": {"module": PACKAGE, "name": kind}, "roles": [],
            "origin": origin(identity), "constraints": [], "extensions": [],
            "unknownPolicy": "reject",
        });
        for (name, value) in extra.as_object().expect("members") {
            node[name] = value.clone();
        }
        node
    };
    let field = |name: &str, type_ref: &str| {
        let identity = ident(&format!("Cell/{name}"));
        json!({"identity": identity, "name": name, "typeRef": type_ref,
            "presence": "required", "nullable": false, "defaultKind": "none",
            "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
            "origin": origin(&identity)})
    };
    let constraint = |keyword: &str, value: &str| {
        json!({"identity": ident(&format!("Level/constraints/{keyword}")), "keyword": keyword,
            "operands": {"value": value}, "appliesTo": "ix://quire/native/Integer",
            "diagnosticCode": format!("bound.{keyword}"), "origin": origin(&ident("Level"))})
    };
    let population_node = |name: &str, member: &str| {
        let identity = ident(name);
        json!({"identity": identity, "displayName": identity,
            "kind": {"module": PACKAGE, "name": "population"},
            "members": [ident(member)], "extent": "closed", "origin": origin(&identity)})
    };
    json!({
        "contractVersion": "2.0.0",
        "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {"identity": ident("spec"), "version": "1.0.0", "dialect": "spec-bundle"},
        "package": {"identity": PACKAGE, "version": "1.0.0",
            "mappingVersions": [], "profileVersions": []},
        "occurrences": [], "extensions": [],
        "constructs": constructs,
        "types": [
            node(&ident("Level"), "value_type", json!({"scalar": "integer",
                "constraints": [constraint("min", "0"), constraint("max", "3")]})),
            node(&ident("Cell"), "object_type", json!({
                "supertypes": [], "fields": [field("v", &ident("Level"))],
                "operations": [], "relationships": []})),
            // A type for each role: the link refuses two roles on one
            // authority.
            node(&ident("Shop"), "object_type", json!({
                "supertypes": [], "fields": [], "operations": [], "relationships": []})),
            node(&ident("Depot"), "object_type", json!({
                "supertypes": [], "fields": [], "operations": [], "relationships": []})),
        ],
        "populations": [population_node("cells", "Cell"),
            population_node("shops", "Shop"), population_node("depots", "Depot")],
    })
    .to_string()
    .into_bytes()
}

const UNLIMITED: ScalarLimits = ScalarLimits {
    integer_bits: 1 << 20,
    decimal_digits: 1 << 20,
    scale_expansion: 1 << 20,
    text_input_bytes: 1 << 20,
    text_scalars: 1 << 20,
    normalized_scalars: 1 << 20,
    unit_edges: 1 << 20,
    value_occurrences: 1 << 20,
    work_units: 1 << 30,
    result_units: 1 << 30,
};

fn key(suffix: &str) -> DeclarationKey {
    DeclarationKey {
        package: PACKAGE.to_owned(),
        node: ident(suffix),
    }
}

/// The checked package of the QSL unit `text`, by QSL's public front end.
fn checked(text: &str, document: &[u8]) -> CheckedPackage {
    let packages = package_input([document]).expect("the model package reads");
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "realizability"),
        "unit.native",
        text.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("the unit parses");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let built = qsl_forms::build_unit(&parsed).expect("the unit builds");
    let models = admit_unit(
        &built.selections().models,
        &packages,
        ModelNormalizationLimits::default(),
    )
    .expect("the model selection admits")
    .with_parsed_operation_clauses(|text| {
        qsl_forms::parse_clause_text(text, qsl_cst::Limits::default())
    })
    .expect("the operation clauses parse");
    let declarations = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "package".to_owned(),
        built,
        models,
        Vec::new(),
    )
    .expect("the unit assembles");
    let graph = declarations
        .check(CheckingLimits::default())
        .unwrap_or_else(|refusals| panic!("the unit checks: {refusals:?}"));
    CheckedPackage::link(graph)
}

const ROLES: [&str; 3] = ["Buyer", "Seller", "Warehouse"];

fn channel(name: &str, from: &str, to: &str, carries: &str) -> String {
    format!(
        "channel {name} from {from} to {to} carries {carries} ordering unordered delivery \
         exactly-once-premise capacity 4 overflow reject;\n"
    )
}

/// One fixture: both package forms and the model's documents.
pub struct Realizability {
    document: Vec<u8>,
    checked: CheckedPackage,
    admitted: Box<CheckedPackageV2>,
    packages: PackageInput,
    snapshots: DocumentProvision,
    initial: Vec<DocumentRef>,
    clause: CheckedNodeId,
    linked: LinkedProtocol,
}

impl Realizability {
    /// `Buyer` places an order; `Seller` reads it and owns `ship | cancel`.
    pub fn worked(ship: &str, cancel: &str) -> Self {
        let members = [
            "role Buyer on M::Cell;\nrole Seller on M::Shop;\nrole Warehouse on M::Depot;\n"
                .to_owned(),
            channel("Order", "Buyer", "Seller", "Ack"),
            channel("Pick", "Seller", "Warehouse", "PickMsg"),
            channel("Cancelled", "Seller", "Warehouse", "CancelMsg"),
        ]
        .concat();
        let text = format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.value.complete/v1\";\n\
             profile t0 = \"quire.temporal.infinite-trace/v1\";\n\
             profile pp = \"quire.protocol.finite-global/v1\";\n\
             model M = \"{PACKAGE}\";\n\
             {RECORDS}protocol Flow using pp over (k: M::Cell) on origin {{\n{members}\n\
             run sequence Main {{\n\
             send Placed via Order as (placed: Ack) {{ true }};\n\
             receive Got via Order of Placed as (got: Ack) {{ true }};\n\
             choice Decide by Seller visible (got.ok) {{\n\
               case no when {{ not got.ok }} sequence Cancel {{ {cancel} }}\n\
               case yes when {{ got.ok }} sequence Ship {{ {ship} }}\n\
             }} }}\n\
             finish Done as (zz: Boolean) {{ true }};\n}}\n"
        );
        let document = document();
        let checked = checked(&text, &document);
        let emission = qsl_package::emit_checked(&checked, EmitLimits::new(1 << 32))
            .expect("the protocol package emits");
        let mut evidence = emission.evidence().clone();
        evidence
            .insert_domain_package_document(PACKAGE, document.as_slice())
            .expect("fresh domain evidence");
        let admitted = match quire_contract_model::read_checked_package(
            emission.package().bytes(),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        ) {
            CheckedPackageDispatchResult::AdmittedV2(admitted) => admitted,
            other => panic!("the package is admitted: {other:?}"),
        };
        let snapshot = json!({
            "format": "quire.state.snapshot/v1",
            "identity": {"authority": "test", "identity": "s0"},
            "observation": "current", "model": {"identity": PACKAGE},
            "anchor": {"kind": "initialization", "name": "start"},
            "populations": [{"population": ident("cells"), "complete": true,
                "objects": [{"key": "c", "type": ident("Cell"),
                    "fields": {"v": {"integer": "0"}}}]},
                {"population": ident("shops"), "complete": true, "objects": []},
                {"population": ident("depots"), "complete": true, "objects": []}],
        })
        .to_string()
        .into_bytes();
        let mut snapshots = BTreeMap::new();
        snapshots.insert(
            document_reference(&snapshot).expect("a document names itself"),
            snapshot,
        );
        let clause = admitted
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| &*node.semantic_form == "protocol_clause")
            .expect("one protocol clause")
            .node_id
            .clone();
        let linked = link(&admitted, &clause).expect("the protocol links");
        Self {
            document: document.clone(),
            clause,
            linked,
            checked,
            admitted,
            packages: package_input([document.as_slice()]).expect("the model package reads"),
            snapshots,
            initial: vec![DocumentRef {
                authority: "test".to_owned(),
                identity: "s0".to_owned(),
            }],
        }
    }

    /// The package emitted again with `mutate` applied to its wire JSON, read
    /// by Contract IR's reader.
    pub fn read_mutated(&self, mutate: impl FnOnce(&mut Json)) -> CheckedPackageDispatchResult {
        let emission = qsl_package::emit_checked(&self.checked, EmitLimits::new(1 << 32))
            .expect("the protocol package emits");
        let mut evidence = emission.evidence().clone();
        evidence
            .insert_domain_package_document(PACKAGE, self.document.as_slice())
            .expect("fresh domain evidence");
        let mut wire: Json =
            serde_json::from_slice(emission.package().bytes()).expect("the package is JSON");
        mutate(&mut wire);
        quire_contract_model::read_checked_package(
            &serde_json::to_vec(&wire).expect("JSON bytes"),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        )
    }

    /// The admitted package, the request's own form.
    pub fn package(&self) -> &CheckedPackageV2 {
        &self.admitted
    }

    /// The node of form `form` whose declared name ends in `last`.
    pub fn named(&self, form: &str, last: &str) -> CheckedNodeId {
        self.admitted
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| {
                &*node.semantic_form == form
                    && node.declaration.as_ref().is_some_and(|declaration| {
                        declaration.qualified_name.last().map(|segment| &**segment) == Some(last)
                    })
            })
            .unwrap_or_else(|| panic!("no {form} named {last}"))
            .node_id
            .clone()
    }

    /// The protocol clause.
    pub fn clause(&self) -> &CheckedNodeId {
        &self.clause
    }

    /// The linked protocol.
    pub fn linked(&self) -> &LinkedProtocol {
        &self.linked
    }

    fn subject(&self) -> ProtocolSubject<'_> {
        ProtocolSubject {
            model: ModelSubject {
                simulation: ModelSimulationRequest {
                    emit: EmitLimits::new(1 << 32),
                    package: &self.checked,
                    packages: &self.packages,
                    normalization: ModelNormalizationLimits::default(),
                    snapshots: &self.snapshots,
                    initial: self.initial.clone(),
                    universes: [
                        ("cells", vec!["c".to_owned()]),
                        ("shops", Vec::new()),
                        ("depots", Vec::new()),
                    ]
                    .into_iter()
                    .map(|(name, keys)| PopulationUniverse {
                        population: key(name),
                        keys,
                    })
                    .collect(),
                    observation: ObservationLimits::default(),
                    expansion: ExpansionLimits {
                        max_candidates: 1_000_000,
                    },
                    meter: UNLIMITED,
                },
                proof_bounds: Vec::new(),
            },
            protocol: "Flow".to_owned(),
            over: ModelInstance {
                population: key("cells"),
                key: "c".to_owned(),
            },
            roles: ROLES
                .iter()
                .map(|role| RoleBinding {
                    role: (*role).to_owned(),
                    model: PACKAGE.to_owned(),
                })
                .collect(),
            memory: self
                .checked
                .graph()
                .protocol("Flow")
                .map(|found| ResolvedMemory::resolve(found, &[]).expect("no selection resolves"))
                .expect("the protocol Flow"),
        }
    }

    /// The realizability request over the protocol, bounded at depth 64.
    pub fn request(&self) -> RealizabilityRequest<'_> {
        RealizabilityRequest {
            protocol: self.clause.clone(),
            subject: self.subject(),
            definition_key: Some("ix://test/Flow".to_owned()),
            strategy_profile: Some("bounded-static".to_owned()),
            role_knowledge: Some(Some(BTreeSet::new())),
            environment: Some(Some(BTreeSet::new())),
            bounds: Some(BoundsDraft {
                max_depth: Some(64),
                max_history: Some(16),
            }),
            assessed_root: Some(assessed_root()),
            budget: RunBudget::default(),
        }
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-151: the compensation slots of recipe R evaluated through the
//! checked-package evaluation entry (FR-045).

mod support {
    pub mod handoff;
    pub mod limits;
}

use ix_trace_rs::trace;
use std::num::NonZeroU32;
use std::path::Path;

use qsl_eval::value::{ProtocolSlot, ProtocolSuppliedValue, ProtocolSupply, SuppliedValue};
use qsl_package::{BarePackage, BareSupply, ProtocolSlotDescriptor};
use qsl_semantics::check::wire_id::WireGraph;
use quire_contract_model::CheckedPackageReadLimits;
use quire_eval::CallFailure;
use quire_exact::{FieldValue, LimitKind, Meter, NodeKey, Value, ValueType};
use quire_protocol::checked_package::{admit, AdmittedHandoff, HandoffFiles};
use quire_protocol::compensation::algebra::{
    AttemptAdmission, AttemptFact, AttemptLedger, AttemptOutcome, Definition, ForwardFact,
    Limits as AlgebraLimits, Registry,
};
use quire_protocol::compensation::purity::{Capability, CapabilityInventory, PureAssessment};
use quire_protocol::ids::{AttemptId, DefinitionId, EffectId};
use quire_protocol::slot::{attempts_maximum, evaluate, SlotEvaluation, SlotOutcome};
use quire_spec_language::checked_handoff::Corpus;
use support::handoff::written;
use support::limits::UNLIMITED;

fn handoff() -> AdmittedHandoff {
    let handoff = written("event_position", Corpus::None);
    let files = HandoffFiles {
        checked_package: &handoff.files["checked-package.json"],
        selection: &handoff.files["selection.json"],
        identities: &handoff.files["identities.json"],
        equality: &handoff.files["equality.json"],
    };
    admit(
        &files,
        &handoff.evidence,
        CheckedPackageReadLimits::bounded(),
    )
    .expect("R admits")
}

fn bare(admitted: &AdmittedHandoff) -> BarePackage<'_> {
    BarePackage::new(admitted.package()).with_supply(BareSupply {
        identities: admitted.identities(),
        equality: Some(admitted.equality()),
    })
}

fn compensation_key(admitted: &AdmittedHandoff, name: &str) -> NodeKey {
    let package = admitted.package();
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            &*node.semantic_form == "compensation"
                && node.declaration.as_ref().is_some_and(|declaration| {
                    declaration.qualified_name.last().map(|part| &**part) == Some(name)
                })
        })
        .expect("compensation node");
    WireGraph::new(package.graph().nodes.iter().map(|node| &node.node_id))
        .expect("wire ids")
        .key(&node.node_id)
        .expect("key")
}

fn int(value: i64) -> Value {
    Value::Integer(value.into())
}

/// A slot of the compensation `name` and the supply of `values` by binder
/// name; `None` supplies the binder as unavailable.
fn call(
    bare: &BarePackage<'_>,
    node: &NodeKey,
    member: &str,
    values: Vec<(&str, Option<Value>)>,
) -> (ProtocolSlot, ProtocolSupply<'static>) {
    let selected = bare
        .select_protocol_slot(&ProtocolSlotDescriptor::new(node.clone(), member, None))
        .expect("slot selects");
    let binders = values
        .into_iter()
        .map(|(name, value)| {
            let source = selected
                .sources()
                .iter()
                .find(|source| source.name() == name)
                .unwrap_or_else(|| panic!("{member} reads {name}"));
            let supplied = match value {
                Some(value) => SuppliedValue::Value(value),
                None => {
                    SuppliedValue::Unavailable(quire_semantic_value::catalog::CatalogCode::new(
                        "unavailable_observation",
                        "missing-required-artifact",
                    ))
                }
            };
            (
                source.binder().clone(),
                ProtocolSuppliedValue {
                    value: supplied,
                    references: Vec::new(),
                },
            )
        })
        .collect();
    (
        ProtocolSlot {
            node: node.clone(),
            member: member.to_owned(),
            index: None,
        },
        ProtocolSupply {
            binders,
            observation: None,
        },
    )
}

fn record_key(bare: &BarePackage<'_>, node: &NodeKey, member: &str, binder: &str) -> NodeKey {
    let selected = bare
        .select_protocol_slot(&ProtocolSlotDescriptor::new(node.clone(), member, None))
        .expect("slot selects");
    let source = selected
        .sources()
        .iter()
        .find(|source| source.name() == binder)
        .expect("binder");
    let ValueType::Composite(key) = source.value_type() else {
        panic!("{binder} is a record");
    };
    key.clone()
}

fn notice(bare: &BarePackage<'_>, node: &NodeKey, name: &str, ready: bool) -> Value {
    let key = record_key(bare, node, "trigger.guard", &format!("trigger{name}"));
    bare.world()
        .expect("world")
        .types()
        .record(
            key,
            vec![("ready", FieldValue::Present(Value::Boolean(ready)))],
        )
        .expect("Notice")
}

fn attempt(bare: &BarePackage<'_>, node: &NodeKey, name: &str, tally: i64) -> Value {
    let key = record_key(bare, node, "retry.relation", &format!("earlier{name}"));
    bare.world()
        .expect("world")
        .types()
        .record(key, vec![("tally", FieldValue::Present(int(tally)))])
        .expect("Attempt")
}

fn guard(admitted: &AdmittedHandoff, name: &str, ready: bool) -> SlotEvaluation {
    let bare = bare(admitted);
    let node = compensation_key(admitted, name);
    let value = notice(&bare, &node, name, ready);
    let (slot, supply) = call(
        &bare,
        &node,
        "trigger.guard",
        vec![(&format!("trigger{name}"), Some(value))],
    );
    evaluate(&bare, &slot, &supply, &mut Meter::new(UNLIMITED)).expect("guard evaluates")
}

fn retry(
    admitted: &AdmittedHandoff,
    name: &str,
    tallies: (i64, i64),
    target: i64,
) -> SlotEvaluation {
    let bare = bare(admitted);
    let node = compensation_key(admitted, name);
    let values = [
        (format!("target{name}"), Some(int(target))),
        (format!("activated{name}"), Some(Value::Boolean(false))),
        (
            format!("earlier{name}"),
            Some(attempt(&bare, &node, name, tallies.0)),
        ),
        (
            format!("later{name}"),
            Some(attempt(&bare, &node, name, tallies.1)),
        ),
    ];
    let values = values
        .iter()
        .map(|(n, v)| (n.as_str(), v.clone()))
        .collect();
    let (slot, supply) = call(&bare, &node, "retry.relation", values);
    evaluate(&bare, &slot, &supply, &mut Meter::new(UNLIMITED)).expect("retry evaluates")
}

fn holds(evaluation: &SlotEvaluation) -> bool {
    match &evaluation.outcome {
        SlotOutcome::Holds => true,
        SlotOutcome::DoesNotHold => false,
        other => panic!("expected a Boolean outcome, got {other:?}"),
    }
}

/// FR-045-AC-1: the outcome-map rows over R's `Full` and `Partial`
/// compensations that evaluate today: true and false `trigger.guard` and
/// `retry.relation`, an unavailable binder, a wrongly typed value and a meter
/// one below measured usage. R's `recovery.predicate` rows are owed: QSL's bare projection faults on its comprehension binder.
#[test]
#[trace("FR-045-AC-1", "TC-151")]
fn every_outcome_row_has_its_successor_outcome() {
    let admitted = handoff();
    for name in ["Full", "Partial"] {
        assert!(holds(&guard(&admitted, name, false)), "{name} guard true");
        assert!(!holds(&guard(&admitted, name, true)), "{name} guard false");
        assert!(
            holds(&retry(&admitted, name, (0, 1), 5)),
            "{name} retry true"
        );
        assert!(
            !holds(&retry(&admitted, name, (3, 1), 5)),
            "{name} retry false"
        );
        assert!(
            !holds(&retry(&admitted, name, (0, 2), 5)),
            "{name} retry: tally must advance by one"
        );
    }

    let bare = bare(&admitted);
    let node = compensation_key(&admitted, "Full");

    // A binder supplied unavailable is the owner's Unavailable, naming it.
    let unavailable = {
        let notice = notice(&bare, &node, "Full", false);
        let _ = notice;
        let (slot, supply) = call(&bare, &node, "trigger.guard", vec![("triggerFull", None)]);
        evaluate(&bare, &slot, &supply, &mut Meter::new(UNLIMITED)).expect("call")
    };
    let SlotOutcome::Unavailable { binder, cause } = &unavailable.outcome else {
        panic!("expected Unavailable, got {:?}", unavailable.outcome);
    };
    let selected = bare
        .select_protocol_slot(&ProtocolSlotDescriptor::new(
            node.clone(),
            "trigger.guard",
            None,
        ))
        .expect("selects");
    assert_eq!(binder, selected.sources()[0].binder());
    assert_eq!(
        *cause,
        quire_semantic_value::catalog::CatalogCode::new(
            "unavailable_observation",
            "missing-required-artifact"
        )
    );

    // A value of the wrong type is refused at admission with nothing charged.
    let (slot, mut supply) = call(
        &bare,
        &node,
        "trigger.guard",
        vec![("triggerFull", Some(notice(&bare, &node, "Full", false)))],
    );
    for (_, supplied) in &mut supply.binders {
        supplied.value = SuppliedValue::Value(Value::Boolean(true));
    }
    let mut meter = Meter::new(UNLIMITED);
    let failure = evaluate(&bare, &slot, &supply, &mut meter).expect_err("wrong type");
    assert!(matches!(failure, CallFailure::Admission(_)), "{failure:?}");
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 0, "nothing charged");

    // A meter one below measured usage stops the call as Incomplete; the
    // exact usage completes.
    let (slot, supply) = call(
        &bare,
        &node,
        "trigger.guard",
        vec![("triggerFull", Some(notice(&bare, &node, "Full", false)))],
    );
    let mut measured = Meter::new(UNLIMITED);
    evaluate(&bare, &slot, &supply, &mut measured).expect("measured");
    let used = measured.consumed(LimitKind::WorkUnits);
    assert!(used > 0);
    let mut below = UNLIMITED;
    below.work_units = used - 1;
    let stopped = evaluate(&bare, &slot, &supply, &mut Meter::new(below)).expect("call");
    assert!(
        matches!(stopped.outcome, SlotOutcome::Incomplete(_)),
        "{:?}",
        stopped.outcome
    );
    let mut exact = UNLIMITED;
    exact.work_units = used;
    let at_budget = evaluate(&bare, &slot, &supply, &mut Meter::new(exact)).expect("call");
    assert!(holds(&at_budget), "the exact budget completes");
}

/// FR-045-AC-3 (the part that runs today): an unavailable binder of the retry
/// relation is never a true result, and no source converts an evaluation to
/// `bool`. The recovery supplies of the criterion are owed (the same QSL projection fault).
#[test]
#[trace("FR-045-AC-3", "TC-151")]
fn an_unavailable_binder_is_never_a_true_result() {
    let admitted = handoff();
    let bare = bare(&admitted);
    let node = compensation_key(&admitted, "Full");
    for missing in ["targetFull", "activatedFull", "earlierFull", "laterFull"] {
        let values = [
            ("targetFull", int(5)),
            ("activatedFull", Value::Boolean(false)),
            ("earlierFull", attempt(&bare, &node, "Full", 0)),
            ("laterFull", attempt(&bare, &node, "Full", 1)),
        ]
        .into_iter()
        .map(|(name, value)| (name, (name != missing).then_some(value)))
        .collect();
        let (slot, supply) = call(&bare, &node, "retry.relation", values);
        let result = evaluate(&bare, &slot, &supply, &mut Meter::new(UNLIMITED)).expect("call");
        assert!(
            matches!(result.outcome, SlotOutcome::Unavailable { .. }),
            "{missing}: {:?}",
            result.outcome
        );
    }
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    for (path, text) in &files {
        for forbidden in ["unwrap_or(false)", "as_bool", "Into<bool>"] {
            assert!(!text.contains(forbidden), "{path} names {forbidden}");
        }
    }
}

fn sources(dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).expect("src directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push((
                path.display().to_string(),
                std::fs::read_to_string(&path).expect("source"),
            ));
        }
    }
}

#[derive(Default)]
struct NoCapabilities(Vec<Capability>);

impl CapabilityInventory for NoCapabilities {
    fn capabilities(&self) -> &[Capability] {
        &self.0
    }
}

/// FR-045-AC-4: the bound read from `attempts maximum` admits exactly that
/// many attempts, counting the initial one, and one more is the refusal.
#[test]
#[trace("FR-045-AC-4", "TC-151")]
fn the_attempts_maximum_bounds_the_attempt_ledger() {
    let admitted = handoff();
    let package = admitted.package();
    let mut bounds = Vec::new();
    for name in ["Full", "Partial"] {
        let node = package
            .graph()
            .nodes
            .iter()
            .find(|node| {
                &*node.semantic_form == "compensation"
                    && node
                        .declaration
                        .as_ref()
                        .is_some_and(|d| d.qualified_name.last().map(|part| &**part) == Some(name))
            })
            .expect("compensation");
        bounds.push(attempts_maximum(node).expect("authored bound"));
    }
    assert_eq!(bounds, [NonZeroU32::new(3), NonZeroU32::new(3)]);

    let inventory = NoCapabilities::default();
    let pure = || PureAssessment::checked(&inventory).expect("empty surface");
    let forward = EffectId::new(b"effect:forward", "forward");
    let definition = Definition {
        identity: DefinitionId::new(b"definition:full", "Full"),
        paired_forward_effect: forward.clone(),
        retry_bound: bounds[0],
    };
    let registration = Registry::default()
        .observe(
            pure(),
            &[definition],
            &ForwardFact::EstablishedEffect(forward),
        )
        .expect("registers")[0]
        .registration
        .clone();
    let mut ledger =
        AttemptLedger::new(registration.clone(), AlgebraLimits::owner_max()).expect("ledger");
    let fact = |index: u8| AttemptFact {
        attempt: AttemptId::new(&[index], "attempt"),
        registration: registration.identity.clone(),
        established_effect: None,
        outcome: Some(AttemptOutcome::Failed),
    };
    for index in 1..=3u8 {
        assert!(
            matches!(
                ledger.admit(pure(), fact(index)),
                AttemptAdmission::Admitted { position, .. } if position == usize::from(index)
            ),
            "attempt {index} of 3"
        );
    }
    assert!(matches!(
        ledger.admit(pure(), fact(4)),
        AttemptAdmission::RetryBoundExceeded { bound, .. } if bound.get() == 3
    ));
    assert_eq!(ledger.attempts().len(), 3);
}

/// FR-045-AC-6: no source file names the retired state module's items.
#[test]
#[trace("FR-045-AC-6", "TC-151")]
fn no_source_names_the_retired_state_module() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 10);
    for (path, text) in &files {
        for retired in [
            "StateView",
            "EvaluationReport",
            "state::evaluate_v2",
            "ACCOUNTING_VERSION",
        ] {
            let named = text
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .any(|line| line.contains(retired));
            assert!(!named, "{path} names {retired}");
        }
    }
}

/// FR-045-AC-8: the charge log has an entry for each binder read, none for a
/// binder the expression did not read, and the entries with the unattributed
/// one equal the call's total; the log carries its accounting version.
#[test]
#[trace("FR-045-AC-8", "TC-151")]
fn the_charge_log_attributes_each_binder_read() {
    let admitted = handoff();
    let bare = bare(&admitted);
    let node = compensation_key(&admitted, "Full");
    let values = vec![
        ("targetFull", Some(int(5))),
        ("activatedFull", Some(Value::Boolean(false))),
        ("earlierFull", Some(attempt(&bare, &node, "Full", 0))),
        ("laterFull", Some(attempt(&bare, &node, "Full", 1))),
    ];
    let (slot, supply) = call(&bare, &node, "retry.relation", values);
    let mut meter = Meter::new(UNLIMITED);
    let result = evaluate(&bare, &slot, &supply, &mut meter).expect("call");
    let charges = &result.charges;
    assert_eq!(
        charges.accounting_version(),
        qsl_eval::value::BINDER_ACCOUNTING_VERSION
    );
    let selected = bare
        .select_protocol_slot(&ProtocolSlotDescriptor::new(
            node.clone(),
            "retry.relation",
            None,
        ))
        .expect("selects");
    let work = |name: &str| {
        let source = selected
            .sources()
            .iter()
            .find(|source| source.name() == name)
            .expect("binder");
        charges
            .binder(source.binder())
            .map(|amounts| amounts.amount(LimitKind::WorkUnits))
    };
    for read in ["targetFull", "activatedFull", "earlierFull", "laterFull"] {
        assert!(work(read).is_some(), "{read} has an entry");
    }
    assert!(work("earlierFull").is_some_and(|units| units > 0));
    for kind in LimitKind::ALL {
        assert_eq!(
            charges.total().amount(kind),
            meter.consumed(kind),
            "entries and the unattributed entry equal the call's total for {kind:?}"
        );
    }
}

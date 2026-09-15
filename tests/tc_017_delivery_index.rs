// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-017: consumer-owned delivery-index validation.

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::delivery_index::{
    DeliveryIndex, DeliveryIndexContents, DeliveryIndexError, DeliveryIndexKey, DeliveryRoot,
    IndexRead, IndexedInput, IndexedLimit, IndexedMember, IndexedMutation, InputRole,
    LimitDimension, MemberReadError, ProducerRevision, TemporalBindingRecord,
};
use quire_protocol::version_lock::EntryKey;
use quire_spec_language::protocol_artifact::Dimension as CompiledDimension;

fn key(root: &[u8], revision: &[u8]) -> DeliveryIndexKey {
    DeliveryIndexKey::new(DeliveryRoot::new(root), ProducerRevision::new(revision))
}

fn accountant(owner: &str, version: &str) -> EntryKey {
    EntryKey::new(owner.as_bytes(), version.as_bytes()).expect("fixture accountant")
}

fn contents(
    inputs: Vec<IndexedInput>,
    bindings: Vec<TemporalBindingRecord>,
    limits: Vec<IndexedLimit>,
    mutations: Vec<IndexedMutation>,
) -> DeliveryIndexContents {
    DeliveryIndexContents::new(
        IndexedMember::new(b"offer.json", b"{\n  \"exact\": true\n}\n"),
        b"artifact-ref:producer/2",
        inputs,
        b"temporal.json",
        bindings,
        limits,
        mutations,
    )
}

fn valid() -> DeliveryIndex {
    DeliveryIndex::new(
        key(b"delivery-a", b"producer-revision-1"),
        contents(
            vec![IndexedInput::new(
                InputRole::new(b"clock-input"),
                IndexedMember::new(b"clock.json", b"{ \"t\": 1 }\n"),
            )],
            vec![
                TemporalBindingRecord::new(b"producer", 4, b"clock-a.json"),
                TemporalBindingRecord::new(b"producer", 9, b"clock-b.json"),
            ],
            vec![
                IndexedLimit::new(
                    accountant("quire-protocol", "budget/1"),
                    LimitDimension::Protocol(BudgetDimension::InputByte),
                    12,
                ),
                IndexedLimit::new(
                    accountant("quire-spec-language", "compiled/1"),
                    LimitDimension::CompiledProtocol(CompiledDimension::PayloadBytes),
                    12,
                ),
            ],
            vec![IndexedMutation::new(b"wrong-wire", b"seal", b"bad.json")],
        ),
    )
    .expect("valid delivery index")
}

#[test]
#[trace("TC-017", "FR-018-AC-3", "FR-018-AC-6", "FR-018-AC-8", "FR-018-AC-9")]
fn tc_017_retains_exact_state_and_keeps_identity_separate() {
    let index = valid();
    assert_eq!(
        index.offer().bytes().as_bytes(),
        b"{\n  \"exact\": true\n}\n"
    );
    assert_eq!(
        index
            .input(&InputRole::new(b"clock-input"))
            .expect("indexed input")
            .bytes()
            .as_bytes(),
        b"{ \"t\": 1 }\n"
    );
    assert!(matches!(
        index.input(&InputRole::new(b"missing")),
        Err(DeliveryIndexError::UnindexedInputRole(role)) if role.as_bytes() == b"missing"
    ));
    assert_eq!(index.validate_temporal_declarations(&[4, 9]), Ok(()));
    assert_eq!(
        index.validate_temporal_declarations(&[4]),
        Err(DeliveryIndexError::SurplusDeclaration(9))
    );
    assert_eq!(
        index.validate_temporal_declarations(&[4, 9, 10]),
        Err(DeliveryIndexError::MissingDeclaration(10))
    );

    let same_identity_different_state = DeliveryIndex::new(
        key(b"delivery-a", b"producer-revision-1"),
        contents(
            vec![],
            vec![],
            vec![],
            vec![IndexedMutation::new(b"different", b"other", b"other.json")],
        ),
    )
    .expect("state is not identity");
    assert_eq!(index, same_identity_different_state);
    assert_ne!(
        index,
        DeliveryIndex::new(
            key(b"delivery-b", b"producer-revision-1"),
            contents(vec![], vec![], vec![], vec![])
        )
        .expect("other root")
    );
}

#[test]
#[trace("TC-017", "FR-018-AC-3", "FR-018-AC-5", "FR-018-AC-9")]
fn tc_017_refuses_each_local_ambiguity_without_repair() {
    assert_eq!(
        DeliveryIndex::new(
            key(b"", b"revision"),
            contents(vec![], vec![], vec![], vec![]),
        ),
        Err(DeliveryIndexError::EmptyDeliveryRoot)
    );
    assert_eq!(
        DeliveryIndex::new(key(b"root", b""), contents(vec![], vec![], vec![], vec![]),),
        Err(DeliveryIndexError::EmptyProducerRevision)
    );
    let role = InputRole::new(b"clock-input");
    assert!(matches!(
        DeliveryIndex::new(
            key(b"root", b"rev"),
            contents(
                vec![
                    IndexedInput::new(role.clone(), IndexedMember::new(b"one", b"1")),
                    IndexedInput::new(role, IndexedMember::new(b"two", b"2"))
                ],
                vec![],
                vec![],
                vec![]
            )
        ),
        Err(DeliveryIndexError::RepeatedInputRole(_))
    ));
    assert_eq!(
        DeliveryIndex::new(
            key(b"root", b"rev"),
            contents(
                vec![],
                vec![
                    TemporalBindingRecord::new(b"owner", 2, b"two"),
                    TemporalBindingRecord::new(b"owner", 2, b"two-again")
                ],
                vec![],
                vec![]
            )
        ),
        Err(DeliveryIndexError::RepeatedDeclaration(2))
    );
    assert_eq!(
        DeliveryIndex::new(
            key(b"root", b"rev"),
            contents(
                vec![],
                vec![
                    TemporalBindingRecord::new(b"owner", 9, b"nine"),
                    TemporalBindingRecord::new(b"owner", 4, b"four")
                ],
                vec![],
                vec![]
            )
        ),
        Err(DeliveryIndexError::UnsortedBindings {
            previous: 9,
            declaration: 4
        })
    );
    assert!(matches!(
        DeliveryIndex::new(key(b"root", b"rev"), contents(vec![], vec![], vec![], vec![IndexedMutation::new(b"one", b"seal", b"one.json"), IndexedMutation::new(b"two", b"seal", b"two.json")])),
        Err(DeliveryIndexError::AmbiguousMutation(scenario)) if scenario.as_bytes() == b"seal"
    ));
    let duplicate = IndexedLimit::new(
        accountant("owner", "contract/1"),
        LimitDimension::Protocol(BudgetDimension::InputByte),
        1,
    );
    assert!(matches!(
        DeliveryIndex::new(
            key(b"root", b"rev"),
            contents(vec![], vec![], vec![duplicate.clone(), duplicate], vec![])
        ),
        Err(DeliveryIndexError::RepeatedLimit {
            dimension: LimitDimension::Protocol(BudgetDimension::InputByte),
            ..
        })
    ));
}

#[test]
#[trace("TC-017", "FR-018-AC-5", "FR-018-AC-6")]
fn tc_017_preserves_every_adapter_absence_as_a_distinct_outcome() {
    assert_eq!(
        DeliveryIndex::observed(key(b"", b"producer-revision-1"), IndexRead::MissingRoot),
        Err(DeliveryIndexError::EmptyDeliveryRoot)
    );
    let selected = key(b"delivery-a", b"producer-revision-1");
    assert!(matches!(
        DeliveryIndex::observed(selected.clone(), IndexRead::MissingRoot),
        Err(DeliveryIndexError::MissingRoot(root)) if root.as_bytes() == b"delivery-a"
    ));
    assert!(matches!(
        DeliveryIndex::observed(selected.clone(), IndexRead::MissingIndex),
        Err(DeliveryIndexError::MissingIndex(root)) if root.as_bytes() == b"delivery-a"
    ));
    assert!(matches!(
        DeliveryIndex::observed(selected, IndexRead::Unreadable(quire_protocol::version_lock::ExactBytes::new(b"invalid json"))),
        Err(DeliveryIndexError::UnreadableIndex { reason, .. }) if reason.as_bytes() == b"invalid json"
    ));
    assert!(matches!(
        IndexedMember::observed(b"missing.json", Err(MemberReadError::Missing)),
        Err(DeliveryIndexError::MissingMember(path)) if path.as_bytes() == b"missing.json"
    ));
    assert!(matches!(
        IndexedMember::observed(b"unreadable.json", Err(MemberReadError::Unreadable(quire_protocol::version_lock::ExactBytes::new(b"permission")))),
        Err(DeliveryIndexError::UnreadableMember { path, reason })
            if path.as_bytes() == b"unreadable.json" && reason.as_bytes() == b"permission"
    ));
}

#[test]
#[trace("TC-017", "FR-018-AC-9")]
fn tc_017_limit_vocabulary_is_typed_and_ordered_without_substitution() {
    let index = valid();
    let limits = index.limits();
    assert!(matches!(
        limits[0].dimension(),
        LimitDimension::Protocol(BudgetDimension::InputByte)
    ));
    assert!(matches!(
        limits[1].dimension(),
        LimitDimension::CompiledProtocol(CompiledDimension::PayloadBytes)
    ));
    assert_ne!(limits[0].dimension(), limits[1].dimension());
}

#[test]
#[trace("TC-017", "NFR-002-AC-2")]
fn tc_017_bounded_assembly_charges_exact_retained_bytes_before_the_index_exists() {
    let inventory = production_inventory().expect("production sites reconcile");
    let exact_retained_bytes = b"delivery-a".len()
        + b"producer-revision-1".len()
        + b"offer.json".len()
        + b"{\n  \"exact\": true\n}\n".len()
        + b"artifact-ref:producer/2".len()
        + b"temporal.json".len();
    let budget = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, exact_retained_bytes - 1)
        .expect("positive byte budget")
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("positive allocation budget");
    let mut resources = ResourceAccountant::new(budget, &inventory);
    let outcome = DeliveryIndex::new_bounded(
        key(b"delivery-a", b"producer-revision-1"),
        contents(vec![], vec![], vec![], vec![]),
        &mut resources,
    );
    let OperationOutcome::ResourceIncomplete(incomplete) = outcome else {
        panic!("the one-byte budget must stop retention before an index exists");
    };
    assert_eq!(incomplete.dimension(), BudgetDimension::InputByte);
    assert_eq!(resources.usage().consumed(BudgetDimension::InputByte), 0);
    assert_eq!(
        resources
            .usage()
            .consumed(BudgetDimension::AccountedAllocation),
        0
    );

    let sufficient = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, exact_retained_bytes)
        .expect("positive byte budget")
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("positive allocation budget");
    let mut fresh = ResourceAccountant::new(sufficient, &inventory);
    assert!(matches!(
        DeliveryIndex::new_bounded(
            key(b"delivery-a", b"producer-revision-1"),
            contents(vec![], vec![], vec![], vec![]),
            &mut fresh,
        ),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        fresh.usage().consumed(BudgetDimension::InputByte),
        exact_retained_bytes
    );
    assert_eq!(
        fresh.usage().consumed(BudgetDimension::AccountedAllocation),
        1
    );
}

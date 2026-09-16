// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-009: production caller-budget boundaries, monotonic deadlines, and the
//! bidirectionally reconciled source-site inventory.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::{
    production_entries, production_inventory, production_sources, AccountingFault, BoundaryCase,
    Budget, BudgetDimension, BudgetError, ChargeError, ChargePoint, IncompleteCode, Inventory,
    InventoryEntry, InventoryError, LimitAuthority, OperationOutcome, ResourceAccountant,
    SourceSite, Termination,
};
use quire_protocol::bounds::{Clock, Phase, SiteId, SiteKind};
use quire_protocol::repro::ToolchainIdentity;

// ---------------------------------------------------------------------------
// NFR-002-AC-3 / FR-016 — QProtocol caller-budget vocabulary and inventory
// ---------------------------------------------------------------------------

const UNLISTED: SiteId = SiteId::new("c30.fixture.unlisted");
const CALLER_SITE: SiteId = SiteId::new("c30.fixture.caller-controlled-retention");
const CALLER_BOUNDARY: BoundaryCase = BoundaryCase::new("tc_009_caller_budget_boundaries");

fn caller_source() -> SourceSite {
    SourceSite::new(
        CALLER_SITE,
        "c30::fixture::retain",
        BudgetDimension::InputByte,
        SiteKind::Retention,
    )
}

fn caller_entry() -> InventoryEntry {
    InventoryEntry::new(
        caller_source(),
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
        IncompleteCode::BudgetExceeded,
        CALLER_BOUNDARY,
    )
}

fn caller_inventory() -> Inventory {
    Inventory::reconcile(&[caller_source()], &[caller_entry()])
        .expect("one exact source and inventory entry reconcile")
}

struct ScriptedClock {
    monotonic: Vec<Instant>,
    index: Cell<usize>,
}

impl ScriptedClock {
    fn new(origin: Instant, monotonic_offsets: &[Duration]) -> Self {
        Self {
            monotonic: monotonic_offsets
                .iter()
                .map(|offset| origin + *offset)
                .collect(),
            index: Cell::new(0),
        }
    }
}

impl Clock for ScriptedClock {
    fn monotonic(&self) -> Instant {
        let index = self.index.get();
        self.index.set(index + 1);
        self.monotonic[index.min(self.monotonic.len() - 1)]
    }
}

#[trace("TC-009", "NFR-002-AC-3", "FR-016-AC-1")]
#[test]
fn tc_009_caller_budget_vocabulary_is_closed_and_exact() {
    let labels = BudgetDimension::ALL.map(BudgetDimension::as_str);
    assert_eq!(
        labels,
        [
            "input-byte",
            "definition",
            "node",
            "edge",
            "role",
            "message",
            "branch",
            "iteration",
            "active-instance",
            "observation",
            "lineage",
            "result-item",
            "derived-artifact",
            "output-byte",
            "operation",
            "accounted-allocation",
            "elapsed-time",
            "resolution-entry-count",
            "resolution-bytes-digested",
            "resolution-deadline",
        ]
    );
    let unique: BTreeSet<_> = labels.into_iter().collect();
    assert_eq!(unique.len(), BudgetDimension::ALL.len());
}

#[trace("TC-009", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-3")]
#[test]
fn tc_009_caller_budget_boundaries_are_inclusive_and_dimension_local() {
    let inventory = Inventory::reconcile(&[caller_source()], &[caller_entry()])
        .expect("one exact source and inventory entry reconcile");
    let budget = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, 2)
        .expect("positive grant");
    let mut accountant = ResourceAccountant::new(budget, &inventory);

    accountant
        .charge(CALLER_SITE, 1)
        .expect("below the inclusive boundary");
    accountant
        .charge(CALLER_SITE, 1)
        .expect("at the inclusive boundary");
    let before = accountant.usage();
    let ChargeError::ResourceIncomplete(incomplete) = accountant
        .charge(CALLER_SITE, 1)
        .expect_err("the smallest excess unit is resource-incomplete")
    else {
        panic!("budget exhaustion must not collapse into an accounting fault");
    };

    assert_eq!(incomplete.code(), IncompleteCode::BudgetExceeded);
    assert_eq!(incomplete.identity().code(), IncompleteCode::BudgetExceeded);
    assert_eq!(
        incomplete.identity().dimension(),
        BudgetDimension::InputByte
    );
    assert_eq!(incomplete.identity().site(), CALLER_SITE);
    assert!(
        !incomplete.code().as_str().starts_with("refusal."),
        "resource-incomplete does not borrow an input-refusal identity"
    );
    assert_eq!(incomplete.dimension(), BudgetDimension::InputByte);
    assert_eq!(incomplete.site(), CALLER_SITE);
    assert_eq!(incomplete.granted(), 2);
    assert_eq!(incomplete.consumed(), 2);
    assert_eq!(incomplete.requested(), 1);
    assert_eq!(accountant.usage(), before, "refused work charges nothing");
    for dimension in BudgetDimension::ALL {
        assert_eq!(
            accountant.usage().consumed(dimension),
            usize::from(dimension == BudgetDimension::InputByte) * 2,
            "exhausting one dimension changes no other dimension"
        );
    }
}

#[trace("TC-009", "NFR-002-AC-3", "FR-016-AC-5")]
#[test]
fn tc_009_inventory_and_counter_faults_are_not_resource_incomplete() {
    let inventory = Inventory::reconcile(&[caller_source()], &[caller_entry()])
        .expect("fixture inventory reconciles");
    let mut unbounded = ResourceAccountant::new(Budget::nothing_granted(), &inventory);

    assert_eq!(
        unbounded.charge(UNLISTED, 1),
        Err(ChargeError::UnregisteredSite { site: UNLISTED })
    );
    unbounded
        .charge(CALLER_SITE, 1)
        .expect("an absent caller grant stays absent; the authored maximum is separate");
    assert_eq!(
        unbounded.charge(CALLER_SITE, usize::MAX),
        Err(ChargeError::CounterOverflow {
            site: CALLER_SITE,
            consumed: 1,
            requested: usize::MAX,
        })
    );
}

#[trace("TC-009", "NFR-002-AC-3", "FR-016-AC-4")]
#[test]
fn tc_009_fresh_budget_carries_no_exhausted_state() {
    let inventory = Inventory::reconcile(&[caller_source()], &[caller_entry()])
        .expect("fixture inventory reconciles");
    let make_budget = || {
        Budget::nothing_granted()
            .grant(BudgetDimension::InputByte, 2)
            .expect("positive grant")
    };
    let mut exhausted = ResourceAccountant::new(make_budget(), &inventory);
    exhausted
        .charge(CALLER_SITE, 2)
        .expect("exactly sufficient");
    exhausted
        .charge(CALLER_SITE, 1)
        .expect_err("one unit beyond the grant");

    let mut fresh = ResourceAccountant::new(make_budget(), &inventory);
    fresh
        .charge(CALLER_SITE, 2)
        .expect("fresh exact grant admits independently");
    assert_eq!(
        fresh.usage().consumed(BudgetDimension::InputByte),
        2,
        "no partial state crossed accountant instances"
    );
}

#[trace("TC-009", "NFR-002-AC-3", "FR-016-AC-7")]
#[test]
fn tc_009_absent_zero_and_duplicate_grants_never_default_or_overwrite() {
    let absent = Budget::nothing_granted();
    assert_eq!(absent.grant_for(BudgetDimension::InputByte), None);
    assert_eq!(
        absent.clone().grant(BudgetDimension::InputByte, 0),
        Err(BudgetError::NonPositiveGrant {
            dimension: BudgetDimension::InputByte,
        })
    );
    let once = absent
        .grant(BudgetDimension::InputByte, 2)
        .expect("first positive grant");
    assert_eq!(
        once.grant(BudgetDimension::InputByte, 3),
        Err(BudgetError::DuplicateGrant {
            dimension: BudgetDimension::InputByte,
        }),
        "a second grant cannot silently replace the selected value"
    );
}

#[trace("TC-009", "NFR-002-AC-3")]
#[test]
fn tc_009_inventory_reconciliation_kills_delete_duplicate_orphan_and_unbound_mutants() {
    let source = caller_source();
    let entry = caller_entry();
    assert!(Inventory::reconcile(&[source], &[entry]).is_ok());

    assert_eq!(
        Inventory::reconcile(&[source], &[]),
        Err(InventoryError::MissingEntry { site: CALLER_SITE })
    );
    assert_eq!(
        Inventory::reconcile(&[source], &[entry, entry]),
        Err(InventoryError::DuplicateEntry { site: CALLER_SITE })
    );
    assert_eq!(
        Inventory::reconcile(&[], &[entry]),
        Err(InventoryError::OrphanEntry { site: CALLER_SITE })
    );
    assert_eq!(
        Inventory::reconcile(&[source, source], &[entry]),
        Err(InventoryError::DuplicateSource { site: CALLER_SITE })
    );

    let unbound = InventoryEntry::new(
        source,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
        IncompleteCode::BudgetExceeded,
        BoundaryCase::UNASSIGNED,
    );
    assert_eq!(
        Inventory::reconcile(&[source], &[unbound]),
        Err(InventoryError::BoundaryUnassigned { site: CALLER_SITE })
    );

    let mismatched = InventoryEntry::new(
        SourceSite::new(
            CALLER_SITE,
            "c30::fixture::retain",
            BudgetDimension::OutputByte,
            SiteKind::Retention,
        ),
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
        IncompleteCode::BudgetExceeded,
        CALLER_BOUNDARY,
    );
    assert_eq!(
        Inventory::reconcile(&[source], &[mismatched]),
        Err(InventoryError::BindingMismatch { site: CALLER_SITE })
    );
}

#[trace("TC-009", "NFR-002-AC-1", "NFR-002-AC-3", "FR-016-AC-2")]
#[test]
fn tc_009_every_production_site_has_below_at_above_boundary() {
    let inventory = production_inventory().expect("the production inventory reconciles");
    assert_eq!(inventory.entries(), production_entries());

    let covered: BTreeSet<_> = production_sources()
        .iter()
        .map(|site| site.dimension())
        .collect();
    let required: BTreeSet<_> = BudgetDimension::ALL.into_iter().collect();
    assert_eq!(
        covered, required,
        "every NFR-002 caller dimension has a real production charge point"
    );

    for source in production_sources() {
        let budget = Budget::nothing_granted()
            .grant(source.dimension(), 2)
            .expect("positive grant");
        let mut accountant = ResourceAccountant::new(budget, &inventory);
        if matches!(
            source.dimension(),
            BudgetDimension::ElapsedTime | BudgetDimension::ResolutionDeadline
        ) {
            assert_eq!(
                accountant.charge(source.id(), 1),
                Err(ChargeError::TemporalDimensionChargedStatically {
                    site: source.id(),
                    dimension: source.dimension(),
                }),
                "temporal work must use the monotonic before/after mechanism"
            );
            continue;
        }
        accountant
            .charge(source.id(), 1)
            .expect("below the site's inclusive boundary");
        accountant
            .charge(source.id(), 1)
            .expect("at the site's inclusive boundary");
        let before = accountant.usage();
        let ChargeError::ResourceIncomplete(incomplete) = accountant
            .charge(source.id(), 1)
            .expect_err("one unit over the site's boundary")
        else {
            panic!("{} returned an accounting fault", source.id());
        };
        assert_eq!(incomplete.site(), source.id());
        assert_eq!(incomplete.dimension(), source.dimension());
        assert_eq!(incomplete.granted(), 2);
        assert_eq!(incomplete.consumed(), 2);
        assert_eq!(incomplete.requested(), 1);
        assert_eq!(
            accountant.usage(),
            before,
            "{} charged excess work",
            source.id()
        );
    }
}

#[trace("TC-009", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-7")]
#[test]
fn tc_009_authored_and_caller_ceilings_are_enforced_together() {
    let inventory = caller_inventory();

    let mut authored_only = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    authored_only
        .charge_with_limit(CALLER_SITE, 2, 2)
        .expect("exact authored ceiling");
    let ChargeError::ResourceIncomplete(incomplete) = authored_only
        .charge_with_limit(CALLER_SITE, 1, 2)
        .expect_err("authored ceiling refuses its excess unit")
    else {
        panic!("authored excess returned an accounting fault");
    };
    assert_eq!(incomplete.granted(), 2);
    assert_eq!(incomplete.consumed(), 2);
    assert_eq!(
        authored_only.usage().consumed(BudgetDimension::InputByte),
        2
    );

    let caller_lower = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, 1)
        .expect("positive caller grant");
    let mut caller_lower = ResourceAccountant::new(caller_lower, &inventory);
    let ChargeError::ResourceIncomplete(incomplete) = caller_lower
        .charge_with_limit(CALLER_SITE, 2, 3)
        .expect_err("caller ceiling lowers the authored ceiling")
    else {
        panic!("caller excess returned an accounting fault");
    };
    assert_eq!(incomplete.granted(), 1);
    assert_eq!(incomplete.consumed(), 0);

    let caller_higher = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, 3)
        .expect("positive caller grant");
    let mut caller_higher = ResourceAccountant::new(caller_higher, &inventory);
    caller_higher
        .charge_with_limit(CALLER_SITE, 2, 2)
        .expect("caller cannot lower exact authored use when it grants more");
    let ChargeError::ResourceIncomplete(incomplete) = caller_higher
        .charge_with_limit(CALLER_SITE, 1, 2)
        .expect_err("caller cannot raise the authored ceiling")
    else {
        panic!("authored excess returned an accounting fault");
    };
    assert_eq!(incomplete.granted(), 2);

    const SECOND_SITE: SiteId = SiteId::new("c30.fixture.second-retention");
    let second_source = SourceSite::new(
        SECOND_SITE,
        "c30::fixture::retain_second",
        BudgetDimension::InputByte,
        SiteKind::Retention,
    );
    let second_entry = InventoryEntry::new(
        second_source,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
        IncompleteCode::BudgetExceeded,
        CALLER_BOUNDARY,
    );
    let independent_inventory = Inventory::reconcile(
        &[caller_source(), second_source],
        &[caller_entry(), second_entry],
    )
    .expect("two independently bounded sites reconcile");
    let caller_four = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, 4)
        .expect("positive caller grant");
    let mut independent = ResourceAccountant::new(caller_four, &independent_inventory);
    independent
        .charge_with_limit(CALLER_SITE, 2, 2)
        .expect("first site reaches its own authored ceiling");
    independent
        .charge_with_limit(SECOND_SITE, 2, 2)
        .expect("second site has an independent authored ceiling");
    assert_eq!(independent.usage().consumed(BudgetDimension::InputByte), 4);

    let mut transactional = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    let charges = [(CALLER_SITE, 1, 2), (CALLER_SITE, 2, 2)];
    assert!(matches!(
        transactional.charge_all_with_limits(&charges),
        Err(ChargeError::ResourceIncomplete(_))
    ));
    assert_eq!(
        transactional.usage().consumed(BudgetDimension::InputByte),
        0
    );
}

#[trace("TC-009", "NFR-002-AC-3")]
#[test]
fn tc_009_production_inventory_mutants_fail_bidirectional_reconciliation() {
    let sources = production_sources();
    let entries = production_entries();
    assert!(sources.len() > BudgetDimension::NFR_002.len());
    assert_eq!(sources.len(), entries.len());

    for index in 0..sources.len() {
        let mut missing_entry = entries.to_vec();
        let removed = missing_entry.remove(index);
        assert_eq!(
            Inventory::reconcile(sources, &missing_entry),
            Err(InventoryError::MissingEntry {
                site: removed.source().id(),
            })
        );

        let mut missing_source = sources.to_vec();
        let removed = missing_source.remove(index);
        assert_eq!(
            Inventory::reconcile(&missing_source, entries),
            Err(InventoryError::OrphanEntry { site: removed.id() })
        );
    }

    let mut duplicate_entries = entries.to_vec();
    duplicate_entries.push(entries[0]);
    assert_eq!(
        Inventory::reconcile(sources, &duplicate_entries),
        Err(InventoryError::DuplicateEntry {
            site: entries[0].source().id(),
        })
    );

    let mut duplicate_sources = sources.to_vec();
    duplicate_sources.push(sources[0]);
    assert_eq!(
        Inventory::reconcile(&duplicate_sources, entries),
        Err(InventoryError::DuplicateSource {
            site: sources[0].id(),
        })
    );
}

/// A clock that runs backwards *within* a unit is a fault, not an expiry.
///
/// The intra-unit regression branch was dead to this suite: every regression
/// test drove the clock back past `start`, so only the `checked_duration_since`
/// path ran, and deleting the branch left all tests green. A reading that moves
/// backwards but stays after `start` reaches it — and with the branch gone it
/// returned `Ok`, silently walking the guard's own monotonicity floor backwards
/// and masking a later expiry.
#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_a_reading_that_regresses_within_the_budget_is_a_fault() {
    let origin = Instant::now();
    let clock = ScriptedClock::new(
        origin,
        &[
            Duration::ZERO,
            Duration::from_millis(90),
            Duration::from_millis(50),
        ],
    );
    let inventory = production_inventory().expect("production inventory");
    let budget = Budget::nothing_granted()
        .grant(BudgetDimension::ElapsedTime, 100_000_000)
        .expect("100ms deadline");
    let mut resources = ResourceAccountant::with_clock(budget, &inventory, clock);

    assert!(matches!(
        ToolchainIdentity::new_bounded(b"toolchain", "toolchain", &mut resources),
        OperationOutcome::AccountingFault(AccountingFault::ClockRegressed {
            phase: Phase::AfterUnit,
            previous,
            ..
        }) if previous == Duration::from_millis(90)
    ));
}

#[trace("TC-009", "NFR-002-AC-2", "FR-016-AC-6")]
#[test]
fn tc_009_production_bounded_unit_reports_measured_deadline_overshoot() {
    let origin = Instant::now();
    let clock = ScriptedClock::new(
        origin,
        &[
            Duration::ZERO,
            Duration::from_millis(90),
            Duration::from_millis(110),
        ],
    );
    let inventory = production_inventory().expect("production inventory");
    let budget = Budget::nothing_granted()
        .grant(BudgetDimension::ElapsedTime, 100_000_000)
        .expect("100ms deadline");
    let mut resources = ResourceAccountant::with_clock(budget, &inventory, clock);

    let OperationOutcome::ResourceIncomplete(incomplete) =
        ToolchainIdentity::new_bounded(b"toolchain", "toolchain", &mut resources)
    else {
        panic!("the real bounded identity unit must surface deadline expiry");
    };
    let observation = incomplete
        .deadline()
        .expect("elapsed-time exhaustion carries its observation");

    assert_eq!(incomplete.dimension(), BudgetDimension::ElapsedTime);
    assert_eq!(observation.phase(), Phase::AfterUnit);
    assert_eq!(observation.observed(), Duration::from_millis(110));
    assert_eq!(observation.overshoot(), Duration::from_millis(10));
    assert_eq!(
        resources.usage().consumed(BudgetDimension::ElapsedTime),
        110_000_000
    );
}

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_production_deadline_stops_before_work_and_clock_regression_is_a_fault() {
    let origin = Instant::now();
    let inventory = production_inventory().expect("production inventory");
    let make_budget = || {
        Budget::nothing_granted()
            .grant(BudgetDimension::ElapsedTime, 100_000_000)
            .expect("100ms deadline")
    };

    let expired = ScriptedClock::new(origin, &[Duration::ZERO, Duration::from_millis(101)]);
    let mut expired_resources = ResourceAccountant::with_clock(make_budget(), &inventory, expired);
    let OperationOutcome::ResourceIncomplete(incomplete) =
        ToolchainIdentity::new_bounded(b"not-retained", "not retained", &mut expired_resources)
    else {
        panic!("an expired before-unit observation must stop the operation");
    };
    assert_eq!(
        incomplete.deadline().expect("deadline detail").phase(),
        Phase::BeforeUnit
    );
    assert_eq!(
        expired_resources
            .usage()
            .consumed(BudgetDimension::InputByte),
        0,
        "no identity bytes were retained after before-unit expiry"
    );
    assert_eq!(
        expired_resources
            .usage()
            .consumed(BudgetDimension::AccountedAllocation),
        0,
        "no identity allocation was charged after before-unit expiry"
    );

    let regressed = ScriptedClock::new(
        origin,
        &[
            Duration::ZERO,
            Duration::from_millis(90),
            Duration::from_millis(50),
        ],
    );
    let mut regressed_resources =
        ResourceAccountant::with_clock(make_budget(), &inventory, regressed);
    assert!(matches!(
        ToolchainIdentity::new_bounded(b"toolchain", "toolchain", &mut regressed_resources),
        OperationOutcome::AccountingFault(AccountingFault::ClockRegressed {
            phase: Phase::AfterUnit,
            previous,
            ..
        }) if previous == Duration::from_millis(90)
    ));
}

#[trace("TC-009", "NFR-002-AC-2", "NFR-002-AC-3")]
#[test]
fn tc_009_production_deadline_requires_its_exact_reconciled_site() {
    let origin = Instant::now();
    let make_clock =
        || ScriptedClock::new(origin, &[Duration::ZERO, Duration::ZERO, Duration::ZERO]);
    let make_budget = || {
        Budget::nothing_granted()
            .grant(BudgetDimension::ElapsedTime, 1)
            .expect("one nanosecond deadline")
    };

    let caller_only = caller_inventory();
    let mut missing = ResourceAccountant::with_clock(make_budget(), &caller_only, make_clock());
    assert!(matches!(
        ToolchainIdentity::new_bounded(b"toolchain", "toolchain", &mut missing),
        OperationOutcome::AccountingFault(AccountingFault::UnregisteredSite { site })
            if site == SiteId::new("request/unit/deadline")
    ));

    let wrong_source = SourceSite::new(
        SiteId::new("request/unit/deadline"),
        "fixture::wrong_deadline",
        BudgetDimension::Operation,
        SiteKind::Iteration,
    );
    let wrong_entry = InventoryEntry::new(
        wrong_source,
        LimitAuthority::ConsumerOwner,
        ChargePoint::AroundUnit,
        Termination::SingleUnit,
        IncompleteCode::BudgetExceeded,
        CALLER_BOUNDARY,
    );
    let wrong_inventory =
        Inventory::reconcile(&[wrong_source], &[wrong_entry]).expect("fixture reconciles itself");
    let mut wrong = ResourceAccountant::with_clock(make_budget(), &wrong_inventory, make_clock());
    assert!(matches!(
        ToolchainIdentity::new_bounded(b"toolchain", "toolchain", &mut wrong),
        OperationOutcome::AccountingFault(AccountingFault::SiteDimensionMismatch {
            site,
            declared: BudgetDimension::Operation,
            required: BudgetDimension::ElapsedTime,
        }) if site == SiteId::new("request/unit/deadline")
    ));
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-009/C-29: prechecked count and allocation refusal is a different
//! mechanism from monotonic deadline observation, and the two share no code.
//!
//! Scope note. This file covers NFR-002-AC-1 (inclusive count and allocation
//! boundaries, refused before allocating) and NFR-002-AC-2 (monotonic deadline
//! before and after each indivisible unit, honest overshoot). NFR-002-AC-3
//! (bidirectional source-site/inventory reconciliation) belongs to C-30 and
//! NFR-002-AC-4 (action-capable double) to the assessment chunks; neither is
//! claimed here.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::time::{Duration, Instant, SystemTime};

use ix_trace_rs::trace;
use quire_protocol::bounds::{
    accumulation, limit_for, usage_for, Accumulation, BoundedRun, BoundedSite, Clock, CountGuard,
    DeadlineCode, DeadlineGuard, DeadlineRefusal, DurationOverflow, Phase, PrecheckCode,
    PrecheckRefusal, RegistryError, Reservation, RunStop, SiteId, SiteKind, SiteRegistry,
    UnitLabel, ALL_DIMENSIONS, ALL_PHASES, ALL_SITE_KINDS,
};
use quire_spec_language::protocol_artifact::{Dimension, Limits};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// An allocation site: reserves a definition table sized by the caller.
const TABLE: SiteId = SiteId::new("c29.fixture.definition-table");
/// A traversal site: walks caller-controlled graph edges.
const WALK: SiteId = SiteId::new("c29.fixture.reference-walk");
/// Declared but never registered, to exercise the inventory hook.
const UNLISTED: SiteId = SiteId::new("c29.fixture.unlisted");

const UNIT: UnitLabel = UnitLabel::new("c29.fixture.indivisible-unit");

/// Ceilings low enough to reach in a test. `Limits::bounded()` only clamps
/// downwards, so every lowered value below survives it unchanged.
fn test_limits() -> Limits {
    Limits {
        definitions: 2,
        references: 3,
        ..Limits::default()
    }
}

fn registry() -> SiteRegistry {
    let mut registry = SiteRegistry::new();
    registry
        .register(BoundedSite::new(
            TABLE,
            Dimension::Definitions,
            SiteKind::Allocation,
        ))
        .expect("fresh registry accepts the allocation site");
    registry
        .register(BoundedSite::new(
            WALK,
            Dimension::References,
            SiteKind::Traversal,
        ))
        .expect("fresh registry accepts the traversal site");
    registry
}

fn guard() -> CountGuard {
    CountGuard::new(test_limits(), registry())
}

/// A clock driven by a written script.
///
/// It carries a wall-clock script alongside the monotonic one so that the
/// wall-clock property below is a real comparison and not an assertion about
/// nothing: the wall-clock readings exist, they move, and [`Clock`] still has
/// no way to hand any of them to the guard.
struct ScriptedClock {
    monotonic: Vec<Instant>,
    wall: Vec<SystemTime>,
    index: Cell<usize>,
}

impl ScriptedClock {
    fn new(origin: Instant, monotonic_offsets: &[Duration], wall: Vec<SystemTime>) -> Self {
        assert_eq!(
            monotonic_offsets.len(),
            wall.len(),
            "each reading needs both a monotonic and a wall-clock value"
        );
        Self {
            monotonic: monotonic_offsets
                .iter()
                .map(|offset| origin + *offset)
                .collect(),
            wall,
            index: Cell::new(0),
        }
    }

    /// The wall-clock reading that accompanies the next monotonic reading.
    fn peek_wall(&self) -> SystemTime {
        self.wall[self.index.get().min(self.wall.len() - 1)]
    }
}

impl Clock for ScriptedClock {
    fn monotonic(&self) -> Instant {
        let index = self.index.get();
        self.index.set(index + 1);
        self.monotonic[index.min(self.monotonic.len() - 1)]
    }
}

/// A wall-clock script that only moves forward.
fn wall_forward(origin: SystemTime, count: usize) -> Vec<SystemTime> {
    (0..count)
        .map(|step| {
            origin + Duration::from_secs(u64::try_from(step).expect("a test step count fits u64"))
        })
        .collect()
}

/// The same script with a backwards step in the middle — an operator or NTP
/// setting the wall clock back an hour.
fn wall_jumped_back(origin: SystemTime, count: usize) -> Vec<SystemTime> {
    wall_forward(origin, count)
        .into_iter()
        .enumerate()
        .map(|(step, at)| {
            if step >= count / 2 {
                at - Duration::from_secs(3_600)
            } else {
                at
            }
        })
        .collect()
}

/// The same script with a forward step far past any test budget.
fn wall_jumped_forward(origin: SystemTime, count: usize) -> Vec<SystemTime> {
    wall_forward(origin, count)
        .into_iter()
        .enumerate()
        .map(|(step, at)| {
            if step >= count / 2 {
                at + Duration::from_secs(86_400)
            } else {
                at
            }
        })
        .collect()
}

/// One run's verdicts, recorded as stable code strings so two runs compare
/// without depending on any timing value.
fn run_verdicts(clock: &ScriptedClock, budget: Duration) -> Vec<String> {
    let mut deadline = DeadlineGuard::start(clock, budget);
    let mut verdicts = Vec::new();
    for _ in 0..2 {
        // Record the wall clock that accompanies this observation, to show it
        // is moving while the verdicts are not responding to it.
        let _ = clock.peek_wall();
        match deadline.begin_unit(clock, UNIT) {
            Ok(token) => {
                verdicts.push("admitted".to_owned());
                match deadline.end_unit(clock, token) {
                    Ok(_) => verdicts.push("closed".to_owned()),
                    Err(refusal) => verdicts.push(refusal.code().as_str().to_owned()),
                }
            }
            Err(refusal) => verdicts.push(refusal.code().as_str().to_owned()),
        }
    }
    verdicts
}

// ---------------------------------------------------------------------------
// NFR-002-AC-1 — inclusive count and allocation boundaries
// ---------------------------------------------------------------------------

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_count_limit_admits_boundary_refuses_boundary_plus_one() {
    let mut guard = guard();
    let limit = limit_for(guard.limits(), Dimension::References);
    assert_eq!(limit, 3, "the lowered ceiling survives Limits::bounded()");
    assert_eq!(
        accumulation(Dimension::References),
        Accumulation::Cumulative
    );

    // Below.
    let below = guard
        .charge(WALK, 1)
        .expect("below the ceiling is admitted");
    assert_eq!(below.granted(), 1);
    assert_eq!(usage_for(guard.usage(), Dimension::References), 1);

    // Exactly at the inclusive boundary.
    let _ = guard
        .charge(WALK, 2)
        .expect("the exact ceiling is admitted");
    assert_eq!(usage_for(guard.usage(), Dimension::References), 3);

    // The smallest representable excess unit.
    let refusal = guard
        .charge(WALK, 1)
        .expect_err("one unit past the ceiling is refused");
    assert_eq!(
        refusal,
        PrecheckRefusal::CountExceeded {
            site: WALK,
            dimension: Dimension::References,
            used: 3,
            requested: 1,
            limit: 3,
        }
    );
    assert_eq!(refusal.code(), PrecheckCode::CountExceeded);
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_allocation_site_refuses_on_its_own_code() {
    let mut guard = guard();
    assert_eq!(accumulation(Dimension::Definitions), Accumulation::Peak);

    let _ = guard.charge(TABLE, 1).expect("below the ceiling");
    let _ = guard.charge(TABLE, 2).expect("the exact ceiling");

    let refusal = guard
        .charge(TABLE, 3)
        .expect_err("one record past the ceiling is refused");
    assert_eq!(
        refusal,
        PrecheckRefusal::AllocationExceeded {
            site: TABLE,
            dimension: Dimension::Definitions,
            used: 2,
            requested: 3,
            limit: 2,
        }
    );
    assert_eq!(refusal.code(), PrecheckCode::AllocationExceeded);
    assert_ne!(
        refusal.code(),
        PrecheckCode::CountExceeded,
        "an allocation site does not borrow the counting code"
    );
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_refusal_precedes_allocation() {
    let mut guard = guard();
    let _ = guard.charge(TABLE, 2).expect("the exact ceiling");
    let before = *guard.usage();

    let refusal = guard.charge(TABLE, 3).expect_err("refused");
    assert_eq!(refusal.code(), PrecheckCode::AllocationExceeded);
    assert_eq!(
        *guard.usage(),
        before,
        "a refused request charges nothing at all"
    );

    // The only route to reserved memory is a granted Reservation, so no memory
    // exists to have been reserved before the refusal.
    let granted: Reservation = guard.precheck(TABLE, 2).expect("within the ceiling");
    let reserved: Vec<u8> = granted.with_capacity();
    assert!(reserved.capacity() >= granted.granted());
    assert!(reserved.is_empty());
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_counter_overflow_is_its_own_code() {
    let mut guard = guard();
    let _ = guard.charge(WALK, 1).expect("below the ceiling");

    let refusal = guard
        .charge(WALK, usize::MAX)
        .expect_err("an increment that overflows the accumulator is refused");
    assert_eq!(
        refusal,
        PrecheckRefusal::CounterOverflow {
            site: WALK,
            dimension: Dimension::References,
            used: 1,
            requested: usize::MAX,
        }
    );
    assert_eq!(refusal.code(), PrecheckCode::CounterOverflow);
    assert_eq!(
        usage_for(guard.usage(), Dimension::References),
        1,
        "the overflowing request charged nothing"
    );
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_unregistered_site_cannot_do_bounded_work() {
    let mut guard = guard();
    let refusal = guard
        .charge(UNLISTED, 1)
        .expect_err("an undeclared site has no bound, so it does no work");
    assert_eq!(
        refusal,
        PrecheckRefusal::UnregisteredSite { site: UNLISTED }
    );
    assert_eq!(refusal.code(), PrecheckCode::UnregisteredSite);
    assert_eq!(refusal.site(), UNLISTED);
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_duplicate_site_registration_is_not_a_refusal_code() {
    let mut registry = registry();
    let duplicate = registry
        .register(BoundedSite::new(
            WALK,
            Dimension::References,
            SiteKind::Traversal,
        ))
        .expect_err("a name may be registered once");
    assert_eq!(duplicate, RegistryError::DuplicateSite { site: WALK });
    assert_eq!(registry.sites().len(), 2, "the duplicate was not recorded");

    // A B-internal declaration fault is a different type from a caller's
    // over-large request; there is no code they can share.
    let declared: BTreeSet<&'static str> = registry
        .sites()
        .iter()
        .map(|site| site.id().as_str())
        .collect();
    assert_eq!(declared.len(), 2);
}

// ---------------------------------------------------------------------------
// NFR-002-AC-2 — monotonic deadline observation
// ---------------------------------------------------------------------------

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_deadline_refusal_is_a_distinct_code() {
    let origin = Instant::now();
    let clock = ScriptedClock::new(
        origin,
        &[
            Duration::ZERO,             // start
            Duration::from_millis(10),  // before unit 1
            Duration::from_millis(20),  // after unit 1
            Duration::from_millis(400), // before unit 2 — past the budget
        ],
        wall_forward(SystemTime::UNIX_EPOCH, 4),
    );
    let budget = Duration::from_millis(100);
    let mut deadline = DeadlineGuard::start(&clock, budget);

    let token = deadline.begin_unit(&clock, UNIT).expect("within budget");
    deadline.end_unit(&clock, token).expect("within budget");

    let refusal = deadline
        .begin_unit(&clock, UNIT)
        .expect_err("no later unit starts after expiry");
    assert_eq!(
        refusal,
        DeadlineRefusal::Expired {
            unit: UNIT,
            phase: Phase::BeforeUnit,
            budget,
            observed: Duration::from_millis(400),
            overshoot: Duration::from_millis(300),
        }
    );
    assert_eq!(refusal.code(), DeadlineCode::Expired);
    assert_eq!(refusal.phase(), Phase::BeforeUnit);
    assert_eq!(refusal.unit(), UNIT);
}

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_overshoot_is_reported_not_clamped() {
    let origin = Instant::now();
    let clock = ScriptedClock::new(
        origin,
        &[
            Duration::ZERO,               // start
            Duration::from_millis(90),    // before the unit — still inside
            Duration::from_millis(5_090), // after a deliberately long unit
        ],
        wall_forward(SystemTime::UNIX_EPOCH, 3),
    );
    let budget = Duration::from_millis(100);
    let mut deadline = DeadlineGuard::start(&clock, budget);

    let token = deadline
        .begin_unit(&clock, UNIT)
        .expect("the unit started inside the budget");
    let refusal = deadline
        .end_unit(&clock, token)
        .expect_err("the indivisible unit ran past the budget");

    let DeadlineRefusal::Expired {
        phase,
        observed,
        overshoot,
        ..
    } = refusal
    else {
        panic!("an overrunning unit expires; it does not fault the clock");
    };
    assert_eq!(phase, Phase::AfterUnit);
    assert_eq!(observed, Duration::from_millis(5_090));
    assert_eq!(overshoot, Duration::from_millis(4_990));
    assert!(
        overshoot > Duration::ZERO,
        "a measured overshoot is never clamped to zero"
    );
    assert_eq!(
        deadline.observed(),
        Duration::from_millis(5_090),
        "the guard retains the elapsed time it measured, not the budget"
    );
}

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_deadline_is_checked_before_and_after_every_unit() {
    assert_eq!(ALL_PHASES.len(), 2);
    let origin = Instant::now();

    // Expiry observable at the "before" phase.
    let before_clock = ScriptedClock::new(
        origin,
        &[Duration::ZERO, Duration::from_millis(500)],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    let mut before_guard = DeadlineGuard::start(&before_clock, Duration::from_millis(100));
    let before = before_guard
        .begin_unit(&before_clock, UNIT)
        .expect_err("expired before the unit");
    assert_eq!(before.phase(), Phase::BeforeUnit);

    // Expiry observable at the "after" phase.
    let after_clock = ScriptedClock::new(
        origin,
        &[
            Duration::ZERO,
            Duration::from_millis(10),
            Duration::from_millis(500),
        ],
        wall_forward(SystemTime::UNIX_EPOCH, 3),
    );
    let mut after_guard = DeadlineGuard::start(&after_clock, Duration::from_millis(100));
    let token = after_guard
        .begin_unit(&after_clock, UNIT)
        .expect("started inside the budget");
    let after = after_guard
        .end_unit(&after_clock, token)
        .expect_err("expired after the unit");
    assert_eq!(after.phase(), Phase::AfterUnit);

    assert_eq!(before.code(), after.code());
    assert_ne!(
        before.phase(),
        after.phase(),
        "the two mandatory observations stay distinguishable"
    );
}

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_budget_boundary_is_inclusive() {
    let origin = Instant::now();
    let budget = Duration::from_millis(100);

    let at = ScriptedClock::new(
        origin,
        &[Duration::ZERO, budget],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    let mut at_guard = DeadlineGuard::start(&at, budget);
    let _ = at_guard
        .begin_unit(&at, UNIT)
        .expect("elapsed exactly equal to the budget has not expired");

    let over = ScriptedClock::new(
        origin,
        &[Duration::ZERO, budget + Duration::from_nanos(1)],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    let mut over_guard = DeadlineGuard::start(&over, budget);
    let refusal = over_guard
        .begin_unit(&over, UNIT)
        .expect_err("one nanosecond past the budget has expired");
    let DeadlineRefusal::Expired { overshoot, .. } = refusal else {
        panic!("expiry, not a clock fault");
    };
    assert_eq!(overshoot, Duration::from_nanos(1));
}

// ---------------------------------------------------------------------------
// Monotonicity: a wall-clock jump neither manufactures nor masks a refusal
// ---------------------------------------------------------------------------

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_wall_clock_jump_changes_no_verdict() {
    let origin = Instant::now();
    let wall_origin = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let budget = Duration::from_millis(100);

    // One monotonic script: unit 1 inside the budget, unit 2 past it.
    let monotonic = [
        Duration::ZERO,
        Duration::from_millis(10),
        Duration::from_millis(20),
        Duration::from_millis(30),
        Duration::from_millis(900),
    ];
    let readings = monotonic.len();

    let forward = ScriptedClock::new(origin, &monotonic, wall_forward(wall_origin, readings));
    let baseline = run_verdicts(&forward, budget);
    assert_eq!(
        baseline,
        vec!["admitted", "closed", "admitted", "bounds.deadline.expired"],
        "the monotonic script alone decides the verdicts"
    );

    for (name, wall) in [
        ("jumped back", wall_jumped_back(wall_origin, readings)),
        ("jumped forward", wall_jumped_forward(wall_origin, readings)),
    ] {
        let jumped = ScriptedClock::new(origin, &monotonic, wall.clone());

        // The wall clock really does move — backwards in the first case, a day
        // forward in the second — so this is a comparison, not a tautology.
        let moved_backwards = wall.windows(2).any(|pair| pair[1] < pair[0]);
        let moved_far_forward = wall
            .windows(2)
            .any(|pair| pair[1] > pair[0] + Duration::from_secs(3_600));
        assert!(
            moved_backwards || moved_far_forward,
            "{name}: the wall-clock script must actually jump"
        );

        assert_eq!(
            run_verdicts(&jumped, budget),
            baseline,
            "{name}: a wall-clock jump neither manufactured nor masked a refusal"
        );
    }
}

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_regressing_monotonic_reading_is_not_an_expiry() {
    let origin = Instant::now();
    let clock = ScriptedClock::new(
        origin,
        &[
            Duration::from_millis(500), // start
            Duration::from_millis(600), // before the unit
            Duration::from_millis(200), // after the unit — earlier than the start
        ],
        wall_forward(SystemTime::UNIX_EPOCH, 3),
    );
    let mut deadline = DeadlineGuard::start(&clock, Duration::from_secs(60));

    let token = deadline
        .begin_unit(&clock, UNIT)
        .expect("inside the budget");
    let refusal = deadline
        .end_unit(&clock, token)
        .expect_err("a backwards reading is a fault, not a result");

    assert_eq!(
        refusal,
        DeadlineRefusal::ClockRegressed {
            unit: UNIT,
            phase: Phase::AfterUnit,
            previous: Duration::from_millis(100),
        }
    );
    assert_eq!(refusal.code(), DeadlineCode::ClockRegressed);
    assert_ne!(
        refusal.code(),
        DeadlineCode::Expired,
        "a broken clock is never reported as an exhausted budget"
    );
}

// ---------------------------------------------------------------------------
// The two mechanisms cannot be confused
// ---------------------------------------------------------------------------

#[trace("TC-009", "NFR-002-AC-1", "NFR-002-AC-2")]
#[test]
fn tc_009_precheck_and_deadline_codes_are_disjoint() {
    // Enumerated through exhaustive matches, so a variant added to either enum
    // without a code of its own fails to compile rather than falling through.
    let precheck: BTreeSet<&'static str> = PrecheckCode::ALL
        .iter()
        .map(|code| match code {
            PrecheckCode::CountExceeded
            | PrecheckCode::AllocationExceeded
            | PrecheckCode::CounterOverflow
            | PrecheckCode::UnregisteredSite => code.as_str(),
        })
        .collect();
    let deadline: BTreeSet<&'static str> = DeadlineCode::ALL
        .iter()
        .map(|code| match code {
            DeadlineCode::Expired | DeadlineCode::ClockRegressed => code.as_str(),
        })
        .collect();

    assert_eq!(
        precheck.len(),
        PrecheckCode::ALL.len(),
        "no duplicate codes"
    );
    assert_eq!(
        deadline.len(),
        DeadlineCode::ALL.len(),
        "no duplicate codes"
    );
    assert!(
        precheck.is_disjoint(&deadline),
        "the two mechanisms share no code string"
    );

    // Namespaces make the disjointness structural rather than incidental.
    assert!(precheck
        .iter()
        .all(|code| code.starts_with(PrecheckCode::NAMESPACE)));
    assert!(deadline
        .iter()
        .all(|code| code.starts_with(DeadlineCode::NAMESPACE)));
    assert!(!PrecheckCode::NAMESPACE.starts_with(DeadlineCode::NAMESPACE));
    assert!(!DeadlineCode::NAMESPACE.starts_with(PrecheckCode::NAMESPACE));
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_no_count_path_can_produce_a_deadline_code() {
    let deadline: BTreeSet<&'static str> =
        DeadlineCode::ALL.iter().map(|code| code.as_str()).collect();

    // Every distinguishable count/allocation outcome this module can reach.
    let mut observed = BTreeSet::new();
    let mut guard = guard();
    let _ = guard.charge(WALK, 1).expect("below the ceiling");
    for refusal in [
        guard.charge(WALK, 9_999).expect_err("count exceeded"),
        guard
            .charge(WALK, usize::MAX)
            .expect_err("counter overflow"),
        guard.charge(TABLE, 9_999).expect_err("allocation exceeded"),
        guard.charge(UNLISTED, 1).expect_err("unregistered site"),
    ] {
        let code = refusal.code().as_str();
        assert!(
            !deadline.contains(code),
            "a count path produced the deadline code {code}"
        );
        assert!(code.starts_with(PrecheckCode::NAMESPACE));
        observed.insert(code);
    }
    assert_eq!(
        observed.len(),
        PrecheckCode::ALL.len(),
        "every precheck code was reached by a real count path"
    );
}

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_no_deadline_path_can_produce_a_count_code() {
    let precheck: BTreeSet<&'static str> =
        PrecheckCode::ALL.iter().map(|code| code.as_str()).collect();
    let origin = Instant::now();

    let expired = ScriptedClock::new(
        origin,
        &[Duration::ZERO, Duration::from_secs(9)],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    let mut expired_guard = DeadlineGuard::start(&expired, Duration::from_millis(1));
    let regressed = ScriptedClock::new(
        origin,
        &[Duration::from_secs(9), Duration::ZERO],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    let mut regressed_guard = DeadlineGuard::start(&regressed, Duration::from_secs(60));

    let mut observed = BTreeSet::new();
    for refusal in [
        expired_guard
            .begin_unit(&expired, UNIT)
            .expect_err("expired"),
        regressed_guard
            .begin_unit(&regressed, UNIT)
            .expect_err("regressed"),
    ] {
        let code = refusal.code().as_str();
        assert!(
            !precheck.contains(code),
            "a deadline path produced the count code {code}"
        );
        assert!(code.starts_with(DeadlineCode::NAMESPACE));
        observed.insert(code);
    }
    assert_eq!(
        observed.len(),
        DeadlineCode::ALL.len(),
        "every deadline code was reached by a real clock path"
    );
}

#[trace("TC-009", "NFR-002-AC-1", "NFR-002-AC-2")]
#[test]
fn tc_009_simultaneous_exhaustion_keeps_both_facts_separate() {
    let origin = Instant::now();
    let clock = ScriptedClock::new(
        origin,
        &[Duration::ZERO, Duration::from_secs(9)],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    // Both are exhausted at once: the count ceiling is zero-headroom and the
    // budget is already gone.
    let mut counts = guard();
    let _ = counts.charge(WALK, 3).expect("the exact ceiling");
    let deadline = DeadlineGuard::start(&clock, Duration::from_millis(1));
    let mut run = BoundedRun::new(counts, deadline);

    let stop = run
        .admit_unit(&clock, UNIT, WALK, 1)
        .expect_err("the run stops");

    // The deadline is observed first, so the stop reports the fact that is
    // actually true of the run — and the count guard is left unspent rather
    // than reporting a refusal it never made.
    let RunStop::Deadline(refusal) = stop else {
        panic!("out of time is reported as out of time, not as too much work");
    };
    assert_eq!(refusal.code(), DeadlineCode::Expired);
    assert_eq!(stop.code_str(), DeadlineCode::Expired.as_str());
    assert_eq!(
        usage_for(run.counts().usage(), Dimension::References),
        3,
        "the refused unit charged nothing"
    );

    // With time to spare, the same over-large request stops on the count arm.
    let roomy = ScriptedClock::new(
        origin,
        &[Duration::ZERO, Duration::from_millis(1)],
        wall_forward(SystemTime::UNIX_EPOCH, 2),
    );
    let mut counts = guard();
    let _ = counts.charge(WALK, 3).expect("the exact ceiling");
    let mut roomy_run = BoundedRun::new(
        counts,
        DeadlineGuard::start(&roomy, Duration::from_secs(60)),
    );
    let stop = roomy_run
        .admit_unit(&roomy, UNIT, WALK, 1)
        .expect_err("the count ceiling stops it");
    let RunStop::Count(refusal) = stop else {
        panic!("too much work is reported as too much work, not as out of time");
    };
    assert_eq!(refusal.code(), PrecheckCode::CountExceeded);
    assert_eq!(stop.code_str(), PrecheckCode::CountExceeded.as_str());
}

// ---------------------------------------------------------------------------
// Checked conversions and A reconciliation
// ---------------------------------------------------------------------------

#[trace("TC-009", "NFR-002-AC-2")]
#[test]
fn tc_009_persisted_elapsed_time_is_checked_not_truncated() {
    let representable = DeadlineRefusal::Expired {
        unit: UNIT,
        phase: Phase::AfterUnit,
        budget: Duration::from_millis(1),
        observed: Duration::from_millis(2),
        overshoot: Duration::from_millis(1),
    };
    assert_eq!(representable.observed_nanos(), Ok(2_000_000));

    let unrepresentable = DeadlineRefusal::Expired {
        unit: UNIT,
        phase: Phase::AfterUnit,
        budget: Duration::from_millis(1),
        observed: Duration::MAX,
        overshoot: Duration::MAX - Duration::from_millis(1),
    };
    assert_eq!(
        unrepresentable.observed_nanos(),
        Err(DurationOverflow {
            observed: Duration::MAX
        }),
        "an elapsed time too large for u64 nanoseconds refuses rather than wrapping"
    );
}

#[trace("TC-009", "NFR-002-AC-1")]
#[test]
fn tc_009_every_a_dimension_has_a_reconciled_accumulation() {
    assert_eq!(ALL_DIMENSIONS.len(), 13, "A declares thirteen dimensions");
    let limits = Limits::default();
    for dimension in ALL_DIMENSIONS {
        assert!(
            limit_for(&limits, dimension) > 0,
            "{dimension:?} has a default ceiling"
        );
    }

    // The discipline is asserted per dimension, not as a pair of totals.
    //
    // Counting nine peaks and four cumulative says nothing about which nine.
    // If A were to make `Depth` cumulative and `ContentBytes` a peak, both of
    // B's entries would be wrong, the totals would still read 9 and 4, and this
    // test would still pass while B's precheck and A's in-work accounting
    // disagreed about two budgets.
    //
    // This table is still a mirror of a discipline A does not publish — see the
    // module documentation on `bounds` — so it cannot fail when A changes. What
    // it does is make B's half of the reconciliation an explicit per-dimension
    // statement that has to be edited deliberately, rather than a total that
    // absorbs a swap.
    let expected: [(Dimension, Accumulation); 13] = [
        (Dimension::PayloadBytes, Accumulation::Peak),
        (Dimension::OutputBytes, Accumulation::Peak),
        (Dimension::SourceBytes, Accumulation::Peak),
        (Dimension::ContentBytes, Accumulation::Cumulative),
        (Dimension::Sources, Accumulation::Peak),
        (Dimension::Dependencies, Accumulation::Peak),
        (Dimension::Definitions, Accumulation::Peak),
        (Dimension::Models, Accumulation::Peak),
        (Dimension::Declarations, Accumulation::Peak),
        (Dimension::Entries, Accumulation::Cumulative),
        (Dimension::References, Accumulation::Cumulative),
        (Dimension::ByteWork, Accumulation::Cumulative),
        (Dimension::Depth, Accumulation::Peak),
    ];
    let declared: Vec<Dimension> = expected.iter().map(|(dimension, _)| *dimension).collect();
    assert_eq!(
        declared,
        ALL_DIMENSIONS.to_vec(),
        "the reconciliation table must cover A's dimensions in A's declared order"
    );
    for (dimension, discipline) in expected {
        assert_eq!(
            accumulation(dimension),
            discipline,
            "{dimension:?} is reconciled against A with the wrong discipline"
        );
    }
    assert_eq!(ALL_SITE_KINDS.len(), 5, "NFR-002 names five site kinds");
}

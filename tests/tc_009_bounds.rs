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

#[cfg(any())]
mod retired_parallel_guard {
    use super::*;
    use std::cell::Cell;
    use std::collections::BTreeSet;
    use std::time::{Duration, Instant, SystemTime};

    use ix_trace_rs::trace;
    use quire_protocol::bounds::inventory::{
        production_entries, production_inventory, production_sources, AccountingFault,
        BoundaryCase, Budget, BudgetDimension, BudgetError, ChargeError, ChargePoint,
        IncompleteCode, Inventory, InventoryEntry, InventoryError, LimitAuthority,
        OperationOutcome, ResourceAccountant, SourceSite, Termination,
    };
    use quire_protocol::bounds::{
        accumulation, limit_for, usage_for, Accumulation, BoundedRun, BoundedSite, Clock,
        CountGuard, DeadlineCode, DeadlineGuard, DeadlineRefusal, DurationOverflow, Phase,
        PrecheckCode, PrecheckRefusal, RegistryError, Reservation, RunStop, SiteId, SiteKind,
        SiteRegistry, UnitLabel, ALL_DIMENSIONS, ALL_PHASES, ALL_SITE_KINDS,
    };
    use quire_protocol::repro::ToolchainIdentity;
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

    /// An allocation site on a byte-counted dimension, so the granted unit is the
    /// byte rather than the record.
    const PAYLOAD: SiteId = SiteId::new("c29.fixture.payload-buffer");

    const UNIT: UnitLabel = UnitLabel::new("c29.fixture.indivisible-unit");

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
            .register(BoundedSite::new(
                PAYLOAD,
                Dimension::PayloadBytes,
                SiteKind::Allocation,
            ))
            .expect("fresh registry accepts the byte-counted allocation site");
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
                origin
                    + Duration::from_secs(u64::try_from(step).expect("a test step count fits u64"))
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

        // A precheck answers "would this fit?" and grants nothing: it charges
        // nothing, so it must not hand back permission to allocate.
        let admitted = guard.precheck(TABLE, 2).expect("within the ceiling");
        assert_eq!(admitted.admissible(), 2);
        assert_eq!(
            *guard.usage(),
            before,
            "a precheck charges nothing, so it cannot fund an allocation"
        );

        // The only route to reserved memory is a charge, which debits the budget.
        let granted: Reservation = guard.charge(TABLE, 2).expect("within the ceiling");
        let capacity = granted.granted();
        let reserved: Vec<u8> = granted.with_capacity().expect("one byte per unit");
        assert!(reserved.capacity() >= capacity);
        assert!(reserved.is_empty());
    }

    /// One grant is one allocation, and a byte budget is not an element budget.
    ///
    /// The grant used to be `Copy` with `with_capacity(&self)`, so a single charge
    /// of four units backed a thousand allocations of four — the ceiling enforced
    /// once and then ignored. `precheck` was worse: it charged nothing and returned
    /// a fully powered token, so a loop over it reserved without ever debiting.
    #[trace("TC-009", "NFR-002-AC-1")]
    #[test]
    fn tc_009_one_grant_is_one_allocation() {
        let mut counts = guard();
        let granted = counts.charge(TABLE, 2).expect("within the ceiling");
        let _first: Vec<u8> = granted.with_capacity().expect("one byte per unit");
        // `granted` is moved: a second allocation from the same grant does not
        // compile. The compile_fail doctest on `Reservation::with_capacity` is the
        // standing proof; here the budget is checked instead.
        assert_eq!(usage_for(counts.usage(), Dimension::Definitions), 2);

        // A byte-counted dimension grants bytes. Reserving a wider element would
        // multiply past the ceiling just enforced, so it is refused.
        let mut bytes = guard();
        let granted = bytes.charge(PAYLOAD, 8).expect("within the byte ceiling");
        let refusal = granted
            .with_capacity::<u64>()
            .expect_err("eight bytes is not eight u64s");
        assert_eq!(refusal.code(), PrecheckCode::AllocationUnitMismatch);

        let granted = bytes.charge(PAYLOAD, 8).expect("within the byte ceiling");
        assert!(granted.with_capacity::<u8>().is_ok(), "bytes fit bytes");
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
        assert_eq!(registry.sites().len(), 3, "the duplicate was not recorded");

        // A B-internal declaration fault is a different type from a caller's
        // over-large request; there is no code they can share.
        let declared: BTreeSet<&'static str> = registry
            .sites()
            .iter()
            .map(|site| site.id().as_str())
            .collect();
        assert_eq!(declared.len(), 3);
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
                | PrecheckCode::AllocationUnitMismatch
                | PrecheckCode::AllocationUnavailable
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
        // The two allocation codes are reached through a granted reservation rather
        // than through `charge`, so they are exercised here too — all but one.
        let mut byte_guard = CountGuard::new(test_limits(), registry());
        let granted = byte_guard
            .charge(PAYLOAD, 8)
            .expect("within the byte ceiling");
        let mismatch = granted
            .with_capacity::<u64>()
            .expect_err("a byte budget is not a u64 budget");
        assert!(mismatch
            .code()
            .as_str()
            .starts_with(PrecheckCode::NAMESPACE));
        observed.insert(mismatch.code().as_str());

        // `AllocationUnavailable` reports an allocator that refused a reservation
        // the ceiling admitted. There is no portable way to force that, so it is
        // named as the one unreached code rather than left to make the count look
        // complete. A code added without a path fails here.
        let unreached: Vec<&str> = PrecheckCode::ALL
            .iter()
            .map(|code| code.as_str())
            .filter(|code| !observed.contains(code))
            .collect();
        assert_eq!(
            unreached,
            [PrecheckCode::AllocationUnavailable.as_str()],
            "every precheck code but the allocator-failure one is reached by a real path"
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

    /// Every dimension reads its own `Limits` and `Usage` field.
    ///
    /// `limit_for` and `usage_for` hand-mirror thirteen dimension-to-field bindings
    /// of A's private `Work::fields`. A mis-binding compiles and is invisible: five
    /// dimensions default to 10 000 and three to 8 MiB, so swapping any pair with
    /// equal defaults changed nothing any test could see. Confirmed before this
    /// test existed — pointing `Models` at `declarations` left all eighteen green.
    ///
    /// This needs nothing from A. Giving every field a distinct value makes each
    /// binding observable, which is a different gap from the peak/cumulative
    /// discipline the module documents as unverifiable.
    #[trace("TC-009", "NFR-002-AC-1")]
    #[test]
    fn tc_009_every_dimension_reads_its_own_field() {
        // Thirteen distinct values, all below `Limits::default()` so `bounded()`
        // preserves them.
        let limits = Limits {
            payload_bytes: 101,
            output_bytes: 102,
            source_bytes: 103,
            content_bytes: 104,
            sources: 105,
            dependencies: 106,
            definitions: 107,
            models: 108,
            declarations: 109,
            entries: 110,
            references: 111,
            byte_work: 112,
            depth: 13,
        };
        let expected = [
            (Dimension::PayloadBytes, 101),
            (Dimension::OutputBytes, 102),
            (Dimension::SourceBytes, 103),
            (Dimension::ContentBytes, 104),
            (Dimension::Sources, 105),
            (Dimension::Dependencies, 106),
            (Dimension::Definitions, 107),
            (Dimension::Models, 108),
            (Dimension::Declarations, 109),
            (Dimension::Entries, 110),
            (Dimension::References, 111),
            (Dimension::ByteWork, 112),
            (Dimension::Depth, 13),
        ];
        assert_eq!(expected.len(), ALL_DIMENSIONS.len());
        for (dimension, value) in expected {
            assert_eq!(
                limit_for(&limits, dimension),
                value,
                "{dimension:?} reads the wrong Limits field"
            );
        }

        // The same for `usage_for`, driven through real charges so each dimension's
        // usage slot is written by the guard rather than constructed here.
        let mut registry = SiteRegistry::new();
        for (index, dimension) in ALL_DIMENSIONS.iter().enumerate() {
            let id = SITE_IDS[index];
            registry
                .register(BoundedSite::new(id, *dimension, SiteKind::Traversal))
                .expect("one site per dimension");
        }
        let mut guard = CountGuard::new(limits, registry);
        for (index, dimension) in ALL_DIMENSIONS.iter().enumerate() {
            let charged = index + 1;
            let _ = guard
                .charge(SITE_IDS[index], charged)
                .expect("well inside every ceiling");
            assert_eq!(
                usage_for(guard.usage(), *dimension),
                charged,
                "{dimension:?} reads the wrong Usage field"
            );
        }
    }

    /// One site per dimension, in `ALL_DIMENSIONS` order.
    const SITE_IDS: [SiteId; 13] = [
        SiteId::new("c29.fixture.d00"),
        SiteId::new("c29.fixture.d01"),
        SiteId::new("c29.fixture.d02"),
        SiteId::new("c29.fixture.d03"),
        SiteId::new("c29.fixture.d04"),
        SiteId::new("c29.fixture.d05"),
        SiteId::new("c29.fixture.d06"),
        SiteId::new("c29.fixture.d07"),
        SiteId::new("c29.fixture.d08"),
        SiteId::new("c29.fixture.d09"),
        SiteId::new("c29.fixture.d10"),
        SiteId::new("c29.fixture.d11"),
        SiteId::new("c29.fixture.d12"),
    ];
} // retired_parallel_guard

use std::cell::Cell;
use std::collections::BTreeSet;
use std::time::{Duration, Instant, SystemTime};

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::{
    production_entries, production_inventory, production_sources, AccountingFault, BoundaryCase,
    Budget, BudgetDimension, BudgetError, ChargeError, ChargePoint, IncompleteCode, Inventory,
    InventoryEntry, InventoryError, LimitAuthority, OperationOutcome, ResourceAccountant,
    SourceSite, Termination,
};
use quire_protocol::bounds::{
    Clock, DeadlineCode, DeadlineGuard, Phase, SiteId, SiteKind, UnitLabel,
};
use quire_protocol::repro::ToolchainIdentity;

const UNLISTED: SiteId = SiteId::new("c30.fixture.unlisted");
const UNIT: UnitLabel = UnitLabel::new("c30.fixture.indivisible-unit");
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
        .expect("fixture inventory reconciles")
}
struct ScriptedClock {
    monotonic: Vec<Instant>,
    _wall: Vec<SystemTime>,
    index: Cell<usize>,
}
impl ScriptedClock {
    fn new(origin: Instant, offsets: &[Duration], wall: Vec<SystemTime>) -> Self {
        assert_eq!(offsets.len(), wall.len());
        Self {
            monotonic: offsets.iter().map(|offset| origin + *offset).collect(),
            _wall: wall,
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
fn wall_forward(origin: SystemTime, count: usize) -> Vec<SystemTime> {
    (0..count)
        .map(|step| origin + Duration::from_secs(step as u64))
        .collect()
}

// ---------------------------------------------------------------------------
// NFR-002-AC-3 / FR-016 — QProtocol caller-budget vocabulary and inventory
// ---------------------------------------------------------------------------

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
        wall_forward(SystemTime::now(), 3),
    );
    let mut deadline = DeadlineGuard::start(&clock, Duration::from_millis(100));
    let token = deadline
        .begin_unit(&clock, UNIT)
        .expect("90ms is inside a 100ms budget");
    let refusal = deadline
        .end_unit(&clock, token)
        .expect_err("50ms after 90ms is a clock that ran backwards");
    assert_eq!(
        refusal.code(),
        DeadlineCode::ClockRegressed,
        "a backwards reading is a fault, never an expiry"
    );
    assert_ne!(refusal.code(), DeadlineCode::Expired);
}

#[trace("TC-009", "NFR-002-AC-2", "FR-016-AC-6", "Task-008")]
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
        wall_forward(SystemTime::now(), 3),
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

#[trace("TC-009", "NFR-002-AC-2", "Task-008")]
#[test]
fn tc_009_production_deadline_stops_before_work_and_clock_regression_is_a_fault() {
    let origin = Instant::now();
    let inventory = production_inventory().expect("production inventory");
    let make_budget = || {
        Budget::nothing_granted()
            .grant(BudgetDimension::ElapsedTime, 100_000_000)
            .expect("100ms deadline")
    };

    let expired = ScriptedClock::new(
        origin,
        &[Duration::ZERO, Duration::from_millis(101)],
        wall_forward(SystemTime::now(), 2),
    );
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
        wall_forward(SystemTime::now(), 3),
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

#[trace("TC-009", "NFR-002-AC-2", "NFR-002-AC-3", "Task-008")]
#[test]
fn tc_009_production_deadline_requires_its_exact_reconciled_site() {
    let origin = Instant::now();
    let make_clock = || {
        ScriptedClock::new(
            origin,
            &[Duration::ZERO, Duration::ZERO, Duration::ZERO],
            wall_forward(SystemTime::now(), 3),
        )
    };
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

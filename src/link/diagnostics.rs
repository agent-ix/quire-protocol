// SPDX-License-Identifier: AGPL-3.0-or-later
//! Located construction-fatal diagnostics. Owner chunk: **C-13**.
//!
//! `Task-002` requires the linker to "accumulate independent located
//! diagnostics while every construction-fatal refusal returns no linked
//! representation". Those are two obligations, and they pull in opposite
//! directions: reporting more than one finding means continuing past the first,
//! and continuing past the first is how a half-built representation escapes.
//!
//! [`StaticDiagnostics`] is how both hold at once. The linker keeps going and records
//! what it finds, and the accumulated findings are the **error** half of its
//! result. There is no constructor that produces a [`StaticLinkedProtocol`] alongside
//! them, so "some diagnostics, and here is the graph anyway" is not a value this
//! module can express.
//!
//! [`StaticLinkedProtocol`]: super::StaticLinkedProtocol
//!
//! # What "independent" means
//!
//! Two checks are independent when neither reads the other's output. Those run
//! to completion and every failure is reported:
//!
//! - every repeat control's authored maximum,
//! - every role's model export against the ones already seen,
//! - every role's role-instance binding requirement,
//! - every channel's four binding premises — all four, not the first to fail.
//!
//! A check that consumes another's output is **dependent**, and a dependent
//! check is skipped rather than run on a value that was never established:
//!
//! - a channel's linked form is derived from premises that resolved, so a
//!   channel with a refused premise contributes nothing — the compiler enforces
//!   this, because the four premises are [`Option`]s and there is no channel to
//!   build without all four,
//! - the role-instance collision scan runs over the roles that resolved, so the
//!   absence of a collision proves nothing when a role was skipped,
//! - the declaration lookup and the protocol-body check gate the whole
//!   traversal: if the selected declaration is not there, or is not a protocol,
//!   there are no tables to walk and the result carries that one finding alone.
//!
//! So a [`StaticDiagnostics`] carrying *n* findings says n independent checks failed.
//! It does not say the package would link after n fixes: a dependent check that
//! never ran may fail on the next attempt. That is a property of the input, not
//! a weakness in the accumulation, and stating it is cheaper than implying a
//! completeness this cannot have.
//!
//! # Located
//!
//! A finding's location is A's [`Locus`] — the original source region the
//! producer recorded. It is A's type, imported, never re-spelled here.
//!
//! **Every finding about the submitted artifact carries one.** That is what
//! `FR-002` asks for — "independent *located* diagnostics established from the
//! submitted artifact" — and it is a property of this module rather than an
//! aspiration in its prose. Two earlier revisions fell short and argued the
//! shortfall was inherent:
//!
//! - the first left an out-of-range binding-requirement index unlocated,
//!   calling it a refusal about a "table position". The index is an authored
//!   field of the referring record and the referrer's own `locus` was already in
//!   hand at the site that checks it, so the region was never missing.
//! - the second left [`StaticLinkRefusal::CollidingIdentity`] unlocated on the same
//!   reasoning. Both colliding records are authored and both carry a region; the
//!   finding now names the later of the two, which is the record an author would
//!   change to resolve the collision.
//!
//! Two variants still return `None`, and neither is about the submitted
//! artifact. [`StaticLinkRefusal::NoSuchDeclaration`] and [`StaticLinkRefusal::NotAProtocol`]
//! refuse the **caller's own `declaration` argument** to [`link`] — a selection
//! made at the call site, not a record in the package. There is no authored text
//! to point at, and manufacturing a zero span, or borrowing the package's, would
//! put a reader's caret over source that is not what was refused.
//! [`StaticDiagnostics::located`] yields only the findings that genuinely carry a
//! region.
//!
//! [`link`]: super::link
//! [`StaticLinkRefusal::CollidingIdentity`]: super::StaticLinkRefusal::CollidingIdentity
//! [`StaticLinkRefusal::NoSuchDeclaration`]: super::StaticLinkRefusal::NoSuchDeclaration
//! [`StaticLinkRefusal::NotAProtocol`]: super::StaticLinkRefusal::NotAProtocol
//!
//! # How many findings one refusal may carry
//!
//! As many as the artifact has faulty records, and **no locally invented
//! ceiling**.
//!
//! This was not the first answer. A revision of this module capped the list at
//! a `CEILING` of 1 000 and set a truncation flag past it, on the reasoning
//! that A's `entries` limit of 100 000 makes a one-finding-per-record refusal
//! scale with the input. That reasoning was wrong twice. `entries: 100_000` is
//! A's `Limits::default()` — a grant the caller sets, not a cap — so the
//! constant was derived from another repository's default, which is the
//! mirroring `CLAUDE.md` forbids. And the constant itself is what `FR-016`
//! Absence rules out in as many words: an ungranted dimension "is never
//! defaulted to zero, to unbounded, to another dimension's grant, or to an
//! ambient universal threshold". `FR-016` declares no diagnostic-count
//! dimension, so under `CLAUDE.md` this is "the spec shows no owner → stop and
//! ask", not a gap to fill with a round number.
//!
//! What is true without a ceiling: the accumulator holds at most one finding per
//! record of an artifact A has **already admitted and already holds in memory**,
//! so it grows with an input that is itself bounded by the caller's `entries`
//! and `input-byte` grants. It is not an unbounded traversal of a
//! server-supplied cursor.
//!
//! What is still owed: [`Accumulator`] is a caller-controlled retention site,
//! and `NFR-002` requires every such site to appear in the resource inventory —
//! `crate::bounds::inventory`, owner chunk **C-30**, today an empty stub. When
//! C-30 lands, this site is charged through a granted dimension rather than
//! capped here. Naming the obligation is this module's job; inventing the
//! dimension is not.

use std::fmt;

use quire_spec_language::protocol_artifact::wire::Locus;

use super::StaticLinkRefusal;

/// One or more construction-fatal findings from a single link attempt.
///
/// Non-empty by construction, and by the *shape* of the struct rather than by a
/// convention this module remembers to keep: the first finding is a field, not
/// an element. [`StaticDiagnostics::first`] is therefore a field access, with nothing
/// to unwrap and no panic to reach. `NFR-008-AC-4` requires that no refusal path
/// panics, aborts, unwraps or expects in place of returning a value, and an
/// `expect` whose message asserts an invariant the type does not enforce is
/// exactly what that criterion is about.
///
/// Findings are in the order the linker established them, which is its own
/// traversal order over A's tables: controls, then roles, then channels. The
/// order is declared, not sorted — sorting would need a ranking over refusal
/// kinds, and no requirement gives one.
///
/// Equal findings are kept rather than collapsed. Two records that map onto one
/// identity are two records at fault, and reporting one would name a collision
/// without saying it happened twice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticDiagnostics {
    first: StaticLinkRefusal,
    rest: Vec<StaticLinkRefusal>,
}

impl StaticDiagnostics {
    /// A `StaticDiagnostics` carrying exactly one finding.
    ///
    /// Crate-internal: the linker establishes findings, and a consumer minting
    /// one would be manufacturing a refusal the linker never made.
    pub(crate) fn of(finding: StaticLinkRefusal) -> Self {
        Self {
            first: finding,
            rest: Vec::new(),
        }
    }

    /// The first finding established.
    #[must_use]
    pub const fn first(&self) -> &StaticLinkRefusal {
        &self.first
    }

    /// Every finding, in the order the linker established them.
    ///
    /// The only way to read past the first. There is no `rest` accessor: a
    /// caller wanting all of them wants all of them, and the split into a field
    /// and a vector is how non-emptiness is enforced, not a shape callers should
    /// have to know about.
    pub fn iter(&self) -> impl Iterator<Item = &StaticLinkRefusal> {
        std::iter::once(&self.first).chain(self.rest.iter())
    }

    /// How many independent checks failed.
    ///
    /// Named `count` rather than `len`: a `len` on a type that cannot be empty
    /// invites an `is_empty` that always answers `false`, which is a question
    /// with one answer dressed as a query.
    #[must_use]
    pub fn count(&self) -> usize {
        1 + self.rest.len()
    }

    /// Each finding that carries an original source region, with that region.
    ///
    /// The two findings that refuse the caller's `declaration` argument carry
    /// none and are absent here rather than present with an invented span; see
    /// the module documentation.
    pub fn located(&self) -> impl Iterator<Item = (&StaticLinkRefusal, &Locus)> {
        self.iter()
            .filter_map(|finding| finding.locus().map(|locus| (finding, locus)))
    }
}

impl fmt::Display for StaticDiagnostics {
    /// The findings, one per line, each preceded by its source region when it
    /// has one.
    ///
    /// A single finding renders as that finding's own text with no count and no
    /// list, so the common case reads as a sentence rather than as a report of
    /// length one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, finding) in self.iter().enumerate() {
            if position > 0 {
                writeln!(f)?;
            }
            match finding.locus() {
                Some(locus) => write!(f, "{}: {finding}", region(locus))?,
                None => write!(f, "{finding}")?,
            }
        }
        Ok(())
    }
}

impl std::error::Error for StaticDiagnostics {}

/// One source region, as the producer recorded it.
///
/// A's own coordinates, rendered and not reinterpreted. `source` is A's index
/// into the package's source table, not a path, and the span is A's own offset
/// pair: this crate does not resolve either against source bytes, and does not
/// claim a line and column A did not supply.
///
/// This is presentation. `FR-037` requires no refusal fact derived from text, so
/// a caller asking *where* asks [`StaticDiagnostics::located`] and reads the [`Locus`]
/// itself; this function exists so a human reading one line sees the region.
fn region(locus: &Locus) -> String {
    format!(
        "source {}, {}..{}",
        locus.source, locus.span.start, locus.span.end
    )
}

/// The linker's private accumulator.
///
/// Separate from [`StaticDiagnostics`] so that "nothing found yet" and "at least one
/// finding" are different types. The linker holds the first and can only hand
/// out the second, which is what keeps an empty `StaticDiagnostics` unconstructable
/// from outside as well as inside this crate.
#[derive(Debug, Default)]
pub(super) struct Accumulator {
    findings: Vec<StaticLinkRefusal>,
}

impl Accumulator {
    /// Record one independent finding and keep going.
    ///
    /// No cap, and no early return. See the module documentation: the bound
    /// belongs to the resource inventory C-30 owns, and stopping early here
    /// would make the *presence* of a later finding depend on how many earlier
    /// ones there were.
    pub(super) fn record(&mut self, finding: StaticLinkRefusal) {
        self.findings.push(finding);
    }

    /// Record a finding if this check produced one, and report whether the
    /// value is available to a dependent check.
    ///
    /// The `Result` is consumed rather than returned, so a caller cannot record
    /// a failure and then use the value that failed.
    pub(super) fn keep<T>(&mut self, outcome: Result<T, StaticLinkRefusal>) -> Option<T> {
        match outcome {
            Ok(value) => Some(value),
            Err(finding) => {
                self.record(finding);
                None
            }
        }
    }

    /// The accumulated findings, or `None` when every check passed.
    pub(super) fn into_diagnostics(self) -> Option<StaticDiagnostics> {
        let mut findings = self.findings.into_iter();
        let first = findings.next()?;
        Some(StaticDiagnostics {
            first,
            rest: findings.collect(),
        })
    }
}

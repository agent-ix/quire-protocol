// SPDX-License-Identifier: AGPL-3.0-or-later
//! Located construction-fatal diagnostics. Owner chunk: **C-13**.
//!
//! `Task-002` requires the linker to "accumulate independent located
//! diagnostics while every construction-fatal refusal returns no linked
//! representation". Those are two obligations, and they pull in opposite
//! directions: reporting more than one finding means continuing past the first,
//! and continuing past the first is how a half-built representation escapes.
//!
//! [`Diagnostics`] is how both hold at once. The linker keeps going and records
//! what it finds, and the accumulated findings are the **error** half of its
//! result. There is no constructor that produces a [`LinkedProtocol`] alongside
//! them, so "some diagnostics, and here is the graph anyway" is not a value this
//! module can express.
//!
//! [`LinkedProtocol`]: super::LinkedProtocol
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
//! So a [`Diagnostics`] carrying *n* findings says n independent checks failed.
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
//! Not every finding has one, and the ones that do not are not given a
//! substitute. A declaration index outside the package's declaration table, a
//! declaration whose body is not a protocol, and two records mapping onto one
//! identity are refusals about *table positions* and whole declarations, which
//! occupy no source region. Manufacturing a zero span, or borrowing the
//! enclosing declaration's, would make a reader's caret point at source text
//! that is not what was refused. [`LinkRefusal::locus`] returns `None` for those
//! three, and [`Diagnostics::located`] yields only the findings that genuinely
//! carry a region.
//!
//! The two refusals about a binding-requirement *index* — out of range, and
//! present but of the wrong kind — do carry one. The index is an authored field
//! of the referring record, the referrer's own `locus` is in hand at the site
//! that checks it, and `FR-002-AC-3` requires an unresolved or wrong-kind
//! reference refused at its source location. An earlier revision of this module
//! gave the out-of-range case no region and defended the absence in prose; the
//! prose was wrong, because the region was never missing.
//!
//! # How many findings one refusal may carry
//!
//! [`CEILING`] of them. The accumulation is bounded because an admitted artifact
//! is large: A admits up to 100 000 entries, and a package with a mistake in
//! every record would otherwise make this crate allocate one finding per record
//! — turning a constant-size refusal into one that scales with the input. Past
//! the ceiling the linker stops recording and sets [`Diagnostics::truncated`],
//! which the rendering names. It still refuses, and it still refuses for the
//! reasons already recorded: a ceiling drops findings, never the refusal.
//!
//! This is a bound on *this crate's own output*, not a caller-granted budget.
//! `FR-016`'s dimensions are the ones a caller grants for a request, and the
//! linker takes no budget today; when it does, this ceiling becomes one more
//! thing that budget selects.

use std::fmt;

use quire_spec_language::protocol_artifact::wire::Locus;

use super::LinkRefusal;

/// The most findings one refusal carries. See the module documentation.
pub const CEILING: usize = 1_000;

/// One or more construction-fatal findings from a single link attempt.
///
/// Non-empty by construction, and by the *shape* of the struct rather than by a
/// convention this module remembers to keep: the first finding is a field, not
/// an element. [`Diagnostics::first`] is therefore a field access, with nothing
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
pub struct Diagnostics {
    first: LinkRefusal,
    rest: Vec<LinkRefusal>,
    truncated: bool,
}

impl Diagnostics {
    /// A `Diagnostics` carrying exactly one finding.
    ///
    /// Crate-internal: the linker establishes findings, and a consumer minting
    /// one would be manufacturing a refusal the linker never made.
    pub(crate) fn of(finding: LinkRefusal) -> Self {
        Self {
            first: finding,
            rest: Vec::new(),
            truncated: false,
        }
    }

    /// The first finding established.
    #[must_use]
    pub const fn first(&self) -> &LinkRefusal {
        &self.first
    }

    /// Every finding after the first, in the order the linker established them.
    #[must_use]
    pub fn rest(&self) -> &[LinkRefusal] {
        &self.rest
    }

    /// Every finding, in the order the linker established them.
    pub fn iter(&self) -> impl Iterator<Item = &LinkRefusal> {
        std::iter::once(&self.first).chain(self.rest.iter())
    }

    /// How many independent checks failed, up to [`CEILING`].
    ///
    /// Named `count` rather than `len`: a `len` on a type that cannot be empty
    /// invites an `is_empty` that always answers `false`, which is a question
    /// with one answer dressed as a query.
    #[must_use]
    pub fn count(&self) -> usize {
        1 + self.rest.len()
    }

    /// Whether the linker stopped recording at [`CEILING`] findings.
    ///
    /// `true` means there were more; it never means the refusal is in doubt.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Each finding that carries an original source region, with that region.
    ///
    /// Findings about table positions carry none and are absent here rather
    /// than present with an invented span; see the module documentation.
    pub fn located(&self) -> impl Iterator<Item = (&LinkRefusal, &Locus)> {
        self.iter()
            .filter_map(|finding| finding.locus().map(|locus| (finding, locus)))
    }
}

impl fmt::Display for Diagnostics {
    /// The findings, one per line, each preceded by its source region when it
    /// has one, and a final line when the list was truncated.
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
        if self.truncated {
            write!(f, "\n... and more, beyond the {CEILING} reported here")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

/// One source region, as the producer recorded it.
///
/// A's own coordinates, rendered and not reinterpreted. `source` is A's index
/// into the package's source table, not a path, and the span is A's own offset
/// pair: this crate does not resolve either against source bytes, and does not
/// claim a line and column A did not supply.
///
/// This is presentation. `FR-037` requires no refusal fact derived from text, so
/// a caller asking *where* asks [`Diagnostics::located`] and reads the [`Locus`]
/// itself; this function exists so a human reading one line sees the region.
fn region(locus: &Locus) -> String {
    format!(
        "source {}, {}..{}",
        locus.source, locus.span.start, locus.span.end
    )
}

/// The linker's private accumulator.
///
/// Separate from [`Diagnostics`] so that "nothing found yet" and "at least one
/// finding" are different types. The linker holds the first and can only hand
/// out the second, which is what keeps an empty `Diagnostics` unconstructable
/// from outside as well as inside this crate.
#[derive(Debug, Default)]
pub(super) struct Accumulator {
    findings: Vec<LinkRefusal>,
    truncated: bool,
}

impl Accumulator {
    /// Record one independent finding and keep going.
    ///
    /// Past [`CEILING`] the finding is dropped and the truncation recorded. The
    /// traversal continues either way: stopping early would make the *presence*
    /// of a later finding depend on how many earlier ones there were, and the
    /// dependent-skip decisions below read resolved values, not the finding
    /// count.
    pub(super) fn record(&mut self, finding: LinkRefusal) {
        if self.findings.len() >= CEILING {
            self.truncated = true;
            return;
        }
        self.findings.push(finding);
    }

    /// Record a finding if this check produced one, and report whether the
    /// value is available to a dependent check.
    ///
    /// The `Result` is consumed rather than returned, so a caller cannot record
    /// a failure and then use the value that failed.
    pub(super) fn keep<T>(&mut self, outcome: Result<T, LinkRefusal>) -> Option<T> {
        match outcome {
            Ok(value) => Some(value),
            Err(finding) => {
                self.record(finding);
                None
            }
        }
    }

    /// The accumulated findings, or `None` when every check passed.
    pub(super) fn into_diagnostics(self) -> Option<Diagnostics> {
        let mut findings = self.findings.into_iter();
        let first = findings.next()?;
        Some(Diagnostics {
            first,
            rest: findings.collect(),
            truncated: self.truncated,
        })
    }
}

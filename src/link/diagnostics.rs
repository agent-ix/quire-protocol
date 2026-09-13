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
//! - a channel's identity is derived from premises that resolved, so a channel
//!   with a refused premise contributes no identity and cannot be found
//!   colliding with one,
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
//! declaration whose body is not a protocol, a binding-requirement index
//! outside its declaration's binding table, and two records mapping onto one
//! identity are refusals about *table positions* and whole declarations, which
//! occupy no source region. Manufacturing a zero span, or borrowing the
//! enclosing declaration's, would make a reader's caret point at source text
//! that is not what was refused. [`LinkRefusal::locus`] returns `None` for those
//! four, and [`Diagnostics::located`] yields only the findings that genuinely
//! carry a region.

use std::fmt;

use quire_spec_language::protocol_artifact::wire::Locus;

use super::LinkRefusal;

/// One or more construction-fatal findings from a single link attempt.
///
/// Non-empty by construction: the only ways to build one are
/// [`Diagnostics::of`], which takes a finding, and the crate-internal
/// accumulator, which yields `None` rather than an empty value. A caller
/// holding a `Diagnostics` therefore never has to handle "an error with nothing
/// in it", and [`Diagnostics::first`] returns a reference rather than an
/// `Option`.
///
/// Findings are in the order the linker established them, which is its own
/// traversal order over A's tables: controls, then roles, then channels. The
/// order is declared, not sorted — sorting would need a ranking over refusal
/// kinds, and no requirement gives one.
///
/// Equal findings are kept rather than collapsed. Two channels that map onto
/// one identity are two records at fault, and reporting one would name a
/// collision without saying it happened twice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostics {
    findings: Vec<LinkRefusal>,
}

impl Diagnostics {
    /// A `Diagnostics` carrying exactly one finding.
    #[must_use]
    pub fn of(finding: LinkRefusal) -> Self {
        Self {
            findings: vec![finding],
        }
    }

    /// The first finding established.
    ///
    /// A reference, not an `Option`: the type is non-empty by construction.
    #[must_use]
    pub fn first(&self) -> &LinkRefusal {
        self.findings
            .first()
            .expect("a Diagnostics is non-empty by construction")
    }

    /// Every finding, in the order the linker established them.
    #[must_use]
    pub fn all(&self) -> &[LinkRefusal] {
        &self.findings
    }

    /// How many independent checks failed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.findings.len()
    }

    /// Always `false`. Present because clippy requires it beside [`Self::len`],
    /// and it is the honest answer for a type that cannot be empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Each finding that carries an original source region, with that region.
    ///
    /// Findings about table positions carry none and are absent here rather
    /// than present with an invented span; see the module documentation.
    pub fn located(&self) -> impl Iterator<Item = (&LinkRefusal, &Locus)> {
        self.findings
            .iter()
            .filter_map(|finding| finding.locus().map(|locus| (finding, locus)))
    }
}

impl fmt::Display for Diagnostics {
    /// The findings, one per line, each preceded by its source region when it
    /// has one.
    ///
    /// A single finding renders as that finding's own text with no count and no
    /// list, so the common case reads as a sentence rather than as a report of
    /// length one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, finding) in self.findings.iter().enumerate() {
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

/// One source region, as the producer recorded it.
///
/// A's own coordinates, rendered and not reinterpreted. `source` is A's index
/// into the package's source table, not a path, and the span is A's own offset
/// pair: this crate does not resolve either against source bytes, and does not
/// claim a line and column A did not supply.
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
}

impl Accumulator {
    /// Record one independent finding and keep going.
    pub(super) fn record(&mut self, finding: LinkRefusal) {
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
        (!self.findings.is_empty()).then_some(Diagnostics {
            findings: self.findings,
        })
    }
}

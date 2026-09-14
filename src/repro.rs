// SPDX-License-Identifier: AGPL-3.0-or-later
//! Seed, toolchain and corpus reproduction record. Owner chunk: **C-31**.
//!
//! A local run is reproducible only if the exact deterministic seed, the exact
//! toolchain identity and the exact corpus identity that produced it are all
//! recorded. This module holds those three as typed fields on one record, and
//! makes the absence of any of them a refusal.
//!
//! # The failure this module exists to prevent
//!
//! A defaulted identity converts an unreproducible run into one that *claims*
//! to be reproducible. A default seed, an inferred toolchain or an assumed
//! corpus produces a record that looks complete, compares equal to other
//! equally defaulted records, and is worthless for replay. The failure is
//! silent: nothing fails, and the lie is only discovered when someone tries to
//! reproduce the run and cannot.
//!
//! So this module refuses. There is no fallback value anywhere in it:
//!
//! * No type here implements [`Default`]. Not [`Seed`], not
//!   [`ToolchainIdentity`], not [`CorpusIdentity`], not
//!   [`ReproductionRecord`], not [`ReproductionInputs`]. The `compile_fail`
//!   proofs below are the evidence, and they run under `cargo test`.
//! * No `unwrap_or`, `unwrap_or_default`, `unwrap_or_else` or `if let ... else`
//!   fallback reads an identity. Each offered input is matched, and the `None`
//!   arm returns a refusal rather than a value.
//! * No type here implements [`From<String>`], `From<&str>` or
//!   [`std::str::FromStr`], so no display name can be promoted into an
//!   identity.
//!
//! # Identity is bytes; display text is not identity
//!
//! [`ToolchainIdentity`] and [`CorpusIdentity`] each retain the producer's
//! exact identity bytes unchanged — no trim, no case fold, no Unicode
//! normalisation, no UTF-8 requirement — alongside the producer's
//! human-readable display text. `PartialEq`, `Eq`, `PartialOrd`, `Ord` and
//! `Hash` are hand-written over the bytes alone, so rendering never enters an
//! equality test or a map lookup. This follows the pattern established by the
//! protocol identity newtypes in [`crate::ids`] (chunk C-07); these two types
//! are *not* protocol identities and deliberately do not extend that closed set
//! — a toolchain and a corpus identify the **run**, not anything inside the
//! admitted protocol.
//!
//! # Absent and empty are different, and each has its own code
//!
//! An identity that is present but empty is **not** treated as absent. The two
//! are distinct conditions with distinct codes
//! ([`ReproductionRefusal::ToolchainIdentityAbsent`] versus
//! [`ReproductionRefusal::ToolchainIdentityEmpty`]), because collapsing them
//! would make "the caller never told us" indistinguishable from "the caller
//! told us nothing identifies this toolchain". Both refuse; they refuse
//! separately. An empty identity is never admitted into a record.
//!
//! A seed has no empty form. Zero is a legitimate seed value and is admitted,
//! which is exactly why absence may not be represented by zero: a defaulted
//! `0` would be indistinguishable from a deliberately chosen `0`.
//!
//! # Public API boundary decision: these enums are CLOSED, not `#[non_exhaustive]`
//!
//! [`ReproductionRefusal`] and [`ReproductionInputKind`] are deliberately not
//! `#[non_exhaustive]`. This is a decision, not a derive default, and it weighs
//! the same precedent chunk C-07 set for [`crate::ids`]:
//!
//! * A `#[non_exhaustive]` enum forces every downstream `match` to carry a
//!   wildcard arm, and a wildcard arm silently absorbs a variant added later.
//!   A refusal that is absorbed by someone else's `_ =>` arm is not
//!   independently assertable, which is the entire property this chunk is built
//!   to deliver.
//! * The set is not a growth surface. It enumerates the reproduction inputs
//!   named by NFR-001 and issue #10 box 5 — seed, toolchain, corpus — in their
//!   absent and empty forms. Its authority is the specification, so a fourth
//!   reproduction input is a spec amendment, and breaking every consumer's
//!   `match` at compile time is the intended cost of one.
//! * Both enums expose [`ReproductionRefusal::ALL`] and
//!   [`ReproductionInputKind::ALL`], so a consumer that wants to iterate the
//!   set does so over a named constant rather than over a wildcard.
//!
//! Every `match` in this module over either enum is exhaustive and carries no
//! `_` arm, so adding a variant here fails to compile until each arm is
//! considered.
//!
//! # Boundary
//!
//! This module owns the reproduction record only. It does not canonicalise,
//! digest or serialise it: the single RFC 8785 result-domain implementation is
//! chunk C-11's (`src/digest/jcs.rs`), and embedding this record as provenance
//! on the canonical protocol result is chunk C-26's (`src/result/mod.rs`).
//! Nothing here reaches a parser, a grammar, a compiler output schema or a
//! canonicalizer.
//!
//! # Proof that no reproduction input has a default
//!
//! The positive control first — each type is constructible when its value is
//! actually supplied, so every `compile_fail` case below fails for the missing
//! `Default` and for nothing else.
//!
//! ```
//! use quire_protocol::repro::{CorpusIdentity, ReproductionInputs, Seed, ToolchainIdentity};
//!
//! let record = ReproductionInputs::nothing_offered()
//!     .with_seed(Seed::new(0))
//!     .with_toolchain(ToolchainIdentity::new(b"rustc-1.98.0", "rustc 1.98.0"))
//!     .with_corpus(CorpusIdentity::new(b"corpus-a1b2", "corpus a1b2"))
//!     .record()
//!     .expect("every reproduction input was supplied");
//! assert_eq!(record.seed().value(), 0);
//! ```
//!
//! ```compile_fail
//! let _ = quire_protocol::repro::Seed::default();
//! ```
//!
//! ```compile_fail
//! let _ = quire_protocol::repro::ToolchainIdentity::default();
//! ```
//!
//! ```compile_fail
//! let _ = quire_protocol::repro::CorpusIdentity::default();
//! ```
//!
//! ```compile_fail
//! let _ = quire_protocol::repro::ReproductionRecord::default();
//! ```
//!
//! ```compile_fail
//! let _ = quire_protocol::repro::ReproductionInputs::default();
//! ```
//!
//! # Proof that display text can never become an identity
//!
//! ```compile_fail
//! let _: quire_protocol::repro::ToolchainIdentity = "rustc 1.98.0".parse().unwrap();
//! ```
//!
//! ```compile_fail
//! let _ = quire_protocol::repro::CorpusIdentity::from("corpus a1b2".to_string());
//! ```
//!
//! # Proof that the two run identities are not interchangeable
//!
//! ```compile_fail
//! use quire_protocol::repro::{CorpusIdentity, ReproductionInputs};
//! ReproductionInputs::nothing_offered().with_toolchain(CorpusIdentity::new(b"c", "c"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::repro::{ReproductionInputs, ToolchainIdentity};
//! ReproductionInputs::nothing_offered().with_corpus(ToolchainIdentity::new(b"t", "t"));
//! ```

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind};

pub(crate) const INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("reproduction/identity/retain"),
    "repro::ToolchainIdentity::new/repro::CorpusIdentity::new",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("reproduction/identity/allocate"),
    "repro::ToolchainIdentity::new/repro::CorpusIdentity::new",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// A run identity: the producer's exact bytes, plus display text that takes no
/// part in equality, ordering or hashing.
macro_rules! run_identity {
    ($name:ident, $what:literal) => {
        #[doc = concat!("The identity of the ", $what, " a run was produced with.")]
        ///
        /// Equality, ordering and hashing are over the producer's exact bytes
        /// alone. Two identities with equal bytes and different display text
        /// are equal; two with equal display text and different bytes are not.
        /// The type has no [`Default`], no `From<String>` and no `FromStr`.
        #[derive(Clone, Debug)]
        pub struct $name {
            bytes: Vec<u8>,
            display: String,
        }

        impl $name {
            #[doc = concat!("Retain the producer's exact ", $what, " identity bytes and display text.")]
            ///
            /// The bytes are copied unchanged: no trim, no case fold, no
            /// Unicode normalisation and no UTF-8 requirement.
            #[must_use]
            pub fn new(bytes: &[u8], display: &str) -> Self {
                Self {
                    bytes: bytes.to_vec(),
                    display: display.to_owned(),
                }
            }

            #[doc = concat!("Retain the ", $what, " identity under the shared caller budget.")]
            ///
            /// Exact identity and display bytes, plus the single abstract
            /// identity-storage allocation, are charged transactionally before
            /// either owned buffer is constructed.
            pub fn new_bounded(
                bytes: &[u8],
                display: &str,
                resources: &mut ResourceAccountant<'_>,
            ) -> OperationOutcome<Self, core::convert::Infallible> {
                let retained = match bytes.len().checked_add(display.len()) {
                    Some(retained) => retained,
                    None => {
                        return OperationOutcome::from_charge(ChargeError::CounterOverflow {
                            site: INPUT_SOURCE.id(),
                            consumed: bytes.len(),
                            requested: display.len(),
                        })
                    }
                };
                let charges = [
                    (INPUT_SOURCE.id(), retained),
                    (ALLOCATION_SOURCE.id(), 1),
                ];
                if let Err(error) = resources.charge_all(&charges) {
                    return OperationOutcome::from_charge(error);
                }
                OperationOutcome::Completed(Self::new(bytes, display))
            }

            /// The producer's exact identity bytes.
            #[must_use]
            pub fn bytes(&self) -> &[u8] {
                &self.bytes
            }

            /// The producer's display text. Never part of identity.
            #[must_use]
            pub fn display_text(&self) -> &str {
                &self.display
            }

            /// Whether the retained identity bytes are empty.
            ///
            /// An empty identity is present-but-vacuous, which is a different
            /// condition from absent and refuses under a different code.
            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.bytes.is_empty()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.bytes == other.bytes
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.bytes.cmp(&other.bytes)
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.bytes.hash(state);
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.display)
            }
        }
    };
}

run_identity!(ToolchainIdentity, "toolchain");
run_identity!(CorpusIdentity, "corpus");

/// A deterministic seed, recorded exactly as the caller chose it.
///
/// Zero is a legitimate seed and is admitted. Absence is never represented by
/// zero — an unsupplied seed is [`ReproductionRefusal::SeedAbsent`]. The type
/// has no [`Default`], so no code path can produce a seed nobody chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seed(u64);

impl Seed {
    /// Record the caller's chosen deterministic seed.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The chosen seed value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The kind of reproduction input a refusal concerns.
///
/// Closed, not `#[non_exhaustive]` — see the module documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReproductionInputKind {
    /// The deterministic seed.
    Seed,
    /// The toolchain identity.
    Toolchain,
    /// The corpus identity.
    Corpus,
}

impl ReproductionInputKind {
    /// Every reproduction input this record requires.
    pub const ALL: [Self; 3] = [Self::Seed, Self::Toolchain, Self::Corpus];

    /// The stable label of this input kind.
    ///
    /// The match is exhaustive and carries no `_` arm.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::Toolchain => "toolchain",
            Self::Corpus => "corpus",
        }
    }
}

impl fmt::Display for ReproductionInputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Why a reproduction record could not be built.
///
/// Every variant is independently assertable: no two conditions in this module
/// share a variant or a code, and no variant is reachable from more than one
/// missing or vacuous input. Closed, not `#[non_exhaustive]` — see the module
/// documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReproductionRefusal {
    /// No deterministic seed was supplied. Not defaulted to zero.
    SeedAbsent,
    /// No toolchain identity was supplied. Not inferred from the build.
    ToolchainIdentityAbsent,
    /// A toolchain identity was supplied but retains no bytes.
    ToolchainIdentityEmpty,
    /// No corpus identity was supplied. Not assumed from the working tree.
    CorpusIdentityAbsent,
    /// A corpus identity was supplied but retains no bytes.
    CorpusIdentityEmpty,
}

impl ReproductionRefusal {
    /// Every refusal this module can produce.
    ///
    /// Rust cannot compile-check that this array covers the enum without a
    /// derive macro, and this crate's dependency set is closed, so a new
    /// variant must be added here by hand. The three exhaustive matches below
    /// (`code`, `input_kind`, `is_absence`) each fail to compile until the new
    /// variant is considered, and `tc_008_every_refusal_has_its_own_code_and_kind`
    /// asserts this array's length, so the omission is loud rather than silent.
    pub const ALL: [Self; 5] = [
        Self::SeedAbsent,
        Self::ToolchainIdentityAbsent,
        Self::ToolchainIdentityEmpty,
        Self::CorpusIdentityAbsent,
        Self::CorpusIdentityEmpty,
    ];

    /// The stable code of this refusal.
    ///
    /// These codes are B-owned and stable. They are expected to be carried by
    /// chunk C-01's `ProtocolRefusal` envelope once it lands on `main`; this
    /// module deliberately does not build an envelope of its own.
    ///
    /// The match is exhaustive and carries no `_` arm, so a sixth refusal fails
    /// to compile here until it is given its own code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::SeedAbsent => "reproduction-seed-absent",
            Self::ToolchainIdentityAbsent => "reproduction-toolchain-identity-absent",
            Self::ToolchainIdentityEmpty => "reproduction-toolchain-identity-empty",
            Self::CorpusIdentityAbsent => "reproduction-corpus-identity-absent",
            Self::CorpusIdentityEmpty => "reproduction-corpus-identity-empty",
        }
    }

    /// The reproduction input this refusal concerns.
    ///
    /// The match is exhaustive and carries no `_` arm.
    #[must_use]
    pub const fn input_kind(self) -> ReproductionInputKind {
        match self {
            Self::SeedAbsent => ReproductionInputKind::Seed,
            Self::ToolchainIdentityAbsent | Self::ToolchainIdentityEmpty => {
                ReproductionInputKind::Toolchain
            }
            Self::CorpusIdentityAbsent | Self::CorpusIdentityEmpty => ReproductionInputKind::Corpus,
        }
    }

    /// Whether this refusal reports an input that was never supplied, as
    /// opposed to one supplied without content.
    ///
    /// The match is exhaustive and carries no `_` arm.
    #[must_use]
    pub const fn is_absence(self) -> bool {
        match self {
            Self::SeedAbsent | Self::ToolchainIdentityAbsent | Self::CorpusIdentityAbsent => true,
            Self::ToolchainIdentityEmpty | Self::CorpusIdentityEmpty => false,
        }
    }
}

impl fmt::Display for ReproductionRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for ReproductionRefusal {}

/// The reproduction inputs a caller has offered so far.
///
/// Each field is explicitly present or explicitly absent. There is no
/// [`Default`]: a caller that has offered nothing says so by name with
/// [`ReproductionInputs::nothing_offered`], and nothing is filled in for them.
#[derive(Clone, Debug)]
pub struct ReproductionInputs {
    seed: Option<Seed>,
    toolchain: Option<ToolchainIdentity>,
    corpus: Option<CorpusIdentity>,
}

impl ReproductionInputs {
    /// Begin with every reproduction input explicitly unsupplied.
    #[must_use]
    pub const fn nothing_offered() -> Self {
        Self {
            seed: None,
            toolchain: None,
            corpus: None,
        }
    }

    /// Supply the deterministic seed.
    #[must_use]
    pub fn with_seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Supply the toolchain identity.
    #[must_use]
    pub fn with_toolchain(mut self, toolchain: ToolchainIdentity) -> Self {
        self.toolchain = Some(toolchain);
        self
    }

    /// Supply the corpus identity.
    #[must_use]
    pub fn with_corpus(mut self, corpus: CorpusIdentity) -> Self {
        self.corpus = Some(corpus);
        self
    }

    /// Build the reproduction record, or refuse.
    ///
    /// Every read below matches on the offered value and returns a refusal from
    /// the `None` arm. No read substitutes a value, and no read is written as
    /// `unwrap_or`, `unwrap_or_default` or `unwrap_or_else`.
    ///
    /// # Errors
    ///
    /// Returns the single [`ReproductionRefusal`] naming the first reproduction
    /// input that is absent or empty, in the order seed, toolchain, corpus.
    pub fn record(self) -> Result<ReproductionRecord, ReproductionRefusal> {
        let seed = match self.seed {
            Some(seed) => seed,
            None => return Err(ReproductionRefusal::SeedAbsent),
        };
        let toolchain = match self.toolchain {
            Some(toolchain) if toolchain.is_empty() => {
                return Err(ReproductionRefusal::ToolchainIdentityEmpty)
            }
            Some(toolchain) => toolchain,
            None => return Err(ReproductionRefusal::ToolchainIdentityAbsent),
        };
        let corpus = match self.corpus {
            Some(corpus) if corpus.is_empty() => {
                return Err(ReproductionRefusal::CorpusIdentityEmpty)
            }
            Some(corpus) => corpus,
            None => return Err(ReproductionRefusal::CorpusIdentityAbsent),
        };
        Ok(ReproductionRecord {
            seed,
            toolchain,
            corpus,
        })
    }
}

/// Every identity needed to reproduce one local run, as typed fields.
///
/// A value of this type exists only if all three reproduction inputs were
/// supplied and non-empty, so holding one is itself the evidence that the run
/// it describes is reproducible. Equality and hashing run over the seed value
/// and the two identities' exact bytes; display text of either identity takes
/// no part, so no rendering difference can make two distinct runs compare
/// equal, and no rendering difference can split one run into two.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReproductionRecord {
    seed: Seed,
    toolchain: ToolchainIdentity,
    corpus: CorpusIdentity,
}

impl ReproductionRecord {
    /// The deterministic seed this run used.
    #[must_use]
    pub const fn seed(&self) -> Seed {
        self.seed
    }

    /// The toolchain identity this run was produced with.
    #[must_use]
    pub const fn toolchain(&self) -> &ToolchainIdentity {
        &self.toolchain
    }

    /// The corpus identity this run consumed.
    #[must_use]
    pub const fn corpus(&self) -> &CorpusIdentity {
        &self.corpus
    }
}

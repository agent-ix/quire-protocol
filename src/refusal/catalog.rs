// SPDX-License-Identifier: AGPL-3.0-or-later
//! The B-owned refusal-code catalog and the selected diagnostic catalog.
//! Owner chunk: **C-01** (diagnostic-catalog *selection semantics*: **C-02**).
//!
//! Two different things are called a "catalog" by FR-008 and both live here.
//!
//! 1. **B's own code catalog** — [`REGISTRY`]: every stable refusal-code
//!    spelling this crate is allowed to emit, with its domain, its retry
//!    decision and the chunk that owns it. This is the code *namespace*.
//! 2. **The selected diagnostic catalog** — [`CatalogSelection`]: the exact
//!    identity, revision and digest of the versioned catalog a *source* refusal
//!    was minted against, retained uninterpreted. C-02 owns validating a code
//!    against it and owns `unsupported-source-contract`; C-01 owns only the
//!    fact that the selection is carried, typed, and never inferred.
//!
//! # Why the crate-wide code namespace is a catalog and not one enum
//!
//! See the module docs of [`super`] for the full decision. In short: a single
//! crate-wide code enum would have to be edited by every chunk that mints a
//! code (COLLISION-5), and a `#[non_exhaustive]` one would force a wildcard arm
//! on every downstream `match` — the exact silent-absorption failure the
//! independently-assertable-refusal directive forbids. So each chunk keeps its
//! **own closed enum** in its **own file**, and every variant of every such
//! enum resolves to a [`RefusalCode`] registered here. The property a
//! crate-wide enum would have given for free — no two chunks minting the same
//! spelling — is recovered by [`audit`], which is run as a test.
//!
//! # Reserved roots
//!
//! A module claims the first dotted segment of its codes as a **reserved
//! root**. No two modules may claim the same root, and inside `src/` a
//! code-shaped string literal under a reserved root that is not registered here
//! is a defect — see [`unregistered_literals`], which the TC-011 audit runs
//! over every tracked `src/` file. That is what makes the audit repo-wide
//! rather than self-referential: a later chunk cannot mint a code by writing a
//! bare string literal, it has to register it.

use core::fmt;

use super::{RefusalDomain, RetryDecision};

// ---------------------------------------------------------------------------
// Catalog entries
// ---------------------------------------------------------------------------

/// One registered refusal code: its stable spelling and everything a caller may
/// decide from it without reading display text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CodeEntry {
    /// The stable spelling. Lowercase, dot-separated, hyphen-joined words.
    ///
    /// This spelling is API: changing it is a breaking change, and
    /// [`audit`] rejects a spelling that does not conform.
    pub code: &'static str,
    /// The FR-008 domain of the operation that emits this code.
    pub domain: RefusalDomain,
    /// What a caller may do about it, decided from the code alone.
    pub retry: RetryDecision,
    /// What the code means. Documentation only — never parsed, never compared.
    pub summary: &'static str,
}

/// Every code minted by one owning module, under one reserved root.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CodeModule {
    /// The reserved first dotted segment of every code in [`Self::entries`].
    pub root: &'static str,
    /// The chunk that owns these codes, e.g. `"C-01"`.
    pub owner: &'static str,
    /// The Rust path of the module that emits them.
    pub module_path: &'static str,
    /// The codes themselves, in declared order.
    pub entries: &'static [CodeEntry],
}

impl CodeModule {
    /// The entry with this exact spelling, if this module mints it.
    #[must_use]
    pub fn entry(&self, code: &str) -> Option<&'static CodeEntry> {
        self.entries.iter().find(|entry| entry.code == code)
    }
}

// ---------------------------------------------------------------------------
// C-01's own codes
// ---------------------------------------------------------------------------

/// A source response offered a refusal code with no entry in B's catalog.
///
/// This is B's *local* lookup failure and nothing more. Classifying an unknown
/// source revision or a future code as `unsupported-source-contract`, retaining
/// the original catalog selection and the original uninterpreted value, is
/// FR-008-AC-3 and belongs to chunk C-02.
pub const UNREGISTERED_CODE: CodeEntry = CodeEntry {
    code: "refusal.code.unregistered",
    domain: RefusalDomain::Representation,
    retry: RetryDecision::Never,
    summary: "the offered refusal code has no entry in the consumer's catalog",
};

/// The codes chunk C-01 itself mints, under the reserved root `refusal`.
pub const ENVELOPE_CODES: CodeModule = CodeModule {
    root: "refusal",
    owner: "C-01",
    module_path: "quire_protocol::refusal",
    entries: &[UNREGISTERED_CODE],
};

/// Every code this crate may emit.
///
/// A chunk that mints codes appends **one** `CodeModule` line here and defines
/// its entries in its own file. That single line is the only part of this
/// module a later chunk may touch (COLLISION-5).
pub const REGISTRY: &[CodeModule] = &[ENVELOPE_CODES];

/// The registered code with this exact spelling, if any.
#[must_use]
pub fn lookup(code: &str) -> Option<RefusalCode> {
    lookup_in(REGISTRY, code)
}

/// The registered code with this exact spelling in an explicit registry.
///
/// Exposed so the audit can be exercised against a deliberately defective
/// registry without mutating the real one.
#[must_use]
pub fn lookup_in(registry: &'static [CodeModule], code: &str) -> Option<RefusalCode> {
    registry
        .iter()
        .find_map(|module| module.entry(code))
        .map(RefusalCode::of)
}

// ---------------------------------------------------------------------------
// RefusalCode
// ---------------------------------------------------------------------------

/// A refusal code that is known to be registered.
///
/// The only way to obtain one is from a catalog entry, so an envelope can never
/// carry a spelling nothing owns. Equality is over the stable spelling, never
/// over the summary text.
#[derive(Clone, Copy, Debug)]
pub struct RefusalCode {
    entry: &'static CodeEntry,
}

impl RefusalCode {
    /// Wrap a catalog entry.
    #[must_use]
    pub const fn of(entry: &'static CodeEntry) -> Self {
        Self { entry }
    }

    /// The stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.entry.code
    }

    /// The FR-008 domain of the operation that emits this code.
    #[must_use]
    pub const fn domain(self) -> RefusalDomain {
        self.entry.domain
    }

    /// What a caller may do about it, decided from typed data alone.
    #[must_use]
    pub const fn retry(self) -> RetryDecision {
        self.entry.retry
    }

    /// Documentation for a human reader. Never compared, never parsed.
    #[must_use]
    pub const fn summary(self) -> &'static str {
        self.entry.summary
    }
}

impl PartialEq for RefusalCode {
    fn eq(&self, other: &Self) -> bool {
        self.entry.code == other.entry.code
    }
}

impl Eq for RefusalCode {}

impl core::hash::Hash for RefusalCode {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.entry.code.hash(state);
    }
}

impl fmt::Display for RefusalCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.entry.code)
    }
}

// ---------------------------------------------------------------------------
// The selected diagnostic catalog
// ---------------------------------------------------------------------------

/// Exact bytes retained from a producer or source, unchanged.
///
/// There is deliberately no `From<String>`, no `FromStr` and no normalisation:
/// a value is what the source said it was, byte for byte.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RetainedBytes(Vec<u8>);

impl RetainedBytes {
    /// Retain these exact bytes.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    /// The retained bytes, unchanged.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Why the selected diagnostic catalog has no digest.
///
/// Explicit, like [`super::Unavailability`]: an absent digest is a fact, and no
/// amount of identity or revision text may be turned into one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CatalogDigestUnavailable {
    /// The catalog is B's own local one and has not been published with a
    /// digest yet.
    NotPublished,
    /// The source declared a catalog identity and revision and no digest.
    NotDeclaredBySource,
}

/// The digest of the selected diagnostic catalog: present, or explicitly not.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CatalogDigest {
    /// The digest exactly as the source declared it, uninterpreted.
    Declared(RetainedBytes),
    /// No digest exists, for this stated reason.
    Unavailable(CatalogDigestUnavailable),
}

/// The exact versioned diagnostic catalog a refusal code was minted against.
///
/// C-01 carries it. C-02 validates a code against it, refuses a changed
/// identity, revision or digest (FR-008-AC-6), and owns
/// `unsupported-source-contract` (FR-008-AC-3).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CatalogSelection {
    identity: RetainedBytes,
    revision: RetainedBytes,
    digest: CatalogDigest,
}

impl CatalogSelection {
    /// A selection taken from a source response, retained uninterpreted.
    #[must_use]
    pub fn declared(
        identity: RetainedBytes,
        revision: RetainedBytes,
        digest: CatalogDigest,
    ) -> Self {
        Self {
            identity,
            revision,
            digest,
        }
    }

    /// B's own local catalog: the codes in [`REGISTRY`].
    ///
    /// Its digest is [`CatalogDigestUnavailable::NotPublished`] because this
    /// catalog has no published artifact yet. Declaring a digest B invented
    /// would be exactly the fabrication FR-008 forbids everywhere else.
    #[must_use]
    pub fn local() -> Self {
        Self {
            identity: RetainedBytes::new(LOCAL_CATALOG_IDENTITY.as_bytes()),
            revision: RetainedBytes::new(LOCAL_CATALOG_REVISION.as_bytes()),
            digest: CatalogDigest::Unavailable(CatalogDigestUnavailable::NotPublished),
        }
    }

    /// The catalog identity, exactly as selected.
    #[must_use]
    pub fn identity(&self) -> &RetainedBytes {
        &self.identity
    }

    /// The catalog revision, exactly as selected.
    #[must_use]
    pub fn revision(&self) -> &RetainedBytes {
        &self.revision
    }

    /// The catalog digest, present or explicitly unavailable.
    #[must_use]
    pub fn digest(&self) -> &CatalogDigest {
        &self.digest
    }
}

/// Identity of B's own local diagnostic catalog.
pub const LOCAL_CATALOG_IDENTITY: &str = "agent-ix/quire-protocol/refusal-catalog";
/// Revision of B's own local diagnostic catalog.
pub const LOCAL_CATALOG_REVISION: &str = "1";

// ---------------------------------------------------------------------------
// The repo-wide code-uniqueness audit
// ---------------------------------------------------------------------------

/// Why a code spelling is not a legal code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MalformedCode {
    /// Fewer than two dot-separated segments.
    FewerThanTwoSegments,
    /// A segment was empty.
    EmptySegment,
    /// A segment did not begin with an ASCII lowercase letter.
    SegmentDoesNotStartWithLetter,
    /// A character outside `a-z`, `0-9` and `-` appeared in a segment.
    IllegalCharacter,
    /// A segment began or ended with a hyphen.
    HyphenAtSegmentEdge,
}

/// A defect found by [`audit`]. Every variant is independently assertable.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CatalogDefect {
    /// Two modules mint the same code spelling. This is the defect a
    /// crate-wide closed enum would have made impossible.
    DuplicateCode {
        /// The shared spelling.
        code: &'static str,
        /// The chunk that registered it first.
        first_owner: &'static str,
        /// The chunk that registered it again.
        second_owner: &'static str,
    },
    /// Two modules claim the same reserved root.
    DuplicateRoot {
        /// The shared root.
        root: &'static str,
        /// The chunk that claimed it first.
        first_owner: &'static str,
        /// The chunk that claimed it again.
        second_owner: &'static str,
    },
    /// A code does not begin with its module's reserved root.
    RootMismatch {
        /// The offending spelling.
        code: &'static str,
        /// The root its module reserved.
        root: &'static str,
        /// The chunk that owns it.
        owner: &'static str,
    },
    /// A code is not a legal spelling.
    Malformed {
        /// The offending spelling.
        code: &'static str,
        /// The chunk that owns it.
        owner: &'static str,
        /// Why it is illegal.
        reason: MalformedCode,
    },
    /// A module's declared domain disagrees with an entry it holds.
    ///
    /// Raised when a module mints codes in more than one domain: a code's
    /// domain is part of its meaning, so a module that spans domains cannot be
    /// reasoned about as one owner.
    MixedDomains {
        /// The reserved root.
        root: &'static str,
        /// The chunk that owns it.
        owner: &'static str,
        /// The first domain seen.
        first: RefusalDomain,
        /// The domain that disagreed.
        second: RefusalDomain,
    },
}

/// Check a registry for code-namespace defects.
///
/// Returns every defect found, in discovery order, rather than the first: a
/// namespace with two independent collisions has two problems, and reporting
/// one of them would hide the other.
#[must_use]
pub fn audit(registry: &[CodeModule]) -> Vec<CatalogDefect> {
    let mut defects = Vec::new();
    let mut seen_roots: Vec<(&'static str, &'static str)> = Vec::new();
    let mut seen_codes: Vec<(&'static str, &'static str)> = Vec::new();

    for module in registry {
        if let Some(&(root, first_owner)) = seen_roots.iter().find(|(root, _)| *root == module.root)
        {
            defects.push(CatalogDefect::DuplicateRoot {
                root,
                first_owner,
                second_owner: module.owner,
            });
        } else {
            seen_roots.push((module.root, module.owner));
        }

        let mut module_domain: Option<RefusalDomain> = None;
        for entry in module.entries {
            if let Some(reason) = malformation(entry.code) {
                defects.push(CatalogDefect::Malformed {
                    code: entry.code,
                    owner: module.owner,
                    reason,
                });
            }
            if root_of(entry.code) != Some(module.root) {
                defects.push(CatalogDefect::RootMismatch {
                    code: entry.code,
                    root: module.root,
                    owner: module.owner,
                });
            }
            if let Some(&(code, first_owner)) =
                seen_codes.iter().find(|(code, _)| *code == entry.code)
            {
                defects.push(CatalogDefect::DuplicateCode {
                    code,
                    first_owner,
                    second_owner: module.owner,
                });
            } else {
                seen_codes.push((entry.code, module.owner));
            }
            match module_domain {
                None => module_domain = Some(entry.domain),
                Some(first) if first != entry.domain => {
                    defects.push(CatalogDefect::MixedDomains {
                        root: module.root,
                        owner: module.owner,
                        first,
                        second: entry.domain,
                    });
                }
                Some(_) => {}
            }
        }
    }

    defects
}

/// The reserved first segment of a code, if the code has one.
#[must_use]
pub fn root_of(code: &str) -> Option<&str> {
    let root = code.split('.').next()?;
    (!root.is_empty()).then_some(root)
}

/// Why this spelling is illegal, or `None` if it is legal.
#[must_use]
pub fn malformation(code: &str) -> Option<MalformedCode> {
    let segments: Vec<&str> = code.split('.').collect();
    if segments.len() < 2 {
        return Some(MalformedCode::FewerThanTwoSegments);
    }
    for segment in segments {
        if segment.is_empty() {
            return Some(MalformedCode::EmptySegment);
        }
        if !segment.starts_with(|character: char| character.is_ascii_lowercase()) {
            return Some(MalformedCode::SegmentDoesNotStartWithLetter);
        }
        if segment.ends_with('-') {
            return Some(MalformedCode::HyphenAtSegmentEdge);
        }
        if !segment.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        }) {
            return Some(MalformedCode::IllegalCharacter);
        }
    }
    None
}

/// Whether a string literal is shaped like a refusal code under a root that a
/// registry has reserved.
#[must_use]
pub fn is_reserved_code_shape(registry: &[CodeModule], literal: &str) -> bool {
    if malformation(literal).is_some() {
        return false;
    }
    let Some(root) = root_of(literal) else {
        return false;
    };
    registry.iter().any(|module| module.root == root)
}

/// A code-shaped literal under a reserved root that no module registered.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnregisteredLiteral {
    /// The literal exactly as it appeared in the source.
    pub literal: String,
}

/// Every code-shaped literal in one Rust source text that is under a reserved
/// root and is not registered.
///
/// This is the repo-wide half of the audit: registering a code is the only way
/// to emit one, so a chunk that writes a bare `"admission.something"` in
/// `src/` without registering it fails TC-011 on the commit that adds it.
///
/// The scan is deliberately conservative. It looks only at double-quoted
/// literals, and it only complains about roots a module has already reserved,
/// so an unrelated dotted string such as a file name is never a finding.
#[must_use]
pub fn unregistered_literals(
    registry: &'static [CodeModule],
    source: &str,
) -> Vec<UnregisteredLiteral> {
    let mut found = Vec::new();
    for literal in string_literals(source) {
        if !is_reserved_code_shape(registry, &literal) {
            continue;
        }
        if lookup_in(registry, &literal).is_some() {
            continue;
        }
        let unregistered = UnregisteredLiteral { literal };
        if !found.contains(&unregistered) {
            found.push(unregistered);
        }
    }
    found
}

/// Every double-quoted literal in a Rust source text, escapes honoured.
///
/// Raw strings are read as ordinary quoted text, which can only ever produce a
/// superset of the literals in the file; a superset is safe here because an
/// extra candidate is discarded unless it is shaped like a reserved code.
fn string_literals(source: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut characters = source.chars();
    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }
        let mut literal = String::new();
        loop {
            match characters.next() {
                None => break,
                Some('\\') => {
                    // Skip the escaped character: it can never be a dot, a
                    // lowercase letter or a hyphen in a way that matters here.
                    let _escaped = characters.next();
                    literal.push('\\');
                }
                Some('"') => {
                    literals.push(literal);
                    break;
                }
                Some(other) => literal.push(other),
            }
        }
    }
    literals
}

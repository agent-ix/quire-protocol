// SPDX-License-Identifier: AGPL-3.0-or-later
//! B's own typed refusal envelope. Owner chunk: **C-01**.
//!
//! FR-008 requires that when a public operation cannot return its declared
//! success value it returns **one structure**, not a message: a domain, a
//! stable source-owned code, the selected diagnostic catalog, the affected
//! identity, a present-or-explicitly-absent location, the originating stage and
//! its prerequisites, an ordered cause chain, and an explicit partial-output
//! state — every one of them readable with no string parsing.
//!
//! [`ProtocolRefusal`] is that structure. A refusal is not extendable by A: A
//! owns `Invalid`/`Unsupported` and B cannot add to them, so this envelope is
//! B's own, and it is the type `#1`'s "return the FR-008 typed admission
//! refusal" box actually needs.
//!
//! # The decision: closed enums, an open catalog
//!
//! Three separate namespaces are in play here and they get three different
//! answers.
//!
//! **[`RefusalDomain`] is CLOSED.** FR-008 names exactly five domains. A sixth
//! is a specification amendment, not a library change, and every consumer that
//! switches on the domain should stop compiling until its author has classified
//! the new one. `#[non_exhaustive]` here would force a wildcard arm on every
//! downstream `match`, and a wildcard arm silently absorbs a variant added
//! later — the exact failure the independently-assertable-refusal directive
//! forbids. Same reasoning, same answer, as chunk C-07 reached for
//! `IdentityKind`. [`OriginatingStage`], [`RetryDecision`], [`PartialOutput`]
//! and [`Unavailability`] are closed for the same reason: each is a small set
//! fixed by FR-008's own text.
//!
//! **Each chunk's own code enum is CLOSED, in that chunk's own file.** A
//! consumer that wants to discriminate admission-selection axes matches on the
//! admission chunk's enum, exhaustively, with no wildcard. That is where
//! independent assertability actually lives.
//!
//! **The crate-wide code namespace is NOT an enum at all.** It is
//! [`catalog::REGISTRY`], and [`catalog::RefusalCode`] is a handle into it.
//! Two reasons, and the second is the one that decides it:
//!
//! 1. A crate-wide closed enum would have to be edited by every chunk that
//!    mints a code. That is COLLISION-5 in the chunk plan — one file, thirty
//!    authors, a conflict a day — and the plan's own mitigation is that each
//!    chunk owns its codes in its own file.
//! 2. A crate-wide `#[non_exhaustive]` enum would be the worst of both: it
//!    still centralises the edit, and it *additionally* forces the wildcard arm
//!    on every downstream consumer. C-07's reasoning does not merely transfer
//!    to the code namespace, it applies with more force, because downstream
//!    consumers outside this crate are exactly who reads codes.
//!
//! What a crate-wide enum would have bought for free is the guarantee that no
//! two chunks mint the same spelling. That guarantee is not given up; it is
//! moved from the compiler to [`catalog::audit`] and the repo-wide literal scan
//! in [`catalog::unregistered_literals`], both run as TC-011 tests. A chunk
//! that mints a colliding code, an unregistered code, or a code under another
//! chunk's reserved root fails the test suite on the commit that adds it.
//!
//! # A's enums are `#[non_exhaustive]`, so one wildcard arm is compulsory
//!
//! Nothing above changes the fact that `quire_spec_language`'s `Error`,
//! `Invalid` and `Unsupported` are `#[non_exhaustive]`: the compiler *requires*
//! a final arm on every `match` over them, and a literally catch-all-free
//! mapping is impossible from B. The mapping chunks (C-05, and PR #36 before
//! it) name every published variant explicitly and make the compulsory arm emit
//! a loud, separately assertable "unclassified source" refusal, with a test
//! that fails if any published A variant reaches it. This module supports that
//! shape rather than pretending the constraint away: the envelope carries
//! [`Cause::retained`] so an unclassified source cause keeps A's original
//! rendering uninterpreted instead of being folded into a neighbouring code.
//!
//! # Boundary
//!
//! This module owns the envelope, the code namespace and their invariants. It
//! does not classify A's errors (C-05), does not validate a code against a
//! selected source catalog (C-02), and does not define the typed cause taxonomy
//! (C-03). It also owns no grammar, parser, compiler output schema or
//! canonicalizer: a refusal is consumed after admission, never produced by
//! reading native source.

/// Diagnostic-catalog selection and the B-owned refusal-code catalog. Owner
/// chunks: **C-01** (the code catalog and the carried selection) and **C-02**
/// (validating a source code against the selection, and
/// `unsupported-source-contract`).
pub mod catalog;

/// Typed cause taxonomy, declared cause ordering, partial-output policy. Owner chunk: **C-03**.
///
/// Will own the five structurally distinct cause pairs and the rule that a
/// fatal refusal emits no success value. The ordered chain those causes hang
/// from is [`CauseChain`], landed here by C-01.
pub mod cause;

use core::fmt;
use core::hash::{Hash, Hasher};

use catalog::{CatalogSelection, RefusalCode, RetainedBytes};

// ---------------------------------------------------------------------------
// Domain
// ---------------------------------------------------------------------------

/// The FR-008 refusal domain: which kind of operation refused.
///
/// **Closed on purpose.** See the module docs. There is deliberately no
/// `FromStr` and no `From<&str>`: a domain is never recovered from text.
///
/// ```compile_fail
/// use std::str::FromStr;
/// use quire_protocol::refusal::RefusalDomain;
/// // There is no `FromStr` for a refusal domain, so this does not compile.
/// let _domain = RefusalDomain::from_str("admission");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RefusalDomain {
    /// Admitting offered bytes under an independently selected inventory.
    Admission,
    /// Building or reading a representation of an already admitted artifact.
    Representation,
    /// Assessing compensation, conformance or a dispatched claim.
    Assessment,
    /// Producing or reading a canonical protocol result.
    Result,
    /// Projecting a result onto a downstream target.
    Projection,
}

impl RefusalDomain {
    /// Every domain, in declared order.
    pub const ALL: [Self; 5] = [
        Self::Admission,
        Self::Representation,
        Self::Assessment,
        Self::Result,
        Self::Projection,
    ];

    /// The stable spelling of this domain.
    ///
    /// Rendering only in one direction: there is no inverse, by design.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admission => "admission",
            Self::Representation => "representation",
            Self::Assessment => "assessment",
            Self::Result => "result",
            Self::Projection => "projection",
        }
    }
}

impl fmt::Display for RefusalDomain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// What a caller may do about a refusal, decided from the code alone.
///
/// Closed: FR-008-AC-2 requires the retry decision to be stable under every
/// rendering, so it is a property of the registered code and of nothing else.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RetryDecision {
    /// Retrying the identical request cannot change the outcome.
    Never,
    /// Retrying can only succeed after the caller changes its own independent
    /// selections; the offer is not at fault.
    AfterSelectionChange,
    /// Retrying can succeed once a resource the operation exhausted is
    /// available again.
    AfterResourceRelease,
}

// ---------------------------------------------------------------------------
// Stage and prerequisites
// ---------------------------------------------------------------------------

/// The originating stage FR-008 requires a refusal to retain.
///
/// Closed: the five stages are named by FR-008's own behaviour clause. A
/// wrapper cannot erase or replace the stage because [`ProtocolRefusal`] has no
/// setter for it — the only way to obtain a refusal is to build one with a
/// stage, and the field is private.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum OriginatingStage {
    /// Recognising the source at all.
    SourceRecognition,
    /// Resolving identities and linking references.
    IdentityLinking,
    /// Checking types and profile conformance.
    TypeProfile,
    /// Binding inputs to declared parameters.
    InputBinding,
    /// Selecting a capability or backend.
    CapabilitySelection,
    /// Executing under a resource bound.
    ResourceExecution,
}

impl OriginatingStage {
    /// Every stage, in declared order.
    pub const ALL: [Self; 6] = [
        Self::SourceRecognition,
        Self::IdentityLinking,
        Self::TypeProfile,
        Self::InputBinding,
        Self::CapabilitySelection,
        Self::ResourceExecution,
    ];

    /// The stable spelling of this stage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceRecognition => "source-recognition",
            Self::IdentityLinking => "identity-linking",
            Self::TypeProfile => "type-profile",
            Self::InputBinding => "input-binding",
            Self::CapabilitySelection => "capability-selection",
            Self::ResourceExecution => "resource-execution",
        }
    }
}

/// Whether a prerequisite of the originating stage was established.
///
/// `Unavailable` is explicit and is never a synonym for `NotEstablished`: "the
/// source did not say" and "the source said no" are different facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PrerequisiteState {
    /// The prerequisite stage completed.
    Established,
    /// The prerequisite stage did not complete.
    NotEstablished,
    /// The source reported nothing about this prerequisite.
    Unavailable,
}

/// One prerequisite of the originating stage, with its state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Prerequisite {
    stage: OriginatingStage,
    state: PrerequisiteState,
}

impl Prerequisite {
    /// Record a prerequisite stage and its state.
    #[must_use]
    pub const fn new(stage: OriginatingStage, state: PrerequisiteState) -> Self {
        Self { stage, state }
    }

    /// The prerequisite stage.
    #[must_use]
    pub const fn stage(self) -> OriginatingStage {
        self.stage
    }

    /// Whether it was established.
    #[must_use]
    pub const fn state(self) -> PrerequisiteState {
        self.state
    }
}

// ---------------------------------------------------------------------------
// Affected identity
// ---------------------------------------------------------------------------

/// The identity a refusal is about.
///
/// The producer's exact bytes are retained: no trim, no case-fold, no
/// normalisation, and no construction from a display name. When chunk C-07's
/// identity newtypes are consumed by a later chunk they hand their exact bytes
/// to [`Self::retained`]; that is additive and needs no change here.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum AffectedIdentity {
    /// The exact identity bytes the refusal is about.
    Retained(RetainedBytes),
    /// The refusal is about the caller's own inputs or about the offer as a
    /// whole, so no single producer identity applies. Explicit, so that "no
    /// identity applies" is never confused with "the identity was lost".
    NotApplicable,
}

impl AffectedIdentity {
    /// Retain these exact identity bytes.
    #[must_use]
    pub fn retained(bytes: &[u8]) -> Self {
        Self::Retained(RetainedBytes::new(bytes))
    }
}

// ---------------------------------------------------------------------------
// Location
// ---------------------------------------------------------------------------

/// Why a refusal has no source location.
///
/// Closed, and each variant is a distinct fact. FR-008-AC-7 turns on this
/// type: an unavailable location must stay unavailable, and there is no
/// constructor anywhere in this module that turns a display name, a path or a
/// message into a [`SourceLocation`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Unavailability {
    /// The source produced no location for this refusal.
    NotReportedBySource,
    /// The originating stage has no source position to report.
    NotApplicableToStage,
    /// A zero or exhausted diagnostic-detail budget withheld it. The refusal
    /// still exists; only its detail was withheld.
    WithheldByDetailBudget,
}

/// An exact source position, as the source reported it.
///
/// Built only from typed positional data. There is no `FromStr`, no
/// constructor from a path, and no search over display names.
///
/// ```compile_fail
/// use quire_protocol::refusal::SourceLocation;
/// // A display name is not a location: there is no such constructor.
/// let _location = SourceLocation::from("src/lib.rs");
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SourceLocation {
    unit: RetainedBytes,
    start: u64,
    end: u64,
}

/// A reported span whose end precedes its start.
///
/// Not a [`ProtocolRefusal`]: this is refused at construction, before any
/// envelope exists to carry it, and the caller that holds the transposed offsets
/// is the one that has to decide what to do about them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct InvertedSpan {
    /// The reported first byte offset.
    pub start: u64,
    /// The reported offset one past the span, which precedes `start`.
    pub end: u64,
}

impl fmt::Display for InvertedSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "reported span ends at {} before it starts at {}",
            self.end, self.start
        )
    }
}

impl SourceLocation {
    /// A position the source itself reported, in the source's own unit
    /// identity and byte offsets.
    ///
    /// `end` is the offset one past the span, so `start == end` is the empty
    /// span at `start` and is admitted. `end < start` is refused.
    ///
    /// The check is here rather than left to the consumer because every
    /// consumer of a location does arithmetic on it. A transposed pair — a
    /// mapping chunk writing `reported(unit, span.end, span.start)` — would
    /// otherwise build a well-formed envelope carrying a span that underflows
    /// on `end - start` and panics on `&text[start..end]`, with the defect
    /// surfacing in the renderer rather than at the call site that made it.
    pub fn reported(unit: &[u8], start: u64, end: u64) -> Result<Self, InvertedSpan> {
        if end < start {
            return Err(InvertedSpan { start, end });
        }
        Ok(Self {
            unit: RetainedBytes::new(unit),
            start,
            end,
        })
    }

    /// The source unit identity, exactly as reported.
    #[must_use]
    pub fn unit(&self) -> &RetainedBytes {
        &self.unit
    }

    /// The first byte offset of the span.
    #[must_use]
    pub const fn start(&self) -> u64 {
        self.start
    }

    /// The byte offset one past the span.
    #[must_use]
    pub const fn end(&self) -> u64 {
        self.end
    }
}

/// A source location, or an explicit statement that there is none.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Location {
    /// The source reported this exact position.
    Available(SourceLocation),
    /// There is no position, for this stated reason.
    Unavailable(Unavailability),
}

impl Location {
    /// Whether a position is present.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available(_))
    }
}

// ---------------------------------------------------------------------------
// Causes
// ---------------------------------------------------------------------------

/// The detail retained with a cause.
///
/// Explicit three-way, not `Option`: FR-008-AC-8 requires that an empty
/// diagnostic-detail budget can neither erase a cause's existence nor
/// establish success, so "the budget withheld it" must be distinguishable from
/// "the source offered none".
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CauseDetail {
    /// The source's own detail for this cause, retained uninterpreted.
    Retained(RetainedBytes),
    /// The source offered no detail.
    NoneOffered,
    /// A zero or exhausted diagnostic-detail budget withheld the detail. The
    /// cause itself is still present in the chain.
    WithheldByDetailBudget,
}

/// One cause of a refusal: a registered code and whatever detail survived.
///
/// The taxonomy that decides *which* code a given cause gets — the five
/// structurally distinct cause pairs of FR-008-AC-5 — is chunk C-03's, in
/// [`cause`]. Each of its variants resolves to its own registered code, so a
/// cause is independently assertable without reading this struct's detail.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Cause {
    code: RefusalCode,
    detail: CauseDetail,
}

impl Cause {
    /// A cause identified by its registered code, with no detail offered.
    #[must_use]
    pub const fn new(code: RefusalCode) -> Self {
        Self {
            code,
            detail: CauseDetail::NoneOffered,
        }
    }

    /// A cause with the source's own detail retained uninterpreted.
    #[must_use]
    pub fn with_detail(code: RefusalCode, detail: CauseDetail) -> Self {
        Self { code, detail }
    }

    /// The registered code of this cause.
    #[must_use]
    pub const fn code(&self) -> RefusalCode {
        self.code
    }

    /// The retained detail, or the explicit reason there is none.
    #[must_use]
    pub const fn detail(&self) -> &CauseDetail {
        &self.detail
    }
}

/// The causes of a refusal, in the order the source declared them.
///
/// The order is **declared, never sorted**: FR-008 requires independent causes
/// to be retained in their declared semantic order, so this type exposes no
/// sort, no dedup and no reorder. Appending is the only way to build one.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct CauseChain {
    causes: Vec<Cause>,
}

impl CauseChain {
    /// An empty chain.
    #[must_use]
    pub const fn new() -> Self {
        Self { causes: Vec::new() }
    }

    /// Append a cause at the end of the declared order.
    pub fn push(&mut self, cause: Cause) {
        self.causes.push(cause);
    }

    /// The causes, in declared order.
    #[must_use]
    pub fn declared_order(&self) -> &[Cause] {
        &self.causes
    }

    /// How many causes are retained.
    #[must_use]
    pub fn len(&self) -> usize {
        self.causes.len()
    }

    /// Whether the chain is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.causes.is_empty()
    }

    /// The declared position of the first cause with this code.
    #[must_use]
    pub fn position_of(&self, code: RefusalCode) -> Option<usize> {
        self.causes.iter().position(|cause| cause.code() == code)
    }
}

// ---------------------------------------------------------------------------
// Partial output
// ---------------------------------------------------------------------------

/// Whether anything at all survived the refusal.
///
/// Closed, and **separate from any success value**: the state is declared on
/// the envelope and the surviving values, if any, live in
/// [`RefusedWithIndependent`]. No `Option<T>` doubles as both, so a caller can
/// never read "there is a value" out of "the operation partly succeeded" or
/// the reverse.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PartialOutput {
    /// Nothing survived. The only legal state for a fatal admission,
    /// representation, result or projection refusal (FR-008-AC-4).
    NoneEmitted,
    /// Independent assessment results that FR-005 expressly authorises to
    /// survive another claim's failure were retained.
    IndependentResultsRetained,
}

/// A refusal that retained independent results FR-005 authorises to survive.
///
/// The two fields are separate by construction, and the only constructor
/// requires at least one retained value, so the declared state and the
/// retained values can never disagree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusedWithIndependent<T> {
    refusal: ProtocolRefusal,
    retained: Vec<T>,
}

impl<T> RefusedWithIndependent<T> {
    /// The refusal itself.
    #[must_use]
    pub const fn refusal(&self) -> &ProtocolRefusal {
        &self.refusal
    }

    /// The independent results that survived, in declared order.
    #[must_use]
    pub fn independent_results(&self) -> &[T] {
        &self.retained
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Display material attached to a refusal.
///
/// Explicitly **not** part of refusal identity: [`ProtocolRefusal`] implements
/// `PartialEq` and `Hash` by hand over its typed fields only, so changing the
/// message, the locale, a display name or a path leaves equality, hashing, the
/// code, the stage and the retry decision bit-identical (FR-008-AC-2).
///
/// None of these fields can become a [`SourceLocation`]. That is the whole
/// point of keeping them here rather than near the location.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Rendering {
    /// A human-readable message.
    pub message: String,
    /// The locale the message was rendered in.
    pub locale: String,
    /// A display name for the affected thing. Never an identity.
    pub display_name: String,
    /// A path shown to a human. Never a location.
    pub path: String,
}

// ---------------------------------------------------------------------------
// The envelope
// ---------------------------------------------------------------------------

/// One typed refusal.
///
/// Every FR-008-AC-1 field is present and readable without parsing display
/// text. The domain is not stored separately from the code: it is the domain
/// the code is registered under, so the two can never disagree.
#[derive(Clone, Debug)]
pub struct ProtocolRefusal {
    code: RefusalCode,
    catalog: CatalogSelection,
    stage: OriginatingStage,
    prerequisites: Vec<Prerequisite>,
    identity: AffectedIdentity,
    location: Location,
    causes: CauseChain,
    partial_output: PartialOutput,
    rendering: Rendering,
}

impl ProtocolRefusal {
    /// Start building a refusal.
    ///
    /// The three arguments are the three things a refusal cannot exist
    /// without: what refused, against which catalog, and at which stage.
    #[must_use]
    pub fn builder(
        code: RefusalCode,
        catalog: CatalogSelection,
        stage: OriginatingStage,
    ) -> RefusalBuilder {
        RefusalBuilder {
            code,
            catalog,
            stage,
            prerequisites: Vec::new(),
            identity: AffectedIdentity::NotApplicable,
            location: Location::Unavailable(Unavailability::NotReportedBySource),
            causes: CauseChain::new(),
            rendering: Rendering::default(),
        }
    }

    /// The FR-008 domain, taken from the code's catalog registration.
    #[must_use]
    pub const fn domain(&self) -> RefusalDomain {
        self.code.domain()
    }

    /// The stable registered code.
    #[must_use]
    pub const fn code(&self) -> RefusalCode {
        self.code
    }

    /// The exact selected diagnostic catalog, retained uninterpreted.
    #[must_use]
    pub const fn catalog(&self) -> &CatalogSelection {
        &self.catalog
    }

    /// The originating stage. There is no setter: a wrapper cannot replace it.
    #[must_use]
    pub const fn stage(&self) -> OriginatingStage {
        self.stage
    }

    /// The prerequisites of the originating stage, in declared order.
    #[must_use]
    pub fn prerequisites(&self) -> &[Prerequisite] {
        &self.prerequisites
    }

    /// The identity the refusal is about.
    #[must_use]
    pub const fn identity(&self) -> &AffectedIdentity {
        &self.identity
    }

    /// The location, present or explicitly absent.
    #[must_use]
    pub const fn location(&self) -> &Location {
        &self.location
    }

    /// The causes, in declared order.
    #[must_use]
    pub const fn causes(&self) -> &CauseChain {
        &self.causes
    }

    /// What, if anything, survived the refusal.
    #[must_use]
    pub const fn partial_output(&self) -> PartialOutput {
        self.partial_output
    }

    /// What a caller may do about it, decided from the registered code alone
    /// and never from display text.
    #[must_use]
    pub const fn retry(&self) -> RetryDecision {
        self.code.retry()
    }

    /// The display material. Never part of identity.
    #[must_use]
    pub const fn rendering(&self) -> &Rendering {
        &self.rendering
    }

    /// The same refusal rendered differently.
    ///
    /// Exists so that FR-008-AC-2 is testable in the direction that matters: a
    /// caller can re-render a refusal and must observe no change in identity.
    #[must_use]
    pub fn rendered_as(&self, rendering: Rendering) -> Self {
        let mut rendered = self.clone();
        rendered.rendering = rendering;
        rendered
    }

    /// The typed fields that constitute refusal identity.
    ///
    /// Named once, and used by both `PartialEq` and `Hash`, so the two can
    /// never drift apart.
    fn identity_fields(
        &self,
    ) -> (
        RefusalCode,
        &CatalogSelection,
        OriginatingStage,
        &[Prerequisite],
        &AffectedIdentity,
        &Location,
        &CauseChain,
        PartialOutput,
    ) {
        (
            self.code,
            &self.catalog,
            self.stage,
            &self.prerequisites,
            &self.identity,
            &self.location,
            &self.causes,
            self.partial_output,
        )
    }
}

impl PartialEq for ProtocolRefusal {
    fn eq(&self, other: &Self) -> bool {
        self.identity_fields() == other.identity_fields()
    }
}

impl Eq for ProtocolRefusal {}

impl Hash for ProtocolRefusal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity_fields().hash(state);
    }
}

impl fmt::Display for ProtocolRefusal {
    /// Renders the refusal for a human. Nothing in this crate reads it back.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} refusal {} at stage {}",
            self.domain(),
            self.code,
            self.stage.as_str()
        )?;
        if !self.rendering.message.is_empty() {
            write!(formatter, ": {}", self.rendering.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ProtocolRefusal {}

/// Builder for [`ProtocolRefusal`].
///
/// It has no `partial_output` setter. The partial-output state is decided by
/// which of the two terminal methods is called, so a refusal that emitted
/// nothing can never claim to have retained results and a refusal that retained
/// results can never claim to have emitted nothing.
#[derive(Clone, Debug)]
pub struct RefusalBuilder {
    code: RefusalCode,
    catalog: CatalogSelection,
    stage: OriginatingStage,
    prerequisites: Vec<Prerequisite>,
    identity: AffectedIdentity,
    location: Location,
    causes: CauseChain,
    rendering: Rendering,
}

impl RefusalBuilder {
    /// Record a prerequisite of the originating stage.
    #[must_use]
    pub fn prerequisite(mut self, prerequisite: Prerequisite) -> Self {
        self.prerequisites.push(prerequisite);
        self
    }

    /// Record the identity the refusal is about.
    #[must_use]
    pub fn identity(mut self, identity: AffectedIdentity) -> Self {
        self.identity = identity;
        self
    }

    /// Record a location the source itself reported.
    ///
    /// Takes a [`SourceLocation`], never text: there is no path through this
    /// builder by which a display name, a path or a message becomes a span.
    #[must_use]
    pub fn located_at(mut self, location: SourceLocation) -> Self {
        self.location = Location::Available(location);
        self
    }

    /// Record why there is no location.
    #[must_use]
    pub fn unavailable_location(mut self, reason: Unavailability) -> Self {
        self.location = Location::Unavailable(reason);
        self
    }

    /// Append a cause at the end of the declared order.
    #[must_use]
    pub fn cause(mut self, cause: Cause) -> Self {
        self.causes.push(cause);
        self
    }

    /// Attach display material. It never becomes identity or location.
    #[must_use]
    pub fn rendering(mut self, rendering: Rendering) -> Self {
        self.rendering = rendering;
        self
    }

    /// Finish a refusal that emitted no success value at all.
    #[must_use]
    pub fn emitting_nothing(self) -> ProtocolRefusal {
        self.finish(PartialOutput::NoneEmitted)
    }

    /// Finish a refusal that retained independent results FR-005 authorises to
    /// survive another claim's failure.
    ///
    /// At least one result is required by the signature, so
    /// [`PartialOutput::IndependentResultsRetained`] can never be declared over
    /// an empty set.
    #[must_use]
    pub fn retaining_independent<T>(self, first: T, rest: Vec<T>) -> RefusedWithIndependent<T> {
        let refusal = self.finish(PartialOutput::IndependentResultsRetained);
        let mut retained = Vec::with_capacity(1 + rest.len());
        retained.push(first);
        retained.extend(rest);
        RefusedWithIndependent { refusal, retained }
    }

    fn finish(self, partial_output: PartialOutput) -> ProtocolRefusal {
        ProtocolRefusal {
            code: self.code,
            catalog: self.catalog,
            stage: self.stage,
            prerequisites: self.prerequisites,
            identity: self.identity,
            location: self.location,
            causes: self.causes,
            partial_output,
            rendering: self.rendering,
        }
    }
}

---
id: NFR-008
title: "Carry every refusal entirely in its return value"
type: NFR
quality_attribute: maintainability
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-014
    type: constrains
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: depends_on
---
# NFR-008: Carry every refusal entirely in its return value

## Statement

The library SHALL determine which refusal occurred from the returned value alone, using only public typed accessors on that value.
The library SHALL expose no refusal information through a log, a stream, a file, a global, a thread-local, a panic, or any channel other than the returned value.
The library SHALL derive no refusal determination from a rendered or formatted representation of the returned value.
The library SHALL represent an absent refusal component as an explicit absence distinguishable from a present component whose content is empty.

## Scope

Applies to every value a public operation returns instead of its declared
success value: the typed refusal envelope of
[FR-008](../functional/FR-008-return-typed-refusals.md), the
resource-incomplete outcome of
[NFR-002](./NFR-002-enforce-resource-bounds.md), and the completed-empty result
the three-way distinction in
[FR-014](../functional/FR-014-typed-outcome-identity.md) keeps separate from
both.

Applies to the **return channel**: what the value carries, and how a caller
reaches it. It does not apply to logging, telemetry, tracing, metrics, or
any recording of a refusal after it is returned. `AD-001` Views states that no
boundary reads the filesystem, spawns a process, accesses a network, retains
evidence, or invokes a business action, and
[NFR-003](./NFR-003-preserve-rust-assurance-boundary.md) holds that boundary.
This requirement is the reason that containment is survivable: a library that
may not write anywhere must put everything a caller needs inside the value it
hands back.

Does not apply to:

- Which fields the envelope has. FR-008 owns the five-member domain, the stable
  source code, the catalog selection, the originating stage and prerequisites,
  the affected identity, the location, the ordered causes, and the partial-output
  state, and this requirement adds no member and renames none.
- Which outcomes are the same outcome. FR-014 owns the typed outcome identity,
  its key, and the injectivity of the axis-to-identity mapping. This requirement
  does not restate injectivity; it requires that whatever injectivity FR-014
  obliges be **decidable from the returned value with no input beyond it**.
- The envelope's interior vocabulary. Upstream `FR-270` through `FR-274` own it,
  and each open gap is cited in FR-008 and FR-014 rather than closed locally. No
  member list, key, order or spelling is supplied here.

### Boundary against the neighbouring non-functional requirements

| Neighbour | What it owns | What this requirement owns instead |
| --- | --- | --- |
| [NFR-001](./NFR-001-reproduce-semantic-results.md) | That equivalent presentations reproduce byte-identical canonical JSON and identical ordered outcomes, and the property/metamorphic method that shows it. | That the refusal value is itself within that reproducibility, and carries no allocation address, wall-clock reading, locale-dependent rendering or map-iteration artefact. NFR-001 supplies the method; this names the value. |
| [NFR-002](./NFR-002-enforce-resource-bounds.md) | The budget dimensions, the deadline discipline, and the resource-incomplete outcome including the observed elapsed time it reports. | That the resource-incomplete outcome is reachable through public accessors like any other non-success value. The elapsed-time figure is excluded from the reproducibility comparison of AC-6, exactly as [NFR-005](./NFR-005-retain-composed-acceptance-evidence.md) excludes it, because it is not reproducible. |
| [NFR-003](./NFR-003-preserve-rust-assurance-boundary.md) | That no first-party path is non-Rust, that `unsafe` is forbidden absent an authorized disposition, and that no local runner or evidence store exists. | That no refusal path uses an out-of-band channel or a panic, which NFR-003 does not forbid: `unsafe_code = "forbid"` says nothing about `eprintln!` or `unwrap`. |
| [NFR-005](./NFR-005-retain-composed-acceptance-evidence.md) | What a scenario must retain and hand to Quoin, and the per-axis distinctness check that fails an integration when two axes collide. | That the scenario could observe those things from the value in the first place. NFR-005 retains what this requirement makes observable. |

## Rationale

Refusal-identity injectivity is load-bearing and it is checked by reading
refusals. FR-014 records the defect it exists to prevent, quoting this
repository's own corpus: "A two-valued admits/refuses oracle is satisfied by one
shared error code across every negative row, which is the defect SR-027 recorded
on version 1." That check is only as good as what a test can see. Three ways to make
it unfalsifiable while leaving FR-008 and FR-014 formally satisfied:

1. **Put the discrimination in the rendering.** Two axes carry one typed code and
   differ only in their message text. FR-008-AC-2 forbids deriving identity from
   text, so the honest consequence is that the two axes are indistinguishable —
   but a harness that asserts on `Debug` output will not notice, and will report
   a passing distinctness check.
2. **Put it out of band.** The axis is written to stderr or a log and the
   returned value is one generic variant. Nothing in FR-008 forbids this, and
   nothing in NFR-003 does either, because it is still Rust and still safe.
3. **Panic instead of returning.** A refusal that aborts carries no value at all.
   The negative scenario "fails" and a two-valued oracle records a refusal, with
   no identity to collide.

Each is a way for a typed refusal corpus to collapse back into a two-valued
oracle. The
observability obligation is what closes them, and it is stateable without any of
upstream's open gaps closing, for the same reason FR-014 gives about
injectivity: it is a property of the return channel, not of the interior's
spelling.

The fourth statement is the one most easily lost. FR-008-AC-7 requires
unavailable source context to stay explicit and never become an invented
location. Its observable form is a discrimination: an absent location and a
location present with empty content must not compare equal, or "explicitly
unavailable" is indistinguishable from "available and blank" to the only party
that can check it.

## Measurement and Evaluation

Every metric is denominated on the refusal axes the corpus **owes**, not on
those executed, so an unexecutable axis scores zero rather than a vacuous
hundred per cent.

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Owed refusal axes whose determination is a total function of public typed accessors on the returned value | Every owed axis | 100% | Test (TC-011) |
| Axis determinations that change when every rendering impl on the returned value is replaced by a constant | None | 0 determinations | Test (TC-011) |
| Refusal-path writes to a stream, file, global, thread-local or logging facade | None | 0 sites; 0 in `src/` today | Inspection |
| Refusal paths that panic, abort, unwrap or expect instead of returning a value | None | 0 sites; 0 non-doc sites in `src/` today | Inspection |
| Envelope components an acceptance criterion relies on that are reachable only through a rendered form or a non-public path | None | 0 components | Inspection |
| Absent components that compare equal to a present-but-empty component of the same kind | None | 0 of the location, stage, prerequisite and cause-chain kinds | Test (TC-011) |
| Non-success values that differ between two runs on identical inputs, excluding the observed elapsed time | None | 0 differing values | Test (TC-008) |

## Verification

The three static metrics are read from the repository's own tracked Rust sources
and are offline. They exclude documentation comments **and `#[cfg(test)]`
modules**, which is why the two "0 today" figures are stated as non-doc counts.

An earlier revision of this paragraph said the only `unwrap`, `expect` and
`panic` occurrences in `src/` were three doc comments in `src/ids.rs` and
`src/repro.rs`. That was already false when written and is more so now:
`src/link/mod.rs` alone carries 38, every one of them inside its
`#[cfg(test)] mod tests`. A test that asserts `expect("the fixture links")` is
not a refusal path, and counting it makes the metric fail against a denominator
that was never what AC-4 is about. The exclusion is stated here rather than
left to the reader of the count.

The rendering control is a mutation: every `Display`, `Debug` and serialization
impl reachable from a returned non-success value is replaced by a constant, and
the whole negative suite is re-run. Every axis determination must still succeed
and return the same axis. An axis whose determination breaks under that mutation
was reading the rendering, which FR-008-AC-2 forbids and which this control is
how one sees.

The out-of-band control is two-sided: the static site count, and an execution of
the negative suite with the process's output streams captured and asserted
empty. Either alone is insufficient — a static check misses a logging facade
behind a trait object, and a captured-stream check misses a write to a global.

The accessor control is a compilation control: making one relied-on accessor
private must fail the discrimination harness to compile. This is the cheapest of
the controls and the one that keeps the others honest, because a determination
that compiles only against private state is not a determination a caller can
make.

Each criterion states the mutation the check must refuse. A criterion whose
mutation cannot be run is recorded as unexecutable rather than as passing.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-008-AC-1 | Every owed refusal axis is determined by a total function over public typed accessors on the returned value, taking no input but that value; an axis whose determination needs a file, an environment variable, a global, a clock or a second call fails the criterion and names that input. | Test (TC-011) |
| NFR-008-AC-2 | Replacing every `Display`, `Debug` and serialization impl reachable from a returned non-success value with a constant changes no axis determination; an axis that stops being determinable, or becomes a different axis, under that replacement fails the criterion and names that axis. | Test (TC-011) |
| NFR-008-AC-3 | No refusal path writes to an output stream, a file, a process global, a thread-local or a logging facade; introducing one such write is refused by the static site count, and the negative suite run with the process's output streams captured asserts both streams empty. Introducing a write that only one of those two controls would catch still fails. | Inspection |
| NFR-008-AC-4 | No refusal path panics, aborts, unwraps or expects in place of returning a value; replacing one refusal return with a panic is refused and names that site. The three occurrences in `src/ids.rs` and `src/repro.rs` are inside doc comments and are excluded by the non-doc denominator rather than by a waiver. | Inspection |
| NFR-008-AC-5 | Every envelope component any acceptance criterion relies on is reachable through a public typed accessor on the returned value; making one relied-on accessor private, or moving one component behind a rendered form only, fails the discrimination harness to compile rather than silently weakening a check. | Inspection |
| NFR-008-AC-6 | An absent location, stage, prerequisite or cause chain compares unequal to a present component of that kind whose content is empty, and is never defaulted to it; a fixture pair differing only in absent-versus-empty that compares equal fails the criterion and names the component. | Test (TC-011) |
| NFR-008-AC-7 | Two runs on identical inputs return non-success values that compare equal component by component, with the observed elapsed time of a resource-incomplete outcome excluded; a value carrying an allocation address, a wall-clock reading, a locale-dependent rendering or a map-iteration artefact fails the criterion and names that component. The reproduction method is [NFR-001](./NFR-001-reproduce-semantic-results.md)'s and is not restated here. | Test (TC-008) |
| NFR-008-AC-8 | An owed refusal axis that cannot be observed because its contract version or its diagnostic catalog is `unresolved` under [FR-009](../functional/FR-009-lock-external-producer-revisions.md) is reported unexecutable and counted as zero in the denominators above; reporting such an axis as passing, or omitting it from the denominator, fails this criterion. | Inspection |

## Dependencies

- **Upstream**: [FR-008](../functional/FR-008-return-typed-refusals.md) owns the
  refusal envelope whose components must be observable, and FR-008-AC-2 already
  forbids deriving identity from text; this requirement supplies the control that
  makes that prohibition checkable.
  [FR-014](../functional/FR-014-typed-outcome-identity.md) owns the typed outcome
  identity, the axis enumeration, and the injectivity this requirement requires
  to be decidable from the value.
  [NFR-002](./NFR-002-enforce-resource-bounds.md) owns the resource-incomplete
  outcome and the elapsed-time figure AC-7 excludes.
  [NFR-001](./NFR-001-reproduce-semantic-results.md) owns the reproduction
  method AC-7 relies on.
  [NFR-005](./NFR-005-retain-composed-acceptance-evidence.md) owns the per-axis
  distinctness check that consumes what this requirement makes observable.
- **Boundary**: [NFR-003](./NFR-003-preserve-rust-assurance-boundary.md) owns
  language and containment; it forbids `unsafe` and it forbids local runners, and
  it forbids neither an out-of-band write nor a panic, which is why AC-3 and AC-4
  are stated here.
- **Upstream gaps cited, not closed**: `ix://agent-ix/quire-specification/FR-270`
  through `FR-274` own the envelope's interior. FR-270's catalog is not an adopted
  selection and is an `unresolved` FR-009 entry; FR-271 through FR-274 each record
  an open gap that FR-008 and FR-014 enumerate. No member list, key, order or
  spelling is supplied here.
- **Downstream**: [TM-001](../tests.md) does not yet carry a row for this
  requirement, and neither TC-008's nor TC-011's Traceability yet names it. Both
  are outstanding back-edges, not claims of coverage.

## Status

Draft, and unexecuted. Nothing here asserts that any refusal has been observed.

- The refusal envelope does not exist yet. `src/refusal/mod.rs` is a facade
  documenting `ProtocolRefusal` as owner chunk **C-01**, and no
  `ProtocolRefusal` type is defined anywhere in `src/`. Every criterion stating a
  property of the returned value is therefore an obligation on a type that is
  planned, not a measurement of one that exists.
- AC-3 and AC-4 are the two criteria measurable against the tree today, and both
  hold so far: `src/` contains no `println!` or `eprintln!` at all, and outside
  doc comments and `#[cfg(test)]` modules no `panic!`, `unwrap()` or `expect(`.
  That is a baseline worth recording precisely because it is cheap to lose once
  the envelope lands — and it was nearly lost by C-13, whose `Diagnostics::first`
  carried an `expect` asserting an invariant the type did not enforce until the
  type was reshaped to enforce it.
- AC-1, AC-2, AC-6 depend on TC-011, which [TM-001](../tests.md) records as
  planned. AC-7 depends on TC-008, whose reproduction-record slice has executed
  and whose full mutation matrix has not.
- AC-8 is the honest denominator: the diagnostic catalog is an `unresolved`
  FR-009 entry and the version 2 contract version is another, so every axis whose
  observation depends on either is currently counted as zero.

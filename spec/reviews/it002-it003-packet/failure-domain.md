---
id: SR-045
title: "Failure-domain review of the FR-009/NFR-005 composed-integration packet"
type: SpecReview
analysis: failure-domain
review_set: subset
scope: "spec/functional/FR-009-lock-external-producer-revisions.md; spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md; spec/integration/IT-002-consume-observation-temporal-facts.md (owner-boundaries table, SC-05..SC-08); spec/integration/IT-003-publish-assurance-and-e02-results.md (owner-boundaries table, SC-06..SC-09); spec/assurance/AD-001-protocol-library-boundaries.md (qualification-evidence intake); spec/tests.md rows for FR-009 and NFR-005"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-003, type: reviews }
  - { target: ix://agent-ix/quire-protocol/AD-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
---
## Summary

This failure-domain analysis examines the newly authored version-lock manifest
requirement (FR-009), the composed-evidence requirement (NFR-005), the four new
IT-002 scenarios and five new IT-003 scenarios, the two new owner-boundaries
tables, AD-001's qualification-evidence intake allocation, and the two new
TM-001 rows, against the four checklist domains: extension-point failure policy
at trust boundaries, entity identity, evaluation purity, and topological
robustness.

Seventeen findings are recorded, nine of them high. The high ones cluster in
three places. Manifest resolution is specified as an operation that cannot be
performed inside the boundary AD-001 places it in, is undefined for half the
producers it names, has no uniqueness key, and is authorised by the same
document it governs. Owner identity is retained at a granularity coarser than
the substitution the packet exists to forbid, and one owner that crosses IT-002's
boundary has no manifest entry at all. And NFR-005's obligations are measured
over a denominator that is presently empty, while IT-002-SC-05's correspondence
claim is satisfiable by an implementation that performs no incremental
assessment.

The budget probes of IT-002-SC-06 and IT-003-SC-07 were read as budget probes —
input held constant, budget varied — and are not re-raised as an inclusive-maximum
inversion. The two-valued refusal oracle, selection-set divergence and
unfalsifiable "unchanged" clauses recorded in SR-037..SR-044 are treated as
closed by this packet and are not restated.

## Verdict

**CONDITIONAL** — the packet adds the two requirements the composed integrations
were missing, and the scenario shapes are right. It is not yet safe to task.
FND-001 through FND-005 mean the manifest cannot be built as specified;
FND-006 through FND-009 mean the evidence it would feed cannot discriminate the
failures it exists to discriminate. Those nine should close in the FR-009,
NFR-005 and AD-001 text before any producer revision is pinned. The remaining
eight can close as the lanes are built out.

## Assurance Context

Evaluated on `agent-b/it001-v2-intake` with the packet in the working tree. No
producer revision is pinned, no manifest artefact exists in the repository yet,
and neither integration has an executable lane, so every finding below is a
statement about the specification text, not about observed behaviour.

Unavailable selected context, declared rather than reconstructed: the concrete
shape of the version-lock manifest (no schema or example is authored anywhere in
the packet), Engineering Assurance's incomplete-result vocabulary, Quoin's
evidence intake contract, and A's unpublished `/2` boundary. Findings that
depend on those are phrased as missing constraints, not as defects in the
producers.

## Checklist Application

**1. Extension points (trust boundaries).** The packet adds one new trust
boundary — manifest resolution — ahead of all the others, and six producers
behind it. Its failure policy is stated for four input conditions (absent,
unpinned, provisional, digest-mismatched) and for none of the boundary's own
failure modes: an unreadable manifest, a manifest that fails its own schema, an
artefact that resolves but cannot be read, a digest computation that cannot run,
or an owner history that cannot be consulted. See FND-001, FND-002, FND-012.

**2. Entity identity.** Three identity families are introduced without
uniqueness keys: the manifest entry (FND-003), the owner as it appears in the
two new boundary tables and in NFR-005-AC-2 (FND-005, FND-006), and the
"scenario" that NFR-005-AC-3 requires to be distinct (FND-008).

**3. Evaluation purity.** Resolution is specified to depend on another
repository's published history, which is neither pure nor local (FND-001), and
on a digest whose input bytes are undefined for non-artefact producers
(FND-002). The retained evidence includes a non-deterministic quantity
(FND-015).

**4. Topological robustness.** FR-009-AC-5 asserts entry independence while the
`/2` note asserts an inter-entry dependency relation; no resolution order,
acyclicity rule or termination argument is stated over that relation (FND-016).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Manifest resolution is placed inside a boundary that may not perform it. AD-001's new intake section puts the manifest and the per-scenario evidence record inside the system boundary, while AD-001 Views still states that no boundary "reads the filesystem, spawns a process, accesses a network, records evidence". FR-009 requires the library to resolve each artefact, recompute its digest, and confirm that a revision "is reachable from its owner's published history" — three operations that need at minimum filesystem access and, for reachability, access to another repository's history. Reachability is also not a function of the manifest: a force-push or a retention policy in the owner's repository changes the resolution outcome of an unchanged manifest, so FR-009-AC-2 is not reproducible and collides with NFR-001. | FR-009 Behavior; FR-009-AC-2; AD-001 Views; AD-001 Qualification-evidence intake; NFR-001 | wrong-requirement |
| FND-002 | high | Digest verification is undefined for half the producers FR-009 names. FR-009 Inputs enumerates six producers, three of which are not byte artefacts: the Engineering Assurance "typed producer-execution seam", the E02 "strict reader", and the "Quoin evidence interface". The requirement gives no rule for what bytes are digested for an interface, a transport seam, or a running service, and no canonicalisation rule for what is digested even in the artefact cases. FR-009-AC-1 and FR-009-AC-3 are therefore vacuous for three of six producers — and IT-003 consumes exactly those three — while an implementation that digests nothing for them satisfies both criteria. The digest algorithm identity is likewise unstated, so two digests computed under different algorithms compare as unequal strings and refuse for the wrong reason. | FR-009 Inputs; FR-009-AC-1; FR-009-AC-3; IT-003 Owner boundaries; IT-003-SC-09 | missing-requirement |
| FND-003 | high | Manifest entries have no uniqueness key and no duplicate rule. FR-009-AC-5 requires that "a manifest carrying two versions of one producer resolves each version independently", which presupposes a key of at least (producer, contract version) — but no key is declared anywhere. Consequently: two entries naming the same owner and version with different digests have no stated outcome; whether "owner" or "producer" or "contract version" identifies an entry is undecided, so it is undefined whether F's observation contract and F's output-mapping contract are one entry or two; and the "entry that is not pinned" that FR-009-AC-2's refusal must name has no name to carry. This is the SR-038 FND-008 identity defect reappearing on a new entity family. | FR-009-AC-2; FR-009-AC-5; FR-009 Inputs | missing-requirement |
| FND-004 | high | The manifest declares its own requiredness, so it authorises itself. FR-009 refuses when "any required manifest entry is absent", and requiredness is defined by the manifest's own field naming "the integration scenarios that depend on it". A scenario blocked by the deliberately unresolved A `/2` entry can therefore be made resolvable by deleting that scenario from the entry's dependency list — no producer value changes, no digest changes, and every acceptance criterion still passes. This is structurally the self-authorising offer that FR-009's own sidecar clause and AD-001's intake section forbid a producer from making, relocated into B's document. Nothing derives the required set from the scenarios' actual consumption. | FR-009 Description; FR-009 Inputs; FR-009-AC-2; AD-001 Qualification-evidence intake; IT-002 Preconditions; IT-003 Preconditions | missing-requirement |
| FND-005 | high | PT02 crosses IT-002's owner boundary with no manifest entry. IT-002's new owner-boundaries table lists PT02 as the owner of the complete-global-conformance closure record and its premises, and IT-002 Preconditions asserts that "every external producer consumed by this integration resolves through the FR-009 version-lock manifest before any fact is read". FR-009 Inputs enumerates A, E, F, Engineering Assurance, E02 and Quoin — PT02 is absent. Either the precondition is false as authored, or the closure record is consumed unpinned, which is exactly the condition FR-009 exists to refuse. IT-002-SC-04 mutates PT02 premises and IT-002-SC-08 claims manifest coverage for the scenario set that includes it. | IT-002 Owner boundaries; IT-002 Preconditions; IT-002-SC-04; IT-002-SC-08; FR-009 Inputs | missing-requirement |
| FND-006 | high | Per-value attribution is owner-level while the forbidden substitution is entry-level. NFR-005-AC-2 requires each consumed value to retain "its owner"; IT-002 and IT-003 both forbid reporting a scenario as executed "by substituting one contract version for the other". A `/1` value and a `/2` value share the owner A, so the retained field cannot distinguish them, and the substitution the preconditions name as the principal hazard is invisible in the evidence that was added to catch it. The same coarsening hides F's observation contract behind F's output-mapping contract. IT-003's own Expected Results claims "every outcome is attributable to one pinned producer revision", which owner-level retention does not deliver; the per-scenario manifest snapshot of NFR-005-AC-1 does not close the gap, because a snapshot containing several entries for one owner still cannot say which entry a given value came from. | NFR-005 Statement; NFR-005-AC-1; NFR-005-AC-2; IT-002 Preconditions; IT-003 Preconditions; IT-003 Expected Results | wrong-requirement |
| FND-007 | high | Batch/incremental correspondence is vacuously satisfiable. IT-002-SC-05 requires batch and settled-incremental assessment of the same history to agree on outcome, settled truth and closure, and requires equal-time permutations to change neither. An "incremental" implementation that buffers every fact and runs the batch assessor once at the end satisfies every clause exactly — including the permutation clause, which becomes trivially true — while performing no incremental settling at all. The one anti-vacuity control ("changing an authored premise changes both in the same way") passes for the buffering implementation too. Nothing requires any value to settle strictly before the final fact arrives, nothing asserts on an intermediate state, and no input obligation guarantees the corpus contains a history with an early settlement point. FR-004-AC-4 has the same shape and adds no witness. | IT-002-SC-05; IT-002 Inputs; FR-004 Behavior; FR-004-AC-4 | missing-requirement |
| FND-008 | high | NFR-005-AC-3's distinctness unit is undefined, and both readings break. The criterion forbids two refusing "scenarios within one integration" from retaining the same typed outcome identity. Read at SC granularity it is too coarse to see anything that matters: IT-002-SC-08 and IT-003-SC-09 each mutate many manifest entries one at a time inside a single SC, so the code-collapse across those rows — the SR-027 defect — is entirely invisible to the check. Read at row granularity it is false: every absent-entry refusal legitimately shares one FR-008 domain and code and differs only in the affected identity, so the criterion would fail a correct implementation of IT-002-SC-08. The criterion also makes its own oracle depend on how many scenarios happen to be authored — adding one can retroactively fail another, and splitting an integration makes a shared identity legal. | NFR-005-AC-3; NFR-005 Verification; IT-002-SC-08; IT-003-SC-09; FR-008-AC-5 | wrong-requirement |
| FND-009 | high | Every NFR-005 metric is denominated on executed scenarios, and no scenario executes. All five rows of the Measurement table are percentages of "executed scenarios" or of "refusing scenarios"; TM-001 records IT-002 and IT-003 as blocked on external producer acceptance, so the denominator is presently zero and all five metrics read 100% against a 100% threshold while nothing has been observed. A blocked integration therefore scores a perfect NFR-005, and will keep scoring one as scenarios are added but not run. No metric counts planned-but-unexecuted scenarios, and no threshold constrains that count. | NFR-005 Measurement and Evaluation; NFR-005 Verification; spec/tests.md IT-002 and IT-003 rows | wrong-requirement |
| FND-010 | high | "Incomplete" carries three unreconciled senses and no requirement owns the one IT-003-SC-06 tests. NFR-002 uses resource-incomplete as a typed cause inside a refusal; IT-003-SC-06 requires an incomplete outcome and a refusal to be "structurally distinct" at the same boundary; NFR-005-AC-5 uses incomplete as an execution status meaning the scenario claims no acceptance credit. FR-008 states that when any public operation cannot return its declared success value it returns one typed refusal envelope — so by FR-008 an incomplete outcome is a refusal, and SC-06 has nothing to be distinct from. No FR introduces incompleteness as a third outcome kind alongside success and refusal, and SC-06's traceability line attributes it to FR-008, which does not contain it. The scenario also cannot be constructed from inside B: incompleteness originates in the Engineering Assurance transport, whose incomplete vocabulary is undeclared, so "carries the reason it is incomplete" has no vocabulary to draw from. | IT-003-SC-06; IT-003 Traceability; FR-008 Description; FR-008-AC-5; NFR-002 Scope; NFR-005-AC-5 | missing-requirement |
| FND-011 | medium | NFR-005 requires retention and simultaneously disclaims it, leaving the record with no lifetime and no consumer. The Statement says the library SHALL retain five things per scenario; the Scope says retention and storage do not apply and belong to Quoin. Nothing binds the two: no requirement states that the NFR-005 record is the thing handed to IT-003-SC-04's Quoin boundary, nor when it may be discarded, nor who may read it. A record held in a local variable for the duration of the scenario and then dropped satisfies every acceptance criterion, and the composed evidence never reaches the external store that the packet says owns it. | NFR-005 Statement; NFR-005 Scope; NFR-005 Dependencies; IT-003-SC-04 | missing-requirement |
| FND-012 | medium | Digest verification and consumption are not bound to the same bytes. FR-009 requires the digest be recomputed and compared "before that artifact reaches any consumer path", which orders the two acts but does not require the verified bytes to be the consumed bytes. Nothing forbids re-reading the artefact after verification, re-resolving an entry part way through a scenario, or two scenarios in one run resolving the same entry to different bytes. The classic time-of-check/time-of-use window is open, and the per-scenario snapshot of NFR-005-AC-1 records only what was resolved, not that it stayed resolved. | FR-009 Behavior; FR-009-AC-3; NFR-005-AC-1 | missing-requirement |
| FND-013 | medium | The equal-time permutation clause presupposes an undeclared key and can be a singleton. IT-002-SC-05 permutes "the incremental arrival order within equal time", but the time attribute belongs to F's observation facts, its equality relation is not declared, and FR-004 speaks instead of an authored partial order — arrival order and authored order are used interchangeably without being related. If no two facts in the corpus share a time, every equal-time class has size one and the permutation set is a singleton, so the clause is satisfied by doing nothing. IT-002 Inputs obliges no history containing an equal-time class of size two or more. | IT-002-SC-05; IT-002 Inputs; FR-004 Behavior | missing-requirement |
| FND-014 | medium | The batch/incremental divergence refusal has no owning component and no typed identity. IT-002-SC-05 requires that an incremental run settling a value the batch run leaves pending "refuses rather than reporting agreement". Deciding that requires comparing two complete runs, which is not among the eight pure boundaries AD-001 enumerates, and no FR allocates a cross-run comparator to any component. The refusal is therefore either a harness assertion — in which case it is not a typed library outcome and cannot satisfy NFR-005-AC-3's identity requirement — or an unallocated B-owned capability. Which it is determines whether B is building an evaluator duplicate. | IT-002-SC-05; AD-001 Views; FR-004-AC-4; NFR-005-AC-3 | missing-requirement |
| FND-015 | medium | Budget retention is singular where the budget is a vector, and one of its dimensions is non-deterministic. NFR-005 requires "the budget granted and the budget consumed"; NFR-002 declares roughly nineteen independent limit dimensions, and IT-002-SC-06 and IT-003-SC-07 vary the budget per dimension. A scenario retaining one scalar pair satisfies NFR-005-AC-4 while saying nothing about which dimension was exhausted — the very fact the budget probes exist to establish. Retaining the elapsed-time dimension additionally makes the record non-reproducible, so two runs of one unchanged scenario retain different evidence, which NFR-005 does not say is permitted and NFR-001 would otherwise forbid. | NFR-005 Statement; NFR-005-AC-4; NFR-002 Statement; IT-002-SC-06; IT-003-SC-07; NFR-001 | missing-requirement |
| FND-016 | medium | Entry independence and entry dependency are asserted in the same requirement, with no resolution order over the latter. FR-009-AC-5 requires that adding, removing or repinning one entry changes no other entry's resolution, while FR-009 Dependencies says of the unresolved `/2` entry that "no other entry depends on it" — a statement with content only if entries may depend on one another. If they may, FR-009 states no resolution order, no acyclicity requirement, and no termination argument for a manifest whose entries form a cycle; if they may not, the `/2` note is describing a relation the requirement forbids and the deliberate unresolution rests on nothing. | FR-009-AC-5; FR-009 Dependencies; IT-002 Preconditions | missing-requirement |
| FND-017 | low | NFR-005-AC-5 is verified against a hand-maintained table and obliges no negative control. The criterion says a scenario that cannot retain the required evidence "reports incomplete rather than passing, and its matrix row stays unexecuted", verified by Inspection. The second half's oracle is the TM-001 status column in spec/tests.md, a document edited by hand rather than an observable of the system, so the criterion can be satisfied by editing the table. The first half needs a scenario deliberately deprived of one retained field, and no input, scenario or dependency obliges one to exist, so the clause is unfalsifiable as authored. | NFR-005-AC-5; NFR-005 Verification; spec/tests.md | correct-requirement-no-evidence |

## Coverage assessed

| Criterion | Status | Evidence |
| --- | --- | --- |
| FR-009-AC-1 | Blocked | Vacuous for the three non-artefact producers (FND-002) and unperformable inside AD-001's boundary (FND-001). No manifest artefact exists to inspect. |
| FR-009-AC-2 | Blocked | The refusal has no entry name to carry (FND-003) and requiredness is self-declared (FND-004). |
| FR-009-AC-3 | Blocked | Digest input bytes and algorithm undefined (FND-002); check and use not bound to the same bytes (FND-012). |
| FR-009-AC-4 | Planned | The sidecar prohibition is well formed and falsifiable as written; its removal control ("removing the sidecar changes no resolution outcome") is the strongest anti-vacuity clause in the packet. |
| FR-009-AC-5 | Blocked | Contradicted by the inter-entry dependency the same requirement asserts (FND-016); presupposes a key that is not declared (FND-003). |
| NFR-005-AC-1 | Planned | Records the snapshot but cannot attribute a value to an entry within it (FND-006). |
| NFR-005-AC-2 | Not evidenced | Owner-level granularity cannot see the substitution the preconditions forbid (FND-006), and one boundary-crossing owner has no entry (FND-005). |
| NFR-005-AC-3 | Blocked | Distinctness unit undefined; both readings fail (FND-008). |
| NFR-005-AC-4 | Partial | Scalar retention satisfies the text while losing the per-dimension fact the budget probes establish (FND-015). |
| NFR-005-AC-5 | Not evidenced | No negative control obliged; oracle is a hand-maintained table (FND-017). |
| IT-002-SC-05 | Blocked | Satisfiable by an implementation that performs no incremental assessment (FND-007); permutation clause may be a singleton (FND-013); divergence refusal unallocated (FND-014). |
| IT-002-SC-06 | Planned | Read as a budget probe with the input held constant. The three-point shape (exact, one short, fresh) is correct and is not re-raised; its only gap is per-dimension evidence (FND-015). |
| IT-002-SC-07 | Partial | Inherits FND-006, FND-008, FND-011 from NFR-005. |
| IT-002-SC-08 | Partial | One-entry-at-a-time mutation is the right shape; its refusals collide with NFR-005-AC-3 (FND-008) and cannot name entries that have no key (FND-003). |
| IT-003-SC-06 | Blocked | No requirement owns the outcome kind it tests (FND-010). |
| IT-003-SC-07 | Planned | Budget probe, same as IT-002-SC-06. |
| IT-003-SC-08 | Partial | Same as IT-002-SC-07. |
| IT-003-SC-09 | Partial | Same as IT-002-SC-08, plus FND-002 for its three non-artefact producers. |
| AD-001 intake allocation | Conditional | The index/produce split is the right cut and correctly excludes the store, catalogue, sufficiency decision and disposition path. It is contradicted by the unchanged purity sentence two paragraphs below it (FND-001). |
| TM-001 FR-009, NFR-005 rows | Partial | Both rows are honestly marked planned and blocked. Neither records that NFR-005's metrics read 100% while blocked (FND-009). |

## Proposed additions

The smallest wording that closes each high finding. None is authored here.

* **AD-001, FR-009.** State where resolution runs. Either the purity sentence
  gains an explicit exception for manifest resolution — naming the one boundary
  permitted to read artefact bytes, and requiring every later boundary to
  receive already-resolved values — or resolution moves to the caller and FR-009
  constrains what B accepts rather than what B reads. Replace "reachable from its
  owner's published history" with a property computable from the manifest and the
  artefact alone. (FND-001)
* **FR-009 Inputs.** Give each producer an intake kind. For artefact producers,
  state the digest algorithm and the exact byte sequence digested; for interface
  and transport producers, state the pinned surface and what stands in for a
  digest, or say plainly that they are pinned by revision without digest
  verification. (FND-002)
* **FR-009 Inputs.** Declare the manifest entry key — at minimum owner plus
  contract identity plus contract version — and state that two entries sharing a
  key refuse the manifest as a whole. (FND-003)
* **FR-009.** Derive the required entry set from what each scenario consumes,
  not from a field the manifest carries; a scenario that consumes a producer
  whose entry is unresolved is unexecuted regardless of what any dependency list
  says. (FND-004)
* **FR-009 Inputs, IT-002.** Add PT02 to the producer set, or state explicitly
  why the complete-global-conformance closure record is exempt from pinning and
  correct IT-002's precondition accordingly. (FND-005)
* **NFR-005.** Retain, per consumed value, the manifest entry key rather than
  the owner, and require that the entry key resolve within the scenario's own
  retained snapshot. (FND-006)
* **IT-002 Inputs, SC-05.** Oblige at least one history in which a value settles
  strictly before the final fact, and assert on the incremental run's state at
  that point; an incremental run whose first settled value coincides with its
  last fact does not discharge the correspondence claim. (FND-007)
* **NFR-005-AC-3.** Fix the distinctness unit at the refusing *row*, and define
  the compared identity as the FR-008 envelope's domain, code, stage and
  affected identity together, so that per-entry refusals sharing one code
  remain distinct by the entry they name. (FND-008)
* **NFR-005 Measurement.** Denominate on planned scenarios, not executed ones,
  and add a metric counting planned-but-unexecuted scenarios with the external
  blocker each waits on. (FND-009)
* **FR-008 or a new FR.** Introduce incompleteness as a named outcome kind with
  its own reason vocabulary, or delete IT-003-SC-06's distinctness clause and
  state that an incomplete result is one refusal domain among others. (FND-010)

## Gates run

This review edits one Markdown file under `spec/reviews/` and no Rust source, so
the Cargo gate of NFR-004-AC-1 has nothing to detect here and was not run.
The applicable specification gate was run:

- `quire validate --scope /home/peter/dev/quire-protocol "spec/**/*.md"` — see
  the report accompanying this document for its result.

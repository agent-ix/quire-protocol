---
id: NFR-007
title: "Hold each contract version to its own entry point"
type: NFR
quality_attribute: compatibility
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-002
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-008
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-004
    type: depends_on
---
# NFR-007: Hold each contract version to its own entry point

## Statement

The library SHALL expose each published producer contract version through its own version-explicit entry point and no shared one.
The library SHALL refuse bytes of any contract version other than the one an entry point names, without detecting, upgrading or downgrading them.
The library SHALL leave every already-published version-explicit path's public signature, admitted identities and per-axis refusal set unchanged when a later version's path is added.
The library SHALL discriminate on a later version's path every selection axis an earlier version's path discriminates.

## Scope

Applies to the compiled-protocol contract versions this repository consumes
through its public Rust boundary: the version 1 path `admit_and_link`, which exists,
and the version 2 path named `admit_and_link_v2` in
[IT-001](../integration/IT-001-consume-native-protocol-artifact.md) SC-06, which
does not. Applies to the public surface of each path — its signature, its
admitted identity set, and the set of typed refusals it can return.

Does not apply to:

- What either contract *means*. The wire profiles, the temporal binding
  cardinality/uniqueness/sort rule, and the numeric domains are the producer's
  and upstream's. IT-001 records the producer's binding rule verbatim and states
  the limit on relying on it; this requirement restates none of it.
- Which typed refusal a cross-version submission returns.
  [FR-008](../functional/FR-008-return-typed-refusals.md) owns the envelope and
  [FR-014](../functional/FR-014-typed-outcome-identity.md) owns its identity and
  the injectivity that keeps two axes from colliding. This requirement requires
  *a* typed refusal and defers its identity to those two.
- Cargo configuration lanes. Adding a contract-version path is not a
  configuration lane: IT-001 Notes records that the version 2 lane adds no
  `[features]` entry and no `cfg` attribute, so the crate stays single-lane, and
  [NFR-004](./NFR-004-define-local-verification-gate.md)-AC-7 continues to own
  what would oblige a second lane. A contract version and a build configuration
  are different axes and this requirement conflates neither with the other.

### Boundary against the neighbouring non-functional requirements

| Neighbour | What it owns | What this requirement owns instead |
| --- | --- | --- |
| [NFR-001](./NFR-001-reproduce-semantic-results.md) | That identical identities reproduce identical results across equivalent presentations, and that a one-axis identity change either changes the result identity or refuses. | That a *contract-version* change refuses rather than being absorbed. NFR-001's equivalence classes are presentations of one contract; this requirement's are two contracts, which are never equivalent. |
| [NFR-002](./NFR-002-enforce-resource-bounds.md) | The budgets each path enforces. | That each path is reached explicitly. A correctly bounded shared entry point would still violate this requirement. |
| [NFR-004](./NFR-004-define-local-verification-gate.md) | Configuration lanes and the commands that cover them. | Contract-version paths, which are neither features nor `cfg` selections. |
| [NFR-006](./NFR-006-fix-the-consumed-dependency-set.md) | That the closure and the delivery surface are fixed. | That a fixed delivery of one version cannot be consumed as another. |

## Rationale

Two published contract versions of one producer are the most plausible way this
consumer silently admits the wrong thing. The failure is not that version 2 is
rejected — it is that version 1 bytes are quietly *accepted* by a version 2 reader that
happens to tolerate them, producing a result attributed to a contract the
producer never emitted. IT-001 forecloses that by construction: "Neither version
may select, repair, upgrade, or downgrade the other, neither may be detected from
the offered bytes, and the two mutually refuse."

The obligation nothing declares is the other direction. Adding a version 2 path is a
change to a crate that already has a version 1 path, and the standard way a version
two damages a version one is by refactoring it — widening a signature, merging
two refusal variants, or reimplementing version 1 in terms of version 2 because the shapes
are close. IT-001-SC-06a is the falsifiable claim that this did not happen:
"`admit_and_link` is unchanged by the addition of version 2", shown by re-running
SC-01 through SC-05 on the same version 1 fixture at the revision that adds version 2 and
obtaining the same admitted identities and the same typed refusal per axis as at
the revision before it, with "`admit_and_link`'s signature and refusal set
unchanged." That is a requirement on this repository, stated only inside an
integration scenario, and this requirement is where it belongs.

The composition relation makes the fourth statement necessary rather than
decorative. IT-001 Dependencies records that `v2::Expected` composes the version 1
expectation rather than replacing it, so the changed-selection axes "divide into
inherited axes, which version 1 already discriminates, and the temporal axes this
contract adds." A version 2 path that inherits version 1's expectation but loses one of
version 1's discriminations is a regression the version 1 lane cannot see, because the
version 1 lane still refuses correctly.

## Measurement and Evaluation

Every metric is denominated on the axes and scenarios the contract pair
**owes**, not on those executed, so an unexecutable version scores zero rather
than a vacuous hundred per cent.

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Published contract versions reachable only through their own version-explicit entry point | Every published version | 100%, 0 shared entry points | Inspection |
| Code paths selecting a contract version from the offered bytes | None | 0 paths | Inspection |
| Cross-version submissions returning a typed refusal before a linked package is exposed | Both ordered pairs | 2 of 2 | Test (IT-001) |
| version 1 scenarios whose admitted identities or per-axis typed refusal changed at the revision adding version 2 | None | 0 of the owed SC-01..SC-05 scenarios | Test (IT-001) |
| Public items of the version 1 path added, removed or changed in signature at the revision adding version 2 | None | 0 items | Analysis |
| Inherited selection axes that version 1 refuses and the later path admits | None | 0 axes | Test (IT-001) |
| Cross-version refusals sharing one typed outcome identity with each other or with any other axis of the integration | None | 0 collisions | Test (IT-001) |

## Verification

The two static metrics read the crate's own public surface: the set of exported
entry points, and the absence of any path that reads a media type, schema label
or version field out of the offered bytes in order to choose a reader. Both are
offline and need no producer.

The public-surface metric is a comparison of two revisions of this repository —
the revision that adds a later version's path against its immediate predecessor
— over the earlier path's exported items only. It is an analysis rather than an
inspection because it is a diff of two surfaces, and its negative control is
stated below as a mutation of the earlier surface that the comparison must
reject.

The cross-version and inherited-axis metrics are IT-001's, exercised at the two
entry points named in its Preconditions. Each ordered pair is submitted in both
directions, because a reader tolerating the other version's bytes and a reader
refusing them are different defects in each direction.

Each criterion states the exact mutation the check must refuse. A criterion whose
mutation cannot be run today is recorded as unexecutable, not as passing.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-007-AC-1 | Each published contract version is reachable only through an entry point that names that version, and introducing an entry point serving two versions, or re-exporting one version's reader under a version-neutral name, is refused and names the offending item. | Inspection |
| NFR-007-AC-2 | No path selects a contract version from the offered bytes; introducing a dispatcher that reads a media type, schema label or version field out of the offer and chooses a reader from it is refused, and the refusal names that dispatcher. | Inspection |
| NFR-007-AC-3 | Bytes of one published contract version submitted at another version's entry point return a typed refusal before a linked package is exposed, in both ordered directions, with no upgrade and no downgrade; a reader that admits the other version's bytes fails the criterion for that direction independently of the other. | Test (IT-001) |
| NFR-007-AC-4 | The two cross-version refusals carry typed outcome identities distinct from each other and from every other refusal axis of the integration; two directions returning one shared code and cause fail this criterion, and the failure is reported against the pair rather than absorbed as a passing row. Identity and injectivity remain [FR-014](../functional/FR-014-typed-outcome-identity.md)'s. | Test (IT-001) |
| NFR-007-AC-5 | Re-running the earlier version's owed scenarios on the same fixture at the revision that adds a later version's path yields the same admitted identities and the same typed refusal per axis as at the revision before it; one differing admitted identity, or one axis whose refusal changed or disappeared, fails the criterion and names that identity or axis. | Test (IT-001) |
| NFR-007-AC-6 | The earlier path's public signature and refusal set are unchanged across that revision pair; adding a parameter, widening or narrowing a type, adding a refusal variant, removing one, or merging two into one is refused and names the changed item. Reimplementing the earlier path as a call into the later one is refused by the same comparison, because it changes the refusal set. | Analysis |
| NFR-007-AC-7 | Every inherited selection axis the earlier path refuses, the later path also refuses; a later path that admits an inherited-axis mutation the earlier path refuses fails the criterion and names that axis, and the temporal axes the later contract adds are reported separately from the inherited ones rather than counted against them. | Test (IT-001) |
| NFR-007-AC-8 | An owed cross-version or regression scenario that cannot run because its contract version is `unresolved` under [FR-009](../functional/FR-009-lock-external-producer-revisions.md) is reported unexecutable and counted as zero in the denominators above; reporting such a scenario as passing, or omitting it from the denominator, fails this criterion. | Inspection |

## Dependencies

- **Upstream**: [FR-001](../functional/FR-001-admit-selected-protocol-profile.md)
  and [FR-002](../functional/FR-002-link-finite-protocol-representation.md) own
  admission and linking at each version;
  [FR-013](../functional/FR-013-offered-versus-selected.md) owns the
  offered-versus-selected distinction that makes detection from the offer
  illegitimate; [FR-008](../functional/FR-008-return-typed-refusals.md) owns the
  refusal envelope and [FR-014](../functional/FR-014-typed-outcome-identity.md)
  its identity; [FR-009](../functional/FR-009-lock-external-producer-revisions.md)
  supplies the `pinned`/`unresolved` status that decides whether a version's
  scenarios can run at all.
- **Verified by**: [IT-001](../integration/IT-001-consume-native-protocol-artifact.md)
  SC-06 and SC-06a, and SC-01 through SC-05 as the regression baseline SC-06a
  re-runs. IT-001 does not yet declare this requirement in its Traceability or
  its relationships; that is an outstanding back-edge, not evidence.
- **Boundary**: [NFR-004](./NFR-004-define-local-verification-gate.md) owns
  configuration lanes, which a contract version is not.
- **Downstream**: [TM-001](../tests.md) does not yet carry a row for this
  requirement.

## Status

Draft, and unexecutable in the part that matters. Nothing here asserts that version 2
admission works.

- AC-1 and AC-2 are static facts over B's own public surface and are checkable
  now, but they are nearly vacuous today: the crate exports one contract-version
  path, `admit_and_link`, and `admit_and_link_v2` does not exist in `src/`.
- AC-3, AC-4 and AC-7 depend on IT-001 SC-06, which IT-001 Notes records as
  unexecuted: the version 2 reader is implemented and under final integration but published
  to no remote, FR-009 reports the A version 2 entry `unresolved`, and B pins no
  intermediate commit.
- AC-5 and AC-6 are doubly blocked. They compare the revision that adds the
  later path against the revision before it, and neither revision exists: B has
  no version 2 path to add, and no version 2 revision of A to build one against. They are
  stated now so that the comparison is an obligation before the change rather
  than a request after it.
- AC-8 is the honest denominator: every scenario the paragraphs above name is
  currently counted as zero.

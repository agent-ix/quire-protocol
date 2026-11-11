---
id: SR-2127015
title: "TL-251 spec-review/integrity review"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-protocol; TL-251; PR #142; spec/adr/ADR-006-confine-jcs-to-the-protocol-result-domain.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/ADR-006, type: references }
---
## Summary

Ticket: TL-251. Spec-review with the integrity lens, scoped to the one spec file the PR edits: ADR-006. The PR removes the citation `src/digest/jcs.rs:60` and leaves the rest of the ADR as it was.

## Verdict

**FAIL (medium).** Removing the citation was correct, because the file is gone. But the ADR still says in the present tense that this repository owns a canonicaliser. After the PR, the repository has none. The decision itself, that JCS applies only to bytes this repository produces, is still valid as a rule for future work. The statements of current fact are not.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | ADR-006 Context still says "This repository owns a canonicaliser for its result bytes". The Status line says "JCS canonicalizes bytes this repository produces". The paragraph "The canonicaliser is deliberately incomplete, and stays that way" describes how the canonicaliser behaves with JSON numbers. After this PR, no canonicaliser exists in the repository, so a reader would take a removed implementation as present. | spec/adr/ADR-006-confine-jcs-to-the-protocol-result-domain.md:20-26, 89-98 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | d3d8ca9 rewords the ADR-006 Status, Context and number-refusal paragraphs as the rule for a canonicaliser once adopted (quire-canonical) and states the repository currently contains none. |

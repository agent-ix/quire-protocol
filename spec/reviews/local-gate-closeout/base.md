---
id: SR-065
title: "Base review of the executable local-gate closeout"
type: SpecReview
analysis: base
scope: "NFR-004, TC-012, current Cargo manifest and tracked cfg sites at quire-protocol 5ce7978"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-012, type: reviews }
---
## Summary

The earlier gate specification was internally coherent at `681e25a` but no
longer described the implemented repository. The corrected NFR and test case
separate Cargo-feature selection, target kind and target platform, and make the
existing four conditional sites and isolated target directory explicit before
TC-012 is implemented.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | NFR-004 said the crate contained no conditional compilation and AC-7 required a second Cargo-feature lane for any `cfg`, while current main has three `cfg(test)` sites and one `cfg(unix)` site. Applying that rule would either reject the current gate or invent a feature lane that selects none of them. Fixed by treating Cargo feature, target kind and target platform as separate axes and requiring a named selector for every conditional site. | NFR-004 Statement, Scope, AC-5, AC-7, AC-8; `src/link/mod.rs`; `src/link/interleaving.rs`; `tests/it001_v2_handoff.rs` |
| FND-002 | medium | The enumerated clippy and test commands omitted the mandated worktree-owned `target-codex-backends` path even though AC-9 refused shared build output. Fixed by making the literal isolated directory part of both compiling commands and their expected evidence. | NFR-004-AC-1; NFR-004-AC-9; TC-012 |
| FND-003 | medium | NFR-004 still described a specification-only repository with one ignored test, contradicting the 153-test implementation at main. Fixed with the measured current conditional inventory and no claim that a non-executed target is supported. | NFR-004 Scope; `spec/spec.md` section 14 |
| FND-004 | low | The old test procedure conflated feature lanes with every `cfg` site and named no admitted target. Fixed by binding one evidence run to its exact `rustc -vV` host and making unselected conditions explicit refusals. | NFR-004 Scope; TC-012 Test Procedure |

## Disposition

All findings are remediated in the reviewed NFR-004/TC-012 revision. The scope
is executable test and gate-policy work only; it creates no workflow, generic
runner, stdout parser, portability promise, feature or TL dependency.

---
id: SR-2127041
title: "TL-10 code-review review"
type: SpecReview
analysis: code-review
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #148 head 9a7c29c; Cargo.toml, Cargo.lock, src/dispatch.rs, src/disposition.rs, src/lib.rs, tests/support/standin.rs, tests/tc_054, tests/tc_055, tests/tc_160..tc_163"
date: 2026-11-11
---
## Summary

Ticket: TL-10. Code review of PR #148 at head 9a7c29c, scoped to `git diff origin/main...HEAD`, with the Rust lane (rust-review, Agent IX rust-style) folded in. Focused checks on the head: `cargo metadata --locked` resolves; `cargo clippy --locked --all-targets -- -D warnings` clean; tc_054, tc_055, tc_160..tc_163 pass (11+7+9+6+5+7 tests). Full gate not run (the lead runs it once).

## Verdict

PASS WITH FINDINGS (no high).

- **Cargo.lock.** The hand patch only adds lines: one `qsl-route` package entry and two dependency lines. It removes nothing and moves no pin. `qsl-route` resolves to the same quire-spec-language revision as every other QSL crate already in the lock (91c6158). `qsl-replay` was already present. The lock resolves under `--locked`, and no SHA or digest record was added.
- **Owner types.** The PR uses `qsl_route::Registry`, `CandidateOutcome`, `Candidate`, `BackendDescriptor` and `RegistrationRefusal`, `qsl_semantics::check::Capability`, and `qsl_replay::Truth`, `SettlementBasis` and `SeparatingWitnessRecord`, with no local copies. The exception is the FR-290 settlement layer (FND-001).
- **Shared disposition types.** The changes are additive: `Option<Construct>`, `warnings`, the `Refused`/`Failed` arms, and `?Sized` on `settle_claims`. TL-11's callers (tc_054, tc_055) needed only mechanical `Some(..)`/`..` edits. The generic `P` premise slot and `Vec<Warning>` will hold TL-9's FR-200..204 premise lists and warnings without rework, as long as TL-9 adds `Warning` variants rather than a second warning type.
- **`#[allow(clippy::large_enum_variant)]` on `ClaimDisposition`.** Acceptable. The type is generic, so the size gap depends on the caller's parameters. It lives briefly in a `Vec` that callers match on, and boxing would cost every match a deref for no measured gain. The allow carries a reason comment.
- **Stand-in.** It is a `QslBoundary` registered through `ProviderBoundary::register` under a `BackendDescriptor` that QSL's own registry accepts. It is neither a per-request callback nor an entry in FR-005's table, and `dispatch` takes no closure.
- **Trace tags and pins.** Every test uses `#[trace("TC-16x", "FR-15x-AC-y")]`, and there are no `/// Trace:` lines. The PR adds no vendoring, shim, compatibility layer or new pin.
- **FR-152 join.** It is only carried: `sources` stay on the binding, and there is no graph resolution.
- **Can every test fail?** Yes. The ones that only reach ordering are FR-150-AC-7 and FR-153-AC-1, because the global result is an opaque `G` passed through. That limit comes from the design, not the tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `dispatch` works out the FR-290 settlement itself: exactly one candidate means send, an empty set means `unsupported` with a warning, several candidates mean `ambiguous-backend`. It hand-spells `CatalogCode::new("invalid_capability", "unknown-backend"/"ambiguous-backend")` and adds a local `Warning::UnservedCapability { kind, backend }`. QSL's routing.rs says negotiation is not a QSL stage but quire-contract-codegen's `negotiate_*`, under AD-016/FR-290. Codegen publishes the typed causes `routed::capability::Cause::{UnknownBackend, AmbiguousBackend { candidates }, UnsupportedRequestedCapability { kind, backend }}` with `code()`, and `qsl_route::CandidateSet` already exposes `kind()`/`requested_backend()` for this warning. Under the repo rule 'an owner publishes it, import it', import the owner's settlement or ticket the owner to publish one usable in process. At minimum, build the warning from `set.kind()`/`set.requested_backend()`. | src/dispatch.rs:327-360 |
| FND-002 | low | Branches that cannot be reached return valid but wrong values. A candidate with no provider becomes `Failed { diagnostic: None }`, and an empty `settle_claims` result becomes `Refused(NoSourceIdentity)`, which is a false cause. rust-style says not to map an impossible state onto a valid wrong value. Call `provider.report(&binding)` directly, or make the invariant explicit. | src/dispatch.rs:361-374 |
| FND-003 | low | When a registration repeats an existing descriptor exactly, `ProviderBoundary::register` returns `Ok(())` but keeps the first provider (`or_insert_with`) and silently drops the new one. The caller cannot tell that its provider was discarded. Document this, or refuse a second provider for a held identity. | src/dispatch.rs:249-267 |
| FND-004 | low | `ProtocolOutcome::truth` replaces the backend's truth with `Truth::Pending` when `inconclusive()` is set. FR-151 asks only for 'neither satisfied nor violated', and QSL's `Pending` means 'execution completed and the truth is still pending'. Choosing `Pending` over `Unavailable` has no spec line behind it. | src/dispatch.rs:171-185 |
| FND-005 | low | The FR-151-AC-2 test treats `format!("{:?}", premises.request.premises)` as 'provenance'. The code has no request-identity or provenance value, so the test checks only that the changed input is carried, not that a request identity or provenance differs. | tests/tc_161_bind_communication_and_visibility_premises.rs:98-112 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Not imported. Adding quire-contract-codegen, a heavy Kani-oriented crate, would unlock main-branch pins. The warning is now built from qsl_route::CandidateSet::kind()/requested_backend(). Only the two FR-290 CatalogCode spellings (unknown-backend, ambiguous-backend) stay local. Reviewer ruling: acceptable for this PR only if the code lead files an owner ticket for quire-contract-codegen to publish routed::capability::Cause/code() in a light crate, which this library then imports to remove the two literals. Ticket id not yet filed. |
| FND-002 | fixed | Round 1, 72743ff:     let Some(provider) = boundary.providers.get_mut(candidate.id()) else {         return ProtocolOutcome::Refused(DispositionRefusal::NoProvider(candidate.id().clone()));     };     let claim = binding.clone();     let settlement = settle_claim(provider.as_mut(), &claim, binding); |
| FND-003 | fixed | Round 1, 72743ff:             Ok(()) => match self.providers.entry(id) {                 Entry::Occupied(held) => Err(RegistrationError::ProviderHeld(held.key().clone())),                 Entry::Vacant(slot) => {                     slot.insert(Box::new(provider));                     Ok(())                 }             }, |
| FND-004 | fixed | Round 1, 72743ff: - The dispatcher shall give a disposition whose QSL verdict is `Inconclusive` the truth `unavailable`, because `ix://agent-ix/quire-specification/FR-242` reserves `pending` for an open decision scope with settlement basis `unsettled`, which an `Inconclusive` verdict does not state. |
| FND-005 | fixed | Round 1, 72743ff:     for (left, first) in retained.iter().enumerate() {         for second in &retained[left + 1..] {             assert_ne!(first, second);         }     } |

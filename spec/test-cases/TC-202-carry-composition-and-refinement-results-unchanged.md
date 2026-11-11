---
id: TC-202
title: "Carry composition and refinement results without reinterpretation"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-202
    type: verifies
---
# TC-202: Carry composition and refinement results without reinterpretation

## Description

Verify byte-equal carrying of certificates, paired counterexamples, witnesses, discharge records and causes, and the absence of derived axes, with a stand-in for the QSL call boundary and no local analysis.

## Test Procedure

Feed sentinel composition and refinement results from a stand-in, including a two-half counterexample, a one-half counterexample, each cause named in FR-202-AC-4, composition labels `proved`, `inconclusive`, `unsupported` and `incomplete` (`unsupported` beside certificate-shaped bytes), a `malformed-declaration` refusal, and unsupported dynamic behaviour. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Every artifact is byte-equal; composition labels read into execution and truth only through the FR-360 table and into basis through the FR-489 table, and `incomplete` carries basis `unavailable` and no derived execution or truth value; no half is invented; causes are unchanged and unmapped; no axis value is derived for a refinement result; `refused` appears only for a returned refusal; nothing non-success presents as satisfied.

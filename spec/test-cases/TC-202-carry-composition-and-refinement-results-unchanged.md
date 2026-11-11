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

Feed sentinel composition and refinement results from a stand-in, including a two-half counterexample, a one-half counterexample, each cause named in FR-202-AC-4, a non-completed execution beside certificate-shaped bytes, and unsupported dynamic behaviour. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Every artifact is byte-equal; no half is invented; causes are unchanged and unmapped; execution and truth are as stated, none derived; `refused` appears only when the stand-in states it; nothing non-success presents as satisfied.

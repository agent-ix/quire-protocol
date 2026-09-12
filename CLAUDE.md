# quire-protocol

The parser-free **consumer** of compiled protocol artifacts. It reads what a
producer emits. No parser, no source frontend, no encoder, no formal semantics.

This repository is one part of an ecosystem where each repository owns part of
the design. **Most types handled here belong to someone else.**

## Before you declare anything

Before declaring any type, enum member, key, wire string, or file schema, find
its owner in the spec. The spec artifacts and their `ix://` references are the
register of what exists and who owns it — that is what they are for. Then:

- **An owner publishes it** → import it. Do not mirror it, not even in tests.
- **The spec shows no owner** → stop and ask. Do not fill the gap locally.

Never stand up a local placeholder for something another repository is still
building. The placeholder is the failure, not a way around it.

## A contradiction is a stop, not a paragraph

If a requirement and the code disagree, stop and resolve it. Recording the
divergence and committing does not settle it — it preserves it.

This is not hypothetical. `FR-015-AC-5` ("No requirement, test or implementation
in this repository declares a digest domain vocabulary of its own") was authored
while `src/digest/producer.rs` declared exactly such a vocabulary. The
contradiction was written down under a heading and committed. Two days of
downstream work was then built on the shadow vocabulary.

## Also read

`AGENTS.md` for the gate, the language rule, and the licensing rules.
`spec/spec.md` for what this library does.

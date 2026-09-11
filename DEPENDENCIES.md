# Dependency and rights record

`quire-spec-language` is consumed at commit
`84975dd687d38d9b24eae84b7356e3f24cdc7b42` (A's merged four-unit native
producer handoff, PR #58). Its declared license is `AGPL-3.0-only`, compatible
with this repository's `AGPL-3.0-or-later` decision. The dependency supplies the
accepted `quire.compiled-protocol/1` wire contract and public Rust reader; this
crate adds no copied parser, encoder, formal semantics, or fixture corpus.

`ix-trace-rs` at `2ce4ebf47f726b9d76388220545cd0abda8a5cfb` is a test-only trace
attribute dependency already used by the A-owned producer. Its pinned manifest
declares `AGPL-3.0-or-later`, compatible with this repository's license.

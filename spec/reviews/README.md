---
type: index
title: "Specification review index"
description: "Rules and location index for immutable specification review records."
---
# Reviews

Every file under `spec/reviews/` is a `SpecReview`: a dated record of what a
reviewer found in a named part of this repository at a named revision. Reviews
live here and nowhere else — two gap analyses were once filed under a
repository-root `reviews/`, where nothing referenced them.

## Contents

Each child directory contains the immutable review set for the scope named by
that directory; this index defines how those records are interpreted.

## A review is a record, not a standing claim

Each review's `scope:` names the revision it was taken over. Counts and
measurements inside it — "five concrete success criteria", "55 of 55
grammar-clean", "`quire coverage` 0.31.0" — are readings at that revision, not
assertions about the tree as it stands now.

So:

- **Do not rewrite a review when the spec moves past it.** Its findings were
  true, and the record of what was once true is the point of keeping it.
- **Do not cite a review's numbers as current.** Re-measure, and file a new
  review if the number matters.
- **Do pin `scope:` on every new review**, naming the revision reviewed. A
  review without one reads as a claim about the present and will go stale in a
  way nobody can date.

`spec/tests.md` is the opposite kind of document: it states what the gate
reports over the current head, and it is re-measured rather than preserved.

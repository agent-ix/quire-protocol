# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-protocol's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The outcome names come
from src/conformance/batch.rs and src/compensation/mod.rs.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: a compiled protocol plus observed traffic gives a verdict."""
    f = Figure(
        "hero",
        980,
        290,
        "A compiled protocol is admitted and linked into roles and channels. Observed traffic is "
        "then checked against it for conformance, and compensation is assessed alongside. The "
        "result is a conformance verdict and a compensation path that is available or unavailable.",
    )
    f.box(0, 30, 170, 56, "model", "compiled protocol", "from QSL, /1 or /2")
    f.box(0, 190, 170, 56, "ink", "observed traffic", "messages · effects")
    f.box(214, 90, 150, 56, "logic", "admit · link", "two separate steps")
    f.box(408, 90, 150, 56, "model", "linked protocol", "roles · channels")
    f.line([(170, 58), (190, 58), (190, 118), (211, 118)], "model")
    f.line([(364, 118), (405, 118)], "logic")
    f.line([(558, 118), (580, 118)], "model", arrow=False)
    f.line([(580, 58), (580, 178)], "model", arrow=False)
    f.line([(170, 218), (580, 218), (580, 178)], "ink", arrow=False)
    f.line([(580, 58), (602, 58)], "model")
    f.line([(580, 178), (602, 178)], "model")
    f.box(604, 30, 170, 56, "logic", "compensation", "undo steps, in order")
    f.box(604, 150, 170, 56, "logic", "conformance", "finite replay")
    f.line([(774, 58), (807, 58)], "evidence", weight=2.4)
    f.line([(774, 178), (807, 178)], "evidence", weight=2.4)
    f.box(810, 30, 170, 56, "evidence", "compensation path", "available or not")
    f.box(810, 150, 170, 56, "evidence", "conformance verdict", "conforms · violates")
    f.legend(0, 284, [ROLES[1], ROLES[2], ROLES[3], ROLES[4]])
    return f


def outcomes():
    """The per-obligation dispositions and the adverse kinds behind a violation."""
    dispositions = [
        ("conforms", "every required fact present", "logic"),
        ("pending", "a required fact may still arrive", "evidence"),
        ("violates", "an adverse observation exists", "evidence"),
        ("incomplete", "a required valuation is absent", "evidence"),
        ("unavailable", "the assessment did not complete", "evidence"),
    ]
    adverse = [
        ("unexpected", "not the expected observation"),
        ("missing", "absent at a complete boundary"),
        ("causally misordered", "dependent before predecessor"),
        ("duplicate effect", "two observations, one effect"),
        ("cross-workflow", "another workflow instance"),
    ]
    f = Figure(
        "outcomes",
        980,
        290,
        "Each obligation in a protocol ends in one of five dispositions: conforms, pending, "
        "violates, incomplete or unavailable. A violation is always backed by an adverse "
        "observation of one kind: unexpected, missing, causally misordered, duplicate effect or "
        "cross-workflow.",
    )
    f.text(0, 14, "ONE DISPOSITION PER OBLIGATION", "muted", "mono", SMALL)
    f.text(520, 14, "WHAT BACKS A VIOLATION", "muted", "mono", SMALL)
    for i, (name, why, role) in enumerate(dispositions):
        y = 24 + i * 48
        f.box(0, y, 170, 40, role, name)
        f.line([(170, y + 20), (197, y + 20)], role, arrow=False)
        f.text(210, y + 25, why, "muted", "mono", SMALL + 1)
    for i, (name, why) in enumerate(adverse):
        y = 24 + i * 48
        f.box(520, y, 200, 40, "evidence", name)
        f.line([(720, y + 20), (747, y + 20)], "evidence", arrow=False)
        f.text(760, y + 25, why, "muted", "mono", SMALL)
    return f


REPOS = [
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


def this_repo():
    """The ecosystem figure with quire-protocol highlighted."""
    return ecosystem(here="quire-protocol")


FIGURES = [hero, outcomes, this_repo]

# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-protocol's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The outcome names come
from src/slot.rs and src/checked_package.rs.
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
    """README hero: a checked handoff is admitted, then linked and evaluated."""
    f = Figure(
        "hero",
        980,
        290,
        "A checked-package handoff of four files is admitted into one checked package. The "
        "package is then linked into a protocol with its roles and channels, its slots are "
        "evaluated to a slot outcome, and its temporal obligations are assessed to an evidence "
        "record.",
    )
    f.box(0, 90, 170, 56, "model", "checked handoff", "four files from QSL")
    f.box(214, 90, 150, 56, "logic", "admit", "reader · selection")
    f.box(408, 90, 150, 56, "model", "checked package", "plus two files")
    f.line([(170, 118), (211, 118)], "model")
    f.line([(364, 118), (405, 118)], "logic")
    f.line([(558, 118), (580, 118)], "model", arrow=False)
    f.line([(580, 58), (580, 238)], "model", arrow=False)
    for y in (58, 148, 238):
        f.line([(580, y), (602, y)], "model")
    f.box(604, 30, 170, 56, "logic", "link", "roles · channels")
    f.box(604, 120, 170, 56, "logic", "evaluate slots", "through QSL")
    f.box(604, 210, 170, 56, "logic", "assess evidence", "temporal obligations")
    for y in (58, 148, 238):
        f.line([(774, y), (807, y)], "evidence", weight=2.4)
    f.box(810, 30, 170, 56, "evidence", "linked protocol", "or QSL's refusal")
    f.box(810, 120, 170, 56, "evidence", "slot outcome", "holds · refused · ...")
    f.box(810, 210, 170, 56, "evidence", "evidence record", "truth · settlement")
    f.legend(0, 284, [ROLES[1], ROLES[2], ROLES[3]])
    return f


def outcomes():
    """The slot outcomes and the reasons admission refuses."""
    slots = [
        ("holds", "completed with true", "logic"),
        ("does not hold", "completed with false", "evidence"),
        ("not Boolean", "completed with another value", "evidence"),
        ("unavailable", "a binder was supplied unavailable", "evidence"),
        ("undefined", "the kernel left it undefined", "evidence"),
        ("refused", "the kernel refused the slot", "evidence"),
        ("incomplete", "a resource bound stopped it", "evidence"),
        ("family result", "the slot's family answered itself", "evidence"),
    ]
    refusals = [
        ("reader", "the package reader refused"),
        ("incomplete", "a read limit was reached"),
        ("selection file", "not a selection sidecar"),
        ("selection", "the package departs from it"),
        ("identities", "identities.json refused"),
        ("equality", "equality.json refused"),
    ]
    f = Figure(
        "outcomes",
        980,
        420,
        "A slot evaluation ends in one of eight outcomes: holds, does not hold, not Boolean, "
        "unavailable, undefined, refused, incomplete or a family result. Admission of a handoff "
        "fails with one of six refusals: reader, incomplete, selection file, selection, "
        "identities or equality, each carrying its owner's value unchanged.",
    )
    f.text(0, 14, "ONE OUTCOME PER SLOT EVALUATION", "muted", "mono", SMALL)
    f.text(520, 14, "WHY ADMISSION REFUSES", "muted", "mono", SMALL)
    for i, (name, why, role) in enumerate(slots):
        y = 24 + i * 48
        f.box(0, y, 170, 40, role, name)
        f.line([(170, y + 20), (197, y + 20)], role, arrow=False)
        f.text(210, y + 25, why, "muted", "mono", SMALL + 1)
    for i, (name, why) in enumerate(refusals):
        y = 24 + i * 48
        f.box(520, y, 200, 40, "evidence", name)
        f.line([(720, y + 20), (747, y + 20)], "evidence", arrow=False)
        f.text(760, y + 25, why, "muted", "mono", SMALL)
    return f


REPOS = [
    (1, 0, "quire-driver", "the quire command", False),
    (2, 0, "quire-integration", "tests the whole chain", False),
    (3, 0, "quire-verification", "plans verification", False),
    (0, 1, "quire-specification", "the standard", False),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", False),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (3, 2, "quire-eval", "evaluation core", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. quire-eval is the evaluation core. Above the chain, quire-driver runs it, "
        "quire-integration tests it and quire-verification plans verification. Below it, shared "
        "libraries used throughout: quire-exact, quire-semantic-value, quire-walk, "
        "quire-canonical and quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("model", "feeds", "fill")])
    return f


def this_repo():
    """The ecosystem figure with quire-protocol highlighted."""
    return ecosystem(here="quire-protocol")


FIGURES = [hero, outcomes, this_repo]

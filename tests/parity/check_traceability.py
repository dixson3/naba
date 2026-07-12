#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["pyyaml"]
# ///
"""SPEC <-> test traceability check (plan-004, Issue 5.3).

Asserts that every **[PINNED]** and **[NEW]** clause in ``SPEC.md`` maps to at
least one test: a parity case (``tests/parity/cases/*.yaml`` ``spec:`` field), an
MCP/parity/harness pytest module that cites the clause id, or an explicit,
justified exemption in ``tests/parity/traceability_exemptions.yaml`` (for clauses
covered by cargo unit tests, by ``test_mcp.py`` tool-inventory asserts that do not
cite a literal id, or that are help-prose/DIVERGENCE clauses pinned only at the
semantics level).

Exit codes: ``0`` = every required clause covered; ``1`` = one or more uncovered;
``2`` = usage / IO error (missing SPEC.md, malformed exemptions).

Run:  ``uv run tests/parity/check_traceability.py``
      (add ``--json`` for a machine-readable report).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

import yaml

# Clause id + the FIRST bracketed marker that follows it, e.g.
#   **SPEC-GEN-001** [PINNED]           -> PINNED
#   **SPEC-CFGSCHEMA-001** [PINNED+NEW] -> PINNED+NEW
#   **SPEC-MIGRATE-001** [NEW/RESOLVED — Concern 5] -> NEW
CLAUSE_RE = re.compile(r"\*\*(SPEC-[A-Z]+(?:-[A-Z]+)*-\d+)\*\*\s*\[([^\]]+)\]")
# Bare clause id anywhere (case yamls, pytest docstrings/comments).
ID_RE = re.compile(r"SPEC-[A-Z]+(?:-[A-Z]+)*-\d+")

# Markers whose clauses MUST be covered (by ref or exemption).
REQUIRED_PREFIXES = ("PINNED", "NEW")

# Pytest modules whose cited clause ids count as coverage.
TEST_MODULES = ("test_mcp.py", "test_parity.py", "test_harness.py")


def repo_root(start: Path) -> Path:
    """Walk up from ``start`` until SPEC.md is found (worktree-agnostic)."""
    for d in [start, *start.parents]:
        if (d / "SPEC.md").is_file():
            return d
    raise FileNotFoundError("SPEC.md not found walking up from " + str(start))


def marker_kind(marker: str) -> str:
    """Leading token of a marker: ``PINNED+NEW`` -> ``PINNED``, ``NEW/RESOLVED`` -> ``NEW``."""
    return re.split(r"[/+ ]", marker.strip(), maxsplit=1)[0]


def parse_clauses(spec: Path) -> dict[str, str]:
    text = spec.read_text(encoding="utf-8")
    out: dict[str, str] = {}
    for m in CLAUSE_RE.finditer(text):
        out.setdefault(m.group(1), marker_kind(m.group(2)))
    return out


def parse_case_refs(cases_dir: Path) -> dict[str, list[str]]:
    """clause-id -> [case files that reference it] (via any spec: list)."""
    refs: dict[str, list[str]] = {}
    for f in sorted(cases_dir.glob("*.yaml")):
        try:
            doc = yaml.safe_load(f.read_text(encoding="utf-8")) or {}
        except yaml.YAMLError as e:  # pragma: no cover - surfaced to operator
            print(f"error: {f}: {e}", file=sys.stderr)
            continue
        for case in doc.get("cases", []) or []:
            for cid in case.get("spec", []) or []:
                refs.setdefault(cid, []).append(f"{f.name}:{case.get('id', '?')}")
    return refs


def parse_test_refs(parity_dir: Path) -> dict[str, list[str]]:
    """clause-id -> [pytest modules that cite it]."""
    refs: dict[str, list[str]] = {}
    for name in TEST_MODULES:
        p = parity_dir / name
        if not p.is_file():
            continue
        for cid in set(ID_RE.findall(p.read_text(encoding="utf-8"))):
            refs.setdefault(cid, []).append(name)
    return refs


def load_exemptions(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    doc = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    ex = doc.get("exemptions", {}) or {}
    bad = [k for k, v in ex.items() if not (isinstance(v, str) and v.strip())]
    if bad:
        raise ValueError("exemptions missing a reason: " + ", ".join(sorted(bad)))
    return ex


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true", help="machine-readable report")
    args = ap.parse_args(argv)

    here = Path(__file__).resolve().parent
    try:
        root = repo_root(here)
    except FileNotFoundError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2

    spec = root / "SPEC.md"
    parity_dir = root / "tests" / "parity"
    exempt_path = parity_dir / "traceability_exemptions.yaml"

    clauses = parse_clauses(spec)
    case_refs = parse_case_refs(parity_dir / "cases")
    test_refs = parse_test_refs(parity_dir)
    try:
        exemptions = load_exemptions(exempt_path)
    except ValueError as e:
        print(f"error: {exempt_path.name}: {e}", file=sys.stderr)
        return 2

    # An exemption naming a clause that does not exist (typo / stale) is an error.
    stale = sorted(set(exemptions) - set(clauses))
    if stale:
        print("error: exemptions reference unknown clauses: " + ", ".join(stale),
              file=sys.stderr)
        return 2

    covered_by_case = set(case_refs)
    covered_by_test = set(test_refs)

    required = {c for c, k in clauses.items() if k in REQUIRED_PREFIXES}
    # DIVERGENCE clauses are informational unless listed as exemptions.
    divergence = {c for c, k in clauses.items() if k not in REQUIRED_PREFIXES}

    uncovered = []
    for cid in sorted(required):
        if cid in covered_by_case or cid in covered_by_test or cid in exemptions:
            continue
        uncovered.append(cid)

    report = {
        "total_clauses": len(clauses),
        "required": len(required),
        "divergence": sorted(divergence),
        "covered_by_case": len(required & covered_by_case),
        "covered_by_test_module": sorted(
            c for c in required if c in covered_by_test and c not in covered_by_case
        ),
        "exempted": {c: exemptions[c] for c in sorted(exemptions)},
        "uncovered": uncovered,
        "ok": not uncovered,
    }

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"SPEC clauses: {len(clauses)} total "
              f"({len(required)} PINNED/NEW required, {len(divergence)} DIVERGENCE)")
        print(f"  covered by parity cases:   {report['covered_by_case']}")
        print(f"  credited to test modules:  {len(report['covered_by_test_module'])} "
              f"({', '.join(report['covered_by_test_module']) or '-'})")
        print(f"  exempted (justified):      {len(exemptions)}")
        if uncovered:
            print(f"\nFAIL: {len(uncovered)} PINNED/NEW clause(s) with no test or exemption:")
            for cid in uncovered:
                print(f"  - {cid} [{clauses[cid]}]")
            print("\nAdd a parity case (spec: [...]), cite it in a test module, or add a "
                  "justified entry to tests/parity/traceability_exemptions.yaml.")
        else:
            print("\nOK: every PINNED/NEW clause maps to a test case or a justified exemption.")

    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

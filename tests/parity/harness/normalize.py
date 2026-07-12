"""Nondeterministic-field normalizer (SPEC-JSON-005).

Canonicalizes the fields the parity suite must ignore when comparing a captured
Result/JSON (or plain text) blob against a golden:

- ``elapsed_ms`` value -> ``<ELAPSED_MS>`` (SPEC-JSON-001).
- Timestamped auto-generated filenames/paths
  ``naba-<cmd>-YYYYMMDD-HHMMSS[-N].<ext>`` -> ``naba-<cmd>-<TIMESTAMP>.<ext>``
  (the command and extension are preserved; the timestamp and dedup index are stabilized).
- Version / commit / build-date in both output formats
  (``naba <v> (commit: <c>, built: <d>)`` and the doctor variant without colons)
  -> ``<VERSION>`` / ``<COMMIT>`` / ``<DATE>`` (SPEC-VERSION-001/002, SPEC-VERSION-BUILD-001).

``normalize`` dispatches on type: dict/list are walked recursively (``normalize_result``),
strings and everything else route through ``normalize_text``. An optional ``replacements``
map lets a case stabilize case-specific literals first (e.g. a per-case temp CWD path ->
``<CWD>``) before the structural rules run.
"""

from __future__ import annotations

import re
from typing import Any, Mapping

ELAPSED_PLACEHOLDER = "<ELAPSED_MS>"
TIMESTAMP_TOKEN = "<TIMESTAMP>"

# naba-<cmd>-YYYYMMDD-HHMMSS[-N].<ext>  ->  naba-<cmd>-<TIMESTAMP>.<ext>
# Anchored on the digit run so a leading directory prefix (absolute path) is untouched.
_AUTONAME_RE = re.compile(
    r"(naba-[a-z0-9]+-)\d{8}-\d{6}(?:-\d+)?(\.[A-Za-z0-9]+)"
)

# Version lines, both the `version` command (colons) and doctor (no colons) formats.
# Captures the colon-or-not separators so the exact punctuation is preserved.
_VERSION_RE = re.compile(
    r"naba \S+ \(commit(?P<sep1>:?) [^,]+, built(?P<sep2>:?) [^)]+\)"
)

# elapsed_ms as it appears in raw (un-parsed) JSON text.
_ELAPSED_TEXT_RE = re.compile(r'("elapsed_ms"\s*:\s*)\d+')


def normalize_text(value: str, *, replacements: Mapping[str, str] | None = None) -> str:
    """Normalize nondeterministic tokens in a plain string."""
    if replacements:
        # Apply longest literals first so a nested path replaces before its parent.
        for literal in sorted(replacements, key=len, reverse=True):
            value = value.replace(literal, replacements[literal])

    value = _AUTONAME_RE.sub(rf"\1{TIMESTAMP_TOKEN}\2", value)

    def _version_sub(m: re.Match) -> str:
        return (
            f"naba <VERSION> (commit{m.group('sep1')} <COMMIT>, "
            f"built{m.group('sep2')} <DATE>)"
        )

    value = _VERSION_RE.sub(_version_sub, value)
    value = _ELAPSED_TEXT_RE.sub(rf'\1"{ELAPSED_PLACEHOLDER}"', value)
    return value


def normalize_result(
    value: Any, *, replacements: Mapping[str, str] | None = None
) -> Any:
    """Recursively normalize a parsed JSON structure (dict / list / scalar)."""
    if isinstance(value, dict):
        out: dict[Any, Any] = {}
        for key, val in value.items():
            if key == "elapsed_ms":
                out[key] = ELAPSED_PLACEHOLDER
            else:
                out[key] = normalize_result(val, replacements=replacements)
        return out
    if isinstance(value, list):
        return [normalize_result(v, replacements=replacements) for v in value]
    if isinstance(value, str):
        return normalize_text(value, replacements=replacements)
    return value


def normalize(value: Any, *, replacements: Mapping[str, str] | None = None) -> Any:
    """Dispatch: structural for dict/list, textual otherwise."""
    if isinstance(value, (dict, list)):
        return normalize_result(value, replacements=replacements)
    if isinstance(value, str):
        return normalize_text(value, replacements=replacements)
    return value

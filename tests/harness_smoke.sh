#!/usr/bin/env bash
# Live harness discovery smoke-test (plan-008 Issue 4.3) — LOCAL tier.
#
# For each harness present on PATH (`command -v`), installs the `naba` skill to that harness's
# idiomatic user-scope path and asserts the SKILL.md landed there. Self-SKIPS any harness that
# is not installed. CI does not have these harnesses, so this whole script skips there; the
# portable path-assertion tests (`cargo test resolve_dest_harness_paths`) are the CI baseline.
#
# The install + path assertion is automated and cheap (no provider call). The harness-specific
# *discovery confirmation* (does the running harness list the skill) may make a billable model
# call and is therefore printed for the operator to run, not executed here. See
# docs/plans/plan-008-james-dixson-24173a/references/harness-smoke-invocations.md.
#
# Usage:  NABA_BIN=/path/to/naba tests/harness_smoke.sh   (defaults to ./target/debug/naba)
set -uo pipefail

NABA_BIN="${NABA_BIN:-./target/debug/naba}"
if [ ! -x "$NABA_BIN" ]; then
  echo "naba binary not found at $NABA_BIN (set NABA_BIN or run: cargo build)" >&2
  exit 2
fi

# harness | probe-cmd | idiomatic user-scope SKILL.md (relative to $HOME)
HARNESSES=(
  "claude-code|claude|.claude/skills/naba/SKILL.md"
  "opencode|opencode|.config/opencode/skills/naba/SKILL.md"
  "pi|pi|.pi/agent/skills/naba/SKILL.md"
  "codex|codex|.agents/skills/naba/SKILL.md"
)

ran=0
skipped=0
failed=0

for row in "${HARNESSES[@]}"; do
  IFS='|' read -r harness probe rel <<<"$row"
  if ! command -v "$probe" >/dev/null 2>&1; then
    echo "SKIP  $harness (no '$probe' on PATH)"
    skipped=$((skipped + 1))
    continue
  fi
  echo "RUN   $harness: naba skills install --harness $harness --scope user"
  if ! "$NABA_BIN" skills install --harness "$harness" --scope user --quiet; then
    echo "FAIL  $harness: install returned non-zero"
    failed=$((failed + 1))
    continue
  fi
  target="$HOME/$rel"
  if [ -f "$target" ]; then
    echo "PASS  $harness: skill installed at $target"
    ran=$((ran + 1))
  else
    echo "FAIL  $harness: expected SKILL.md missing at $target"
    failed=$((failed + 1))
  fi
done

echo "----"
echo "installed+verified: $ran   skipped(absent): $skipped   failed: $failed"
echo "Discovery confirmation (operator-run, may be billable): see"
echo "  docs/plans/plan-008-james-dixson-24173a/references/harness-smoke-invocations.md"
[ "$failed" -eq 0 ]

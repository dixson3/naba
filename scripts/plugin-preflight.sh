#!/usr/bin/env bash
# naba plugin preflight — rule delivery + environment validation
# Runs on SessionStart. Always exits 0 (fail-open).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PREFLIGHT_JSON="$PLUGIN_ROOT/.claude-plugin/preflight.json"

# ---------- helpers ----------

log()  { echo "preflight: naba — $*"; }
warn() { echo "preflight: naba — WARNING: $*" >&2; }

# ---------- rule symlink delivery ----------

install_rules() {
  if [[ ! -f "$PREFLIGHT_JSON" ]]; then
    warn "preflight.json not found, skipping rule delivery"
    return
  fi

  if ! command -v jq >/dev/null 2>&1; then
    warn "jq not found, skipping rule delivery"
    return
  fi

  local installed=0 updated=0 removed=0

  local count
  count=$(jq -r '.artifacts.rules | length' "$PREFLIGHT_JSON" 2>/dev/null || echo 0)

  for ((i = 0; i < count; i++)); do
    local source target
    source=$(jq -r ".artifacts.rules[$i].source" "$PREFLIGHT_JSON")
    target=$(jq -r ".artifacts.rules[$i].target" "$PREFLIGHT_JSON")

    local abs_source="$PLUGIN_ROOT/$source"
    local abs_target="$PWD/$target"
    local target_dir
    target_dir="$(dirname "$abs_target")"

    mkdir -p "$target_dir"

    # Compute relative path from target dir to source
    local rel_source
    rel_source=$(python3 -c "import os.path; print(os.path.relpath('$abs_source', '$target_dir'))" 2>/dev/null || echo "$abs_source")

    if [[ -L "$abs_target" ]]; then
      local existing
      existing=$(readlink "$abs_target")
      if [[ "$existing" == "$rel_source" || "$existing" == "$abs_source" ]]; then
        continue
      fi
      rm "$abs_target"
      updated=$((updated + 1))
    elif [[ -e "$abs_target" ]]; then
      warn "$target exists and is not a symlink, skipping"
      continue
    else
      installed=$((installed + 1))
    fi

    ln -s "$rel_source" "$abs_target"
  done

  # Remove stale symlinks not in the current manifest
  local rules_dir="$PWD/.claude/rules/naba"
  if [[ -d "$rules_dir" ]]; then
    for link in "$rules_dir"/*; do
      [[ -L "$link" ]] || continue
      local link_name
      link_name="$(basename "$link")"

      local found=false
      for ((i = 0; i < count; i++)); do
        local target_name
        target_name=$(jq -r ".artifacts.rules[$i].target" "$PREFLIGHT_JSON")
        target_name="$(basename "$target_name")"
        if [[ "$link_name" == "$target_name" ]]; then
          found=true
          break
        fi
      done

      if [[ "$found" == "false" ]]; then
        rm "$link"
        removed=$((removed + 1))
      fi
    done
  fi

  log "installed:$installed updated:$updated removed:$removed"
}

# ---------- environment validation ----------

check_env() {
  if ! command -v naba >/dev/null 2>&1; then
    warn "naba not found on PATH. Install with: go install github.com/dixson3/naba/cmd/naba@latest"
  fi

  if [[ -z "${GEMINI_API_KEY:-}" ]]; then
    local config_file="${NABA_CONFIG_DIR:-$HOME/.config/naba}/config.yaml"
    if [[ ! -f "$config_file" ]] || ! grep -q 'api_key:' "$config_file" 2>/dev/null; then
      warn "GEMINI_API_KEY not set and no config found. Set with: export GEMINI_API_KEY=<key>"
    fi
  fi
}

# ---------- main ----------

install_rules
check_env

exit 0

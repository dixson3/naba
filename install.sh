#!/usr/bin/env bash
#
# Thin wrapper for the naba skills installer. Delegates to install.py via `uv run`,
# which handles the Python venv + PEP 723 inline dependencies. All arguments are
# passed through (e.g. --scope, --surface, --target, --dry-run, --uninstall).
#
# MIT License — Copyright (c) 2026 James Dixson / Yoshiko Studios LLC. See LICENSE.
#
set -euo pipefail

if ! command -v uv >/dev/null 2>&1; then
  echo "Error: 'uv' is required but not on PATH." >&2
  echo "Install uv: https://docs.astral.sh/uv/" >&2
  exit 1
fi

exec uv run "$(dirname "$0")/install.py" "$@"

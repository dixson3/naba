# naba — JSON Output Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §8 JSON output shapes (SPEC-JSON)

- **SPEC-JSON-001** [PINNED] `Result` object (2-space-indented):

  ```json
  {
    "path": "string",
    "command": "string",
    "prompt": "string",
    "elapsed_ms": 0,
    "params": { },
    "requested_format": "string",
    "actual_format": "string"
  }
  ```

  `params` is omitempty; `requested_format`/`actual_format` omitempty.
- **SPEC-JSON-002** [PINNED] Single-image commands emit a **single object** when there is
  one result, a **JSON array** when there is more than one.
- **SPEC-JSON-003** [PINNED] `story` **always** emits a JSON array, even for one frame.
- **SPEC-JSON-004** [PINNED] `doctor` JSON envelope: `{"ok": bool, "failed": int, "checks":
  [{"name","status","detail"}]}`.
- **SPEC-JSON-005** [PINNED] Nondeterministic fields the suite **normalizes** before
  comparison: `elapsed_ms`, timestamped auto-names/paths, version/commit/date. The parity
  harness has a normalizer (Issue 1.2) that canonicalizes these.
- **SPEC-JSON-006** [NEW] **Universal `--json` envelope contract** (Epic 2). Every subcommand
  emits a **documented** JSON structure under `--json` (including the SPEC-GLOBAL-003 piped
  auto-enable). The **discrete-result** commands — `version`, `config get`/`set`, `skills`
  (`install`/`upgrade`/`remove`/`status`), `provider`, `models` — use the **common envelope**
  `{ "status": <string>, "data": <payload>? , "error": <string>? }`: the success path emits
  `{ "status": "ok", "data": … }` (errors surface as an exit code + a stderr line, so `error` is
  reserved and normally omitted). The **image** commands keep their `Result` object/array shape
  (SPEC-JSON-001..003) and **`doctor`** keeps its `{ok, failed, checks}` envelope (SPEC-JSON-004)
  — these are the pre-existing documented JSON contracts the universal clause **grandfathers**,
  not rewrites. The Epic-1 provisional `config` envelope (`{status, key, value}`) is **normalized**
  into the common shape (`{status, data:{key, value}}`). A parity/traceability test enumerates
  every subcommand and asserts each emits its documented envelope so "universal" is enforced.
- **SPEC-JSON-007** [NEW — plan-008] **Multi-target `skills install`/`upgrade` JSON.** Under
  `--json`, a `skills install`/`upgrade` run that resolves to a **single** target emits the pinned
  **flat** per-target payload (the `{status:"ok", data:{…}}` envelope of SPEC-JSON-006, carrying the
  one target's action result). A run that resolves to **multiple** harnesses/targets (repeated
  `--harness`, or the unqualified receipt-driven `upgrade` over recorded targets — SPEC-INSTALL-002)
  instead emits a **`targets[]` array**, one flat per-target object per resolved target, in
  first-requested order after resolved-path dedupe (SPEC-HARNESS-005 / SPEC-INSTALL-002). The
  single-target shape is exactly one element's worth of the multi-target shape, so a consumer can
  branch on the presence of `targets`.

# Red-Team Review — pass-2

**Plan:** plan-003-james-dixson-94015b
**Date:** 2026-06-14
**Verdict:** REVISE

Second adversarial pass on plan v2. **All 7 pass-1 concerns (C1–C7) verified resolved
against the actual code.** Four new gaps (N1–N4) introduced by the revisions, all in the
docs/spec/manifest surface — mechanical, no design forks.

## Strengths (pass-1 resolutions verified)

- **C1 ✓** `resolveClient()` (`server.go:59`) builds from `cfg.Model` only; `generateAndReturn`(110)/`generateWithImageAndReturn`(136) take no imageConfig — 3.1's signature-refactor framing is correct.
- **C2 ✓** The three hardcoded `OutputPath(..., "image/png")` sites are real at `server.go:187`(generate)/`259`(icon)/`325`(story); the two shared helpers already pass `images[0].MIMEType` and are correctly excluded. "Three sites" is exact.
- **C3/1.4 ✓** `writer.go WriteImage:14-15` only derives extension from mimeType when `outputPath==""`; literal `-o foo.png` is honored — the mislabel is real, fix target correct.
- **C5 ✓** All six commands route through `Generate()`/`GenerateWithImage()` — imageConfig applies, no no-ops.
- **C6 ✓** `--model` is a persistent root flag (`root.go:48`); cobra `Changed()` is the right mechanism over the current `model==""` sentinel.
- **1.4 vs 3.1 ownership clean** — 3.1 defers the png fix to 1.4 explicitly.

## Concerns

- **N1 [medium] Issue 4.2 omits two specs that 1.4 drifts.** `docs/specifications/IG/output-handling.md` (MIME handling + JSON output structure) and `docs/specifications/IG/mcp-server.md:48-52` (MCP write flow) document exactly the behavior 1.4 changes, but 4.2 reconciles only configuration/EDD/image-generation. **Rec:** add `IG/output-handling.md` + `IG/mcp-server.md` to 4.2.
- **N2 [medium] 1.4's "surface requested-vs-actual in JSON" has un-scoped fan-out + no MCP carrier.** The JSON output is `output.Result` (`json.go:10-16`), populated per-command in all seven CLI handlers; the struct/`Params` change touches every call site, not just writer.go + 3 MCP sites. The MCP `imageResult()` (`server.go:78`) emits no JSON metadata at all — "surface in JSON (CLI and MCP)" (SC6) has no MCP carrier today. **Rec:** expand 1.4 to name the `output.Result`/`json.go` change + per-command population; explicitly decide the MCP carrier (e.g. corrected path + mime in the tool result text) or scope MCP to corrected-extension-only.
- **N3 [medium] 4.3's DRIFT-CHECK delta needs new nodes, not just edges.** In the current manifest only `README.md` is a node (`project-readme`); `IG/configuration.md` and `EDD/CORE.md` are **not nodes**. A model-id `value-equal` edge with `gemini-source` as a second `fixed` authority also needs a §7 conflict clause. **Rec:** rewrite 4.3 to enumerate 3 new nodes (`gemini-source`, `ig-configuration`, `edd-core`), the edges, §6 globs (`internal/gemini/client.go` + the two doc paths), and a §7 clause for the new fixed authority — then re-approve.
- **N4 [low] `icon` has its own extension path the 1.4 audit misses.** `icon.go:73-81` builds the path via `output.ExtForFormat(iconFormat)` (default png) outside `WriteImage`, so an icon JPEG also lands `.png` by a different path. Orthogonal to imageConfig (icon stays px-only) but the extension mislabel still applies. **Rec:** add `icon.go`/`ExtForFormat` to 1.4's audit surface, or state `icon --format` makes the extension the user's responsibility.

## Missing

- No issue covers the MCP JSON-metadata gap (N2): `imageResult` returns path + resource link, no format — SC6's "surface in JSON (CLI and MCP)" is unachievable on MCP without new work.
- 5.1's "extension reconciliation logic" test is contingent on the 1.4 struct/signature design (N2) being settled first (sequencing is fine; feasibility depends on N2).

## Gate Assessment

Sound and unchanged. Capability Gate now exercises `imageConfig` (pass-1 gap closed), blocks only 5.2. No over-gating, no new gate concerns.

## Upstream Assessment

Clean, unchanged. 0 open issues. Only upstream-adjacent action is the manifest re-approval, which per N3 is larger than 4.3 currently states.

## Operator Resolutions

| # | Concern | Severity | Resolution | Status |
|:-:|:--------|:---------|:-----------|:-------|
| N1 | 4.2 omits output-handling.md + mcp-server.md | medium | Issue 4.2 reconcile set expanded to include `IG/output-handling.md` and `IG/mcp-server.md`; Approach §5 lists all five specs. | resolved |
| N2 | 1.4 JSON fan-out (output.Result) + MCP carrier | medium | Issue 1.4 surface expanded: names `output.Result`/`json.go` + per-command population (`requested_format`/`actual_format`), and the MCP carrier (corrected path + format note in the MCP tool result text, since MCP doesn't emit the CLI JSON). SC6 updated. | resolved |
| N3 | 4.3 needs 3 nodes + globs + §7 clause, not just edges | medium | Issue 4.3 rewritten: 3 new nodes (`gemini-source` fixed, `ig-configuration`, `edd-core`), value-equal edges, §6 globs, §7 clause for the new fixed authority, `approved:no` + re-approval. | resolved |
| N4 | icon.go/ExtForFormat extension path | low | Issue 1.4 adds `internal/cli/icon.go` + `output.ExtForFormat` to the audit surface (extension-only correction; icon imageConfig stays out of scope). SC6 names icon. | resolved |

**Status:** resolved (all N1–N4 addressed in plan v3; awaiting operator approval)

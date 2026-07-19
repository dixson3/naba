# Exp 001 — MCP internals + `--json` audit

## MCP server (`src/mcp.rs`)

- **Crate:** `rmcp` v2.2.0 (`Cargo.toml:66`), features `server`, `transport-io`. Not hand-rolled.
  `rmcp::serve_server(NabaMcpServer, stdio())` (`src/mcp.rs:60`); server = `impl ServerHandler`.
- **Capabilities:** `get_info()` builds `ServerCapabilities::builder().enable_tools().enable_resources().build()`
  (`src/mcp.rs:76-84`) — advertises **tools + resources**, NOT prompts.
- **Tools (8):** hand-built `rmcp::model::Tool` values from `tools()` (`src/mcp.rs:608-619`), served
  via `list_tools()`. Dispatch is a hand-written `match name { ... }` in `call_tool()`
  (`src/mcp.rs:135-150`): generate_image, edit_image, restore_image, generate_icon,
  generate_pattern, generate_story, generate_diagram, list_images. Tool errors → `CallToolResult::error`,
  never a process exit.
- **Existing resources:** `list_resource_templates()` returns a `file:///{path}` template and
  `read_resource()` reads that path as base64 blob (`src/mcp.rs:94-126`) — for generated image FILES only.
  `resources/list` (concrete) and `prompts` are NOT implemented.

## Lazy-loading surgery = LOW

- rmcp routes JSON-RPC methods to trait-method overrides — no central dispatch to edit (unlike the
  `call_tool` match). Adding **`resources/list`** = one new `list_resources` override; extending
  **`resources/read`** to resolve a `skill://naba/<rel>` scheme = a few lines. `resources` capability is
  already enabled, so **no handshake change** needed for a resources-based lazy-load.
- `prompts/list`+`prompts/get` would additionally need `.enable_prompts()` + two overrides.
- **Decision input:** resources path is the lightest — matches the scoping choice (resources + on-demand detail).

## Embedded skills (`src/embed.rs`, `skills/naba/`)

- `include_dir!` embeds the whole `skills/` tree (`src/embed.rs:34`). Accessors already exist and are
  exactly what a resource handler needs: `skill_names()`, `skill_files(name)` (sorted rel paths),
  `read_skill_file(name, rel) -> Option<&'static [u8]>` (`src/embed.rs:41,62,83`).
- Files: `SKILL.md`, `README.md`, `commands/*.md` (10: batch, brand-kit, diagram, edit, generate, icon,
  pattern, restore, story, storyboard — note batch/brand-kit/storyboard are skill guidance, NOT CLI verbs).
- **MCP can surface these** cleanly: `resources/list` enumerates `skill_files("naba")`; `resources/read`
  returns `read_skill_file(...)` as text/markdown. List is cheap (paths), read fetches on demand = lazy.

## `--json` audit

- **Global auto-enable (SPEC-GLOBAL-003):** `src/main.rs:55-58` — `json = cli.json || !stdout().is_terminal()`;
  carried in `Globals { json }` (`src/commands.rs:25-34`), threaded to handlers.
- **Image envelope:** `src/output.rs` `Result`/`to_json`/`print_json*` (`output.rs:87-114`); doctor has
  `DoctorEnvelope` (`output.rs:154-174`).

| Subcommand | `--json` today | Where |
|:--|:--|:--|
| generate/edit/restore/icon/pattern/diagram/story | yes | `commands.rs` `print_results_json`/`print_json_multi` |
| doctor | yes | `doctor.rs:265-270` → `DoctorEnvelope` |
| skills preflight | yes | `preflight.rs:196-221` |
| self install/update/uninstall | yes | `self_cmd/*.rs` |
| **config get** | **NO** | plain `println!` `commands.rs:55-58` |
| **config set** | **NO** | plain `println!` `commands.rs:63-68` |
| **skills install/upgrade/remove/status** | **NO** | plain `println!` `skills.rs` (0 json refs) |
| **version** | **NO** | plain `println!` `commands.rs:38-39` |
| mcp | N/A | starts stdio server |

- **No `provider`/`models` subcommands exist** — provider/model selection is via global `--provider`/`--model`
  flags (`cli.rs:31-42`). The 12 command groups are in `cli.rs:65-105`. So `naba provider`/`naba models` are
  net-new subcommands.
- **`--json` gaps to close:** config get/set, all skills verbs, version. (These ignore `globals.json` and
  print human text even when piped — not machine-readable today.)

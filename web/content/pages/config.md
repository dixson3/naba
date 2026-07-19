Title: config
Slug: config
Subtitle: providers, keys, models, and defaults

## Set an API key

Set the key for whichever provider(s) you use:

```bash
export GEMINI_API_KEY=<your-key>        # Google Gemini
export OPENROUTER_API_KEY=<your-key>    # OpenRouter
```

You can save the **Gemini** key to config (there is no config key for the OpenRouter key —
it stays env-only):

```bash
naba config set api_key <your-key>
```

For Gemini, the key precedence is `GEMINI_API_KEY` env > config `api_key`.

## Providers

naba routes every image command through one of two providers:

| Provider | API key | Default model |
|:---------|:--------|:--------------|
| `gemini` | `GEMINI_API_KEY` | `gemini-3.1-flash-image` |
| `openrouter` | `OPENROUTER_API_KEY` | `google/gemini-3.1-flash-image-preview` |

Select the provider with the global `--provider` flag or the `provider` config key:

```bash
naba generate "a red apple" --provider gemini
naba config set provider gemini      # pin a default provider
```

### Resolution precedence

When you don't pass `--provider`, naba resolves the provider in this order:

1. **CLI** `--provider`
2. **Config** `provider` key
3. **Env-key autodetect** — only `GEMINI_API_KEY` → gemini; only `OPENROUTER_API_KEY` → openrouter
4. **Built-in fallback** — gemini

> **Multi-key reroute.** If **both** keys are set and no `provider` is configured, autodetect
> resolves to **OpenRouter**. To stay on Gemini when both keys are set, pin it in config —
> config beats autodetect:
>
> ```bash
> naba config set provider gemini
> ```

### `--model` requires `--provider`

A bare `--model` on the CLI is ambiguous across providers, so `--model` **without**
`--provider` is a usage error (exit 2). Always pair them. (A config `model` without a config
`provider` is fine — it is scoped by whatever provider resolves.)

## Models & quality

**Gemini** defaults to `gemini-3.1-flash-image` (Nano Banana 2). A higher-quality tier,
`gemini-3-pro-image` (Nano Banana Pro), is available for final/hero assets.

> **All Gemini image models require a paid (billing-enabled) tier — none work on the free
> tier.** Pro costs roughly 2–3.5× flash per image, so flash is the default.

```bash
naba generate "hero banner" --quality fast    # fast -> gemini-3.1-flash-image (default)
naba generate "hero banner" --quality high    # high -> gemini-3-pro-image
naba generate "hero banner" --provider gemini --model gemini-3-pro-image   # raw id, highest precedence
```

For Gemini, model precedence is `--model` > `--quality` > config `model` > config `quality`
> built-in default. `--quality` means different things per provider: on Gemini it swaps the
model tier; on OpenRouter it is passed through as the native `quality` request parameter and
does **not** swap the model.

## Config keys

```bash
naba config set provider gemini              # default provider (gemini or openrouter)
naba config set model gemini-3-pro-image     # or: naba config set quality high
naba config set aspect 16:9                  # default imageConfig aspect ratio
naba config set resolution 2K                # default imageConfig resolution
naba config get model
```

Recognized keys: `api_key`, `model`, `provider`, `default_output_dir`, `aspect`,
`resolution`, `quality`. Per-call flags override config; within config, `model` beats
`quality`, and `provider` beats env-key autodetect.

## MCP server

`naba mcp` starts a stdio [Model Context Protocol](https://modelcontextprotocol.io) server
exposing the image tools to assistants like Claude Desktop. Set `NABA_OUTPUT_DIR` to tell
naba where to write images (otherwise `~/.local/share/naba/images`).

```json
{
  "mcpServers": {
    "naba": {
      "command": "naba",
      "args": ["mcp"],
      "env": { "GEMINI_API_KEY": "<your-key>", "NABA_OUTPUT_DIR": "/path/to/output" }
    }
  }
}
```

## Self-update

`naba self` manages the binary itself. It only auto-updates a **vendor** install (the
`curl | sh` bootstrap from the [install](/install/) page); Homebrew installs are refused with
a pointer to `brew upgrade naba`, and a from-build/unknown install needs `--force`.

```bash
naba self update            # fetch the latest release, verify sha256, swap in place
naba self update --check    # report whether an update is available; change nothing
naba self update --json     # machine-readable envelope (status/source/current/latest)
naba self install --from-build   # record the running build as a from-build install
naba self uninstall              # remove the from-build marker (--force also deletes the binary)
```

A successful `self update` also refreshes the installed Claude Code skills
(`naba skills upgrade`) unless you pass `--binary-only`. GitHub Releases is canonical for
every binary and for the self-update manifest — the website hosts no binaries.

## Claude Code skills

The `/naba` skill tree is embedded in the binary, so `naba skills` works offline and always
matches the binary's version. The [install](/install/#claude-code-skills) page covers the
common `install` / `upgrade` lifecycle; the full option surface:

```bash
naba skills install                  # default: user scope -> ~/.claude/skills
naba skills install --dry-run        # show what would be written, change nothing
naba skills install --scope project  # install into <git-root>/.claude/skills
naba skills install --surface agents # install into ~/.agents/skills (agents surface)
naba skills install --target DIR     # install into an explicit directory
naba skills upgrade                  # rewrite from the embedded tree, pruning stale files
naba skills remove                   # remove the naba skill again
naba skills status                   # report up-to-date / complete / unmodified
```

On `install`/`upgrade` naba writes a hidden integrity marker into the deployed `SKILL.md`
(`<!-- naba-skills: v=<version> tree=<sha256> -->`); `status` and `naba doctor` use it to
confirm the install is current, complete, and unmodified.

## Health check (`naba doctor`)

`naba doctor` validates your environment and exits non-zero if any check fails:

```bash
naba doctor                  # checks skills install, API key, model, config
naba doctor --json           # structured output
naba doctor --surface agents # check the agents-surface install instead
```

It reports: skills installed and matching this binary; the **effective provider's** API key
present and live-valid (a cheap `models.list` call, no image cost); the configured model
reachable; config parseable; and the binary version.

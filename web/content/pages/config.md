Title: config
Slug: config
Subtitle: providers, keys, models, and defaults

naba's configuration is **nested and per-provider**. A single `config.yaml` holds a default
provider, a per-provider block (default model + api-key source) for each provider you use, and
the top-level image defaults. Every key is optional — an absent key resolves to a built-in
default on read.

## Set an API key

The fastest path is an environment variable — naba reads each provider's conventional key
without any config at all:

```bash
export GEMINI_API_KEY=<your-key>              # Google Gemini
export OPENROUTER_API_KEY=<your-key>          # OpenRouter
export AWS_BEARER_TOKEN_BEDROCK=<your-token>  # AWS Bedrock (api-key bearer path)
```

You can also save a key into config per provider (it is stored inline in `config.yaml`):

```bash
naba config set gemini.api-key <your-key>
naba config set openrouter.api-key <your-key>
naba config set bedrock.api-key <your-token>
```

### API-key resolution precedence

For **every** provider, naba resolves the api-key with one uniform precedence — highest first:

1. **Inline** `providers.<provider>.api-key` (from `config set <provider>.api-key`)
2. **Custom env var** named by `providers.<provider>.api-key-envvar`
3. **Conventional env var** — the provider's default (`GEMINI_API_KEY`,
   `OPENROUTER_API_KEY`, `AWS_BEARER_TOKEN_BEDROCK`)

The `api-key-envvar` indirection lets you point a provider at a differently-named secret without
committing the value:

```bash
naba config set gemini.api-key-envvar MY_TEAM_GEMINI_KEY
```

## Providers

naba routes every image command through one of its registered providers:

| Provider | Conventional key env var | Default model |
|:---------|:-------------------------|:--------------|
| `gemini` | `GEMINI_API_KEY` | `gemini-3.1-flash-image` |
| `openrouter` | `OPENROUTER_API_KEY` | `google/gemini-3.1-flash-image-preview` |
| `bedrock` | `AWS_BEARER_TOKEN_BEDROCK` | `amazon.nova-canvas-v1:0` |

List them — and which have resolvable credentials — with
[`naba provider`](/usage/#provider); list a provider's models with
[`naba models`](/usage/#models):

```bash
naba provider                        # * marks the effective default; shows credential status
naba models --provider bedrock       # a provider's model set (needs a resolvable key)
```

Select the provider with the global `--provider` flag or the `default-provider` config key:

```bash
naba generate "a red apple" --provider gemini
naba config set default-provider gemini      # pin a default provider
```

### Per-provider default model

Each provider carries its **own** default model. When `providers.<provider>.model` is unset,
naba falls back to that provider's compiled-in default (the table above). So switching providers
never leaves you model-less, and you can pin a different model per provider:

```bash
naba config set gemini.model gemini-3-pro-image
naba config set openrouter.model bytedance-seed/seedream-4.5
```

### Provider resolution precedence

When you don't pass `--provider`, naba resolves the provider in this order:

1. **CLI** `--provider`
2. **Config** `default-provider` key
3. **Env-key autodetect** — the provider **latest** in the registry order
   (`gemini` → `openrouter` → `bedrock`) whose credentials resolve
4. **Built-in fallback** — `gemini`

> **Multi-key reroute.** Autodetect scans the registry order and the **latest** provider with
> credentials wins. So if both `GEMINI_API_KEY` and `OPENROUTER_API_KEY` are set and no
> `default-provider` is configured, it resolves to **OpenRouter**; add Bedrock credentials and
> it reroutes to **Bedrock**. To stay put, pin the provider — config beats autodetect:
>
> ```bash
> naba config set default-provider gemini
> ```

### `--model` requires `--provider`

A bare `--model` on the CLI is ambiguous across providers, so `--model` **without**
`--provider` is a usage error (exit 2). Always pair them. (A config `<provider>.model` is fine
on its own — it is scoped to that provider's block.)

## AWS Bedrock

Bedrock is a first-class provider. `naba models --provider bedrock` lists the curated set —
Amazon and Stability image families:

| Model id | Family |
|:---------|:-------|
| `amazon.nova-canvas-v1:0` | Amazon (Nova Canvas, default) |
| `amazon.titan-image-generator-v1` | Amazon (Titan Image v1) |
| `amazon.titan-image-generator-v2:0` | Amazon (Titan Image v2) |
| `stability.stable-image-core-v1:0` | Stability (Stable Image Core) |
| `stability.stable-image-ultra-v1:1` | Stability (Stable Image Ultra) |
| `stability.sd3-5-large-v1:0` | Stability (SD 3.5 Large) |

### Bedrock auth (two modes)

Bedrock supports **both** credential paths; naba prefers the bearer token when one resolves,
otherwise it falls back to AWS SigV4 signing:

| Mode | How to supply | Notes |
|:-----|:--------------|:------|
| **api-key bearer** | `AWS_BEARER_TOKEN_BEDROCK`, or `bedrock.api-key` / `bedrock.api-key-envvar` | Sent as `Authorization: Bearer <token>`; resolves through the uniform api-key precedence above. |
| **AWS profile / SigV4** | `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` (+ optional `AWS_SESSION_TOKEN`), or a named `AWS_PROFILE` in `~/.aws/credentials` | Requests are signed with SigV4. SSO-token / IMDS resolution is not implemented. |

### Bedrock region

The region defaults to **`us-east-1`** (broadest image-model coverage) and is read from
`AWS_REGION` > `AWS_DEFAULT_REGION` > the default:

```bash
export AWS_REGION=us-west-2
naba generate "a red apple" --provider bedrock
```

## Config file

The config lives at `$NABA_CONFIG_DIR/config.yaml` (default `~/.config/naba/config.yaml`;
`$XDG_CONFIG_HOME/naba/config.yaml` when `XDG_CONFIG_HOME` is set). A complete example:

```yaml
default_provider: gemini
providers:
  gemini:
    model: gemini-3-pro-image
    api-key: <inline-key>
  openrouter:
    model: bytedance-seed/seedream-4.5
    api-key-envvar: MY_OR_KEY
  bedrock:
    model: amazon.nova-canvas-v1:0
default_output_dir: /path/to/output
aspect: "16:9"
resolution: 2K
quality: high
```

### Config keys

Address keys through `naba config set <key> <value>` / `naba config get <key>`:

```bash
naba config set default-provider gemini          # default provider (gemini, openrouter, bedrock)
naba config set gemini.model gemini-3-pro-image   # a provider's default model
naba config set gemini.api-key <key>              # a provider's inline api-key
naba config set gemini.api-key-envvar MY_KEY      # a provider's custom key env var
naba config set aspect 16:9                       # default imageConfig aspect ratio
naba config set resolution 2K                     # default imageConfig resolution
naba config set quality high                      # default quality tier
naba config get gemini.model
naba config get gemini.model --json               # machine-readable envelope
```

The full key set:

| Key | Meaning |
|:----|:--------|
| `default-provider` | The provider used when no `--provider` is passed |
| `<provider>.model` | That provider's default model (`gemini`, `openrouter`, `bedrock`) |
| `<provider>.api-key` | That provider's inline api-key |
| `<provider>.api-key-envvar` | Name of a custom env var holding that provider's api-key |
| `default_output_dir` | MCP output directory (CLI uses `-o`/CWD instead) |
| `aspect` | Default imageConfig aspect ratio |
| `resolution` | Default imageConfig resolution |
| `quality` | Default quality tier (`fast`/`high`) |

Per-call flags override config; within config, a provider's `model` beats `quality`, and
`default-provider` beats env-key autodetect. The legacy flat keys `api_key` (→ `gemini.api-key`),
`model` (→ the default provider's model), and `provider` (→ `default-provider`) are still
accepted as aliases for backward compatibility.

### Quality semantics differ per provider

`--quality fast`/`high` is the cross-provider vocabulary, but each provider interprets it:

- **Gemini** — `--quality` selects a **model tier**: `fast` → `gemini-3.1-flash-image`,
  `high` → `gemini-3-pro-image`. An explicit `--model` overrides it.
- **OpenRouter / Bedrock** — `--quality` is passed through as a **native request parameter** and
  does **not** swap the model.

> **All Gemini image models require a paid (billing-enabled) tier — none work on the free
> tier.** Pro costs roughly 2–3.5× flash per image, so flash is the default.

### Config auto-migration

An old **flat** config (top-level `api_key` / `model` / `provider`) is auto-migrated to the
nested schema on first load. naba writes a `config.yaml.bak` backup with the original bytes
first, then rewrites the document: `api_key` → `providers.gemini.api-key` (its historical
Gemini scope), `model` → the resolved default provider's block, `provider` →
`default_provider`. The migration is idempotent and graceful on empty/missing/malformed inputs;
a structural rewrite drops YAML comments (mitigated by the `.bak` backup).

## MCP server

`naba mcp` starts a stdio [Model Context Protocol](https://modelcontextprotocol.io) server that
exposes the image tools to assistants like Claude Desktop. See the [MCP page](/mcp/) for the
full tool list, the lazy-loading `skill://` resources, Claude Desktop config, and
`NABA_OUTPUT_DIR`.

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
matches the binary's version. See the [Skills page](/skills/) for the full subcommand set,
implicit triggering, the claude vs agents surface, user vs project scope, and the whole
install / upgrade / status / remove / preflight lifecycle. The short form:

```bash
naba skills install                  # default: user scope -> ~/.claude/skills
naba skills upgrade                  # rewrite from the embedded tree, pruning stale files
naba skills status                   # report up-to-date / complete / unmodified
naba skills status --json            # machine-readable envelope
```

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

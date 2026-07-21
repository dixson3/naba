Title: install
Slug: install
Subtitle: get naba on your machine

naba is a single self-contained Rust binary — no runtime, no dependencies. The **vendor
`curl | sh` installer** below is the recommended way in; other paths follow.

## Vendor install (curl | sh) — recommended

This runs the vendor installer (a mirror of cargo-dist's `naba-installer.sh`) via a short
bootstrap URL, drops the binary in `~/.local/bin`, and records an install receipt so naba can
update itself later with [`naba self update`](/config/#self-update):

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh | sh
```

> The `naba.ysapp.net/install.sh` bootstrap is a byte-for-byte mirror of the cargo-dist
> installer published on GitHub Releases — it in turn fetches sha256-checksummed release
> tarballs from GitHub. **GitHub Releases** remains canonical for every binary; the
> `naba.ysapp.net` domain hosts only the convenience `install.sh`.

## Alternatives

### Homebrew (macOS and Linux)

If you already live in Homebrew, the tap works too — upgrades come through `brew upgrade`
(a Homebrew install does **not** self-update via `naba self update`):

```bash
brew install dixson3/tap/naba
brew upgrade naba          # update later
```

### Cargo

```bash
cargo install --git https://github.com/dixson3/naba
```

### Build from source

```bash
git clone https://github.com/dixson3/naba.git
cd naba
cargo build --release      # binary at target/release/naba

# Optionally register this build as a from-build install on your PATH (~/.local/bin):
./target/release/naba self install --from-build
```

### Which one should I use?

| Path | Self-updates in place | Best for |
|:-----|:----------------------|:---------|
| **Vendor install `curl \| sh`** | **yes** (`naba self update`) | **most people — fast install + in-place updates** |
| Homebrew | no (use `brew upgrade`) | macOS/Linux users already on Homebrew |
| Cargo | no | Rust developers |
| From source | with `self install --from-build` | hacking on naba |

## Agent harness skills

The steps above give you the `naba` command-line tool. naba can *also* install itself as a
**skill** inside an AI coding agent — so instead of remembering CLI flags, you (or the agent)
can just say *"make me an app icon of a rocket ship"* and the right `naba` command runs for you.
naba ships **one** skill that wraps the whole CLI as a single slash command with subcommands:
`/naba generate`, `/naba edit`, `/naba icon`, and so on.

That skill is not tied to any one agent. It installs into whichever **agent harness** you use —
[Claude Code](https://claude.com/claude-code) is the default and the most common example, but
opencode, pi, codex, and a portable `agents` layout are supported too, each at its own idiomatic
path. The [skills page](/skills/#harnesses-one-tree-five-idiomatic-homes) lists every harness and
where the files land.

The skill files are **embedded in the `naba` binary** at compile time. That means installing
them is fully offline and always matches the binary you already have — there is no marketplace
plugin, no separate download, and no way for the skill and the CLI to drift to different versions:

```bash
naba skills install          # install the /naba skill into the default harness (Claude Code, ~/.claude/skills)
```

### Keep the skill in sync with the binary

Because the skill is a snapshot of the binary at install time, you refresh it whenever the binary
changes:

```bash
naba skills upgrade          # rewrite the skill from the current binary, pruning any stale files
```

You usually don't have to run this by hand. A normal [`naba self update`](/config/#self-update)
upgrades the binary **and** refreshes every skill you've installed in the same step — unless you
opt out with `--binary-only`. Reach for `naba skills upgrade` explicitly after a Homebrew or
`cargo` upgrade (those bump the binary but don't touch skills), or any time `naba doctor` reports
the installed skill is out of date.

### Before it can run

The installed skill is a thin wrapper: when triggered, it **shells out to the `naba` CLI**. So two
things have to be in place for it to actually produce an image:

1. **The `naba` binary is installed and on your `PATH`** — the steps at the top of this page.
2. **A provider API key is set** — `GEMINI_API_KEY`, `OPENROUTER_API_KEY`, or
   `AWS_BEARER_TOKEN_BEDROCK`. See [config](/config/) for how naba resolves keys.

`naba skills install` will happily write the files before either is true; the skill just stays
inert until the binary is reachable and a key is present.

### Where to go next

- **[skills](/skills/)** — the full lifecycle: every subcommand, how the agent triggers naba
  automatically from plain-language requests, and the detailed flags (`--scope` for user vs.
  project installs, the repeatable `--harness` to install into several agents at once, `--target`
  for an explicit directory, plus `status`, `remove`, and the fast `preflight` gate).
- **[config](/config/#health-check-naba-doctor)** — `naba doctor`, the health check that verifies
  your binary, API key, and installed skill are all consistent.
- **[usage](/usage/)** — set a key and run your first command, from the CLI or through the skill.

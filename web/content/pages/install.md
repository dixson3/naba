Title: install
Slug: install
Subtitle: get naba on your machine

naba is a single self-contained Rust binary — no runtime, no dependencies. Pick the install
path that fits you.

## Bootstrap (curl | sh)

The short, memorable one-liner. It fetches the vendor installer (a mirror of cargo-dist's
`naba-installer.sh`), drops the binary in `~/.local/bin`, and records an install receipt so
naba can update itself later with [`naba self update`](/usage/#self-update).

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh | sh
```

> This bootstrap script is a byte-for-byte mirror of the cargo-dist installer published on
> GitHub Releases — it in turn fetches sha256-checksummed release tarballs from GitHub.
> **GitHub Releases remains canonical for every binary; this domain hosts only the
> convenience `install.sh`.** Until the first tagged release is cut, the script prints a
> "no release yet" notice and exits non-zero so a premature run fails safely — browse
> [github.com/dixson3/naba/releases](https://github.com/dixson3/naba/releases) for status.

## Homebrew (macOS and Linux) — recommended

The Homebrew tap is the recommended default. Upgrades come through `brew upgrade`:

```bash
brew install dixson3/tap/naba
brew upgrade naba          # update later
```

## Cargo

```bash
cargo install --git https://github.com/dixson3/naba
```

## Build from source

```bash
git clone https://github.com/dixson3/naba.git
cd naba
cargo build --release      # binary at target/release/naba

# Optionally register this build as a from-build install on your PATH (~/.local/bin):
./target/release/naba self install --from-build
```

## Which one should I use?

| Path | Self-updates in place | Best for |
|:-----|:----------------------|:---------|
| Bootstrap `curl \| sh` | yes (`naba self update`) | a fast first install with in-place updates |
| Homebrew | no (use `brew upgrade`) | macOS/Linux users already on Homebrew |
| Cargo | no | Rust developers |
| From source | with `self install --from-build` | hacking on naba |

Once installed, set an API key and try your first command — see [usage](/usage/) and
[config](/config/).

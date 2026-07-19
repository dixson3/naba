# naba website (`web/`)

The source for **https://naba.ysapp.net** — a bespoke [Pelican](https://getpelican.com)
static site with a dark terminal/technical theme, published to AWS (private S3 + CloudFront +
ACM + Route53).

```text
web/
├── pelicanconf.py          # dev config (pretty directory-style URLs, theme, menu)
├── publishconf.py          # prod overrides (SITEURL=https://naba.ysapp.net)
├── requirements.txt        # pinned Pelican + sitemap + markdown + PyYAML
├── Makefile                # devserver / validate / publish / deploy / provision
├── content/
│   ├── pages/              # install, usage, config, 404 (home is the index template)
│   ├── images/samples/     # dogfooded naba output shown on the usage page
│   └── extra/              # robots.txt, staged install.sh (site-root files)
├── themes/naba-terminal/   # bespoke theme: templates + one hand-authored CSS file
├── scripts/
│   ├── sync_installer.sh   # stage the hosted install.sh from a cargo-dist release
│   ├── provision_aws.sh    # idempotent AWS provisioning (gated, billable)
│   └── aws/index-rewrite.js# CloudFront viewer-request Function (pretty-URL rewrite)
└── docs/provisioning-runbook.md
```

## Develop

```bash
cd web
python3 -m venv .venv && . .venv/bin/activate
pip install -r requirements.txt

make devserver        # serve + live-reload at http://localhost:8000
```

Edit `content/` and `themes/naba-terminal/`; the dev server regenerates on change.

## Validate

```bash
make validate         # production build into a throwaway dir; non-zero on any build error
```

This is the look-approved gate's build check. It does not touch the dev `output/` tree.

## Provision (gated, billable)

Creates the live AWS stack (private S3 bucket, CloudFront OAC + distribution, ACM cert in
us-east-1, CloudFront Function, Route53 alias). **Billable** — run only after the go-live
gate. Idempotent (check-before-create); safe to re-run.

```bash
make provision        # == ./scripts/provision_aws.sh
```

Captured resource ids are written to `aws-config.mk` (feeds `CF_DISTRIBUTION`) and
`.aws-provision-state.json` — both gitignored. Full steps, verification, and the
**teardown/rollback** inverse are in [`docs/provisioning-runbook.md`](docs/provisioning-runbook.md).

## Deploy

```bash
make deploy           # s3_upload (site) + sync_installer (install.sh, short TTL) + invalidate
```

- `make s3_upload` — `aws s3 sync --delete` the built site, then invalidate `/*`.
- `make sync_installer` — stage `install.sh` (via `sync_installer.sh`) and re-upload **that
  one key** with `Cache-Control: max-age=300` (the tree-wide sync does not set per-key cache
  headers), then invalidate `/install.sh`.
- `make invalidate` — invalidate the mutable paths on demand.

## Analytics (GA4)

Google Analytics is rendered **production-only** (via `publishconf.py`), so local
`make devserver` and PR builds never load it. The GA4 **measurement id** is account-specific
and is **never committed** — it is read from the environment:

```bash
export NABA_GA_MEASUREMENT_ID=G-XXXXXXXXXX   # local: repo .envrc (gitignored)
```

The canonical value lives as the GitHub repo secret `NABA_GA_MEASUREMENT_ID`. When the env var
is unset, the theme renders **no** gtag snippet (fail-safe). The GA stream id / property id are
GA-side account metadata and are **not** needed by the page tag.

## The GH-canonical vs site-bootstrap boundary

**GitHub Releases is canonical for every naba binary and for self-update.** This site hosts
exactly **one** convenience artifact: `install.sh`, a byte-for-byte mirror of cargo-dist's
`naba-installer.sh` from a pinned release tag. It exists so users get a short bootstrap:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh | sh
```

The installer itself pulls sha256-checksummed tarballs from GitHub Releases, and thereafter
`naba self update` runs off GitHub. The site never mirrors binaries and hosts no
`latest.json` — it is an additive, human-facing surface only.

### Before the first release (issue #7)

Until the first cargo-dist release is cut, `sync_installer.sh` stages a **fail-safe
placeholder** `install.sh` that prints a "no release yet" message and exits non-zero, so a
premature `curl | sh` fails safely. Once the first `v<semver>` tag lands, re-run
`make deploy` to activate the real installer (see the plan's Issue 4.3 follow-on).

## How pretty URLs resolve on a private bucket

The site uses directory-style URLs (`/install/`). A private bucket read via CloudFront OAC is
not an S3 *website* endpoint, so it does not append `index.html` to directory requests. The
CloudFront viewer-request Function (`scripts/aws/index-rewrite.js`) rewrites slash-terminated
and extension-less paths to `/index.html`; `/install.sh` passes through. See the runbook.

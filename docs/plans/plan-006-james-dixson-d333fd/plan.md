# Plan: Build a Pelican static website for naba (dark terminal theme) published to naba.ysapp.net via AWS S3+CloudFront+ACM+Route53, with a hosted install.sh bootstrap (GitHub Releases stays canonical for binaries + self-update), iterated locally first

**ID:** plan-006-james-dixson-d333fd
**Author:** james-dixson
**Created:** 2026-07-18
**Status:** approved
**Fingerprint:** 64beb8c3b4439c3ee6449666f3630c27706697ae5140e1cc70eeda79c39cb445
**Phase log:**
- 2026-07-18 scoping: initial scope captured
- 2026-07-18 drafting: plan v1 presented
- 2026-07-18 drafting: plan v1 drafted
- 2026-07-18 review: plan v1 presented for review
- 2026-07-18 review: red-team pass 2: APPROVE (pass-1 REVISE concerns resolved)
- 2026-07-18 ready-for-approval: ready-check green — last red-team APPROVE + audit pass
- 2026-07-18 approved: operator approved

## Objective
Build a bespoke Pelican static website for the naba CLI, styled with a dark
terminal/technical aesthetic, and publish it at **https://naba.ysapp.net** using AWS
(S3 + CloudFront + ACM + Route53). The site provides:

1. Product/docs content: name, description, example usage with sample output images,
   configuration guide, install/setup guide, and a link back to the GitHub project.
2. A **bootstrap install entrypoint** — `install.sh` at the site root (a mirror of
   cargo-dist's `naba-installer.sh`), so users can run
   `curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh | sh` to get
   naba installed the first time. The installer pulls the binary from GitHub Releases;
   thereafter `naba self update` takes over.

**Explicitly out of scope** (decided during scoping): no `/downloads/` binary mirror and no
`latest.json` on the domain. GitHub release-asset downloads are CDN-served and effectively
unlimited (separate from the GitHub API rate limit), so mirroring binaries to S3 buys little.
The site's install page links to GitHub Releases for version browsing.

The workflow iterates locally first (`pelican devserver`) so the look can be reviewed and
refined before anything is pushed publicly; the live AWS provisioning is gated.

## Motivation
naba is a standalone Rust CLI for AI image generation. Today its only public presence is
the GitHub repo README, and its only install entrypoints are a long GitHub Releases URL, a
Homebrew tap, and `cargo install`. There is no branded landing page, no memorable install
command, and no human-friendly place to see what the tool does (ironic for an
image-generation tool with no visual showcase). A dedicated site at `naba.ysapp.net`
gives naba: a short, memorable bootstrap install (`curl naba.ysapp.net/install.sh | sh`);
a place to show example commands with their generated-image outputs; and canonical,
readable install/usage/config docs decoupled from the repo README. The operator owns
`ysapp.net` (Route53) and an AWS account, so the marginal cost is small and the pattern is
already proven by the sibling `thesoftwarefactory` Pelican+S3+CloudFront site.

This deliberately does NOT change naba's self-update mechanism, which is correctly canonical
on GitHub Releases via cargo-dist `dist-manifest.json`. The site is an additive,
human-facing surface; GitHub remains the source of truth for all binaries.

## Upstream Issues
| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| #7 | Cut the first cargo-dist release of naba (activates self-update) | partial (related, not resolved) | Soft dependency: the hosted `install.sh` is mirrored FROM a cargo-dist release and only carries real bytes after #7. This plan builds tooling that tolerates "no release yet"; it does NOT cut the release. #7 remains open. | — (not resolved here) |
| #5 | Retire Go source once Rust parity is trusted | exclude | Unrelated to the website. | — |

## Investigation Findings
See `findings/exp-001-landscape.md` for full detail. Summary:

- **AWS/DNS ready.** Account `REDACTED-ACCOUNT-ID` (user `dixson3`) credentials work; `ysapp.net`
  is an existing Route53 public hosted zone (the Route53 zone id (redacted — stored in local .envrc + GitHub repo secret `NABA_HOSTED_ZONE_ID`, not in the repo)). `naba.ysapp.net` is
  a single record upsert.
- **Proven pattern.** The sibling `~/workspace/ys/thesoftwarefactory` site is Pelican +
  `Makefile` (`devserver`, `publish`, `s3_upload` = `aws s3 sync --delete` + CloudFront
  invalidation) + `publishconf.py` (prod `SITEURL`). Mirror this structure.
- **Self-update is GitHub-canonical and untouched.** `src/self_cmd/update.rs` hardcodes the
  manifest URL to `{CARGO_PKG_REPOSITORY}/releases/latest/download/dist-manifest.json`. The
  site does not touch this — **zero Rust changes**.
- **GitHub downloads are not a bottleneck.** Release assets (incl. `dist-manifest.json`) are
  served from GitHub's asset CDN, distinct from the API rate limit — so a domain-hosted
  binary mirror is unnecessary. Only the friendly `install.sh` bootstrap is worth hosting.
- **cargo-dist naming.** Targets: `{aarch64,x86_64}-apple-darwin`,
  `{aarch64,x86_64}-unknown-linux-gnu`. Assets: `naba-<triple>.tar.gz` (+ `.sha256`),
  `dist-manifest.json`, `naba-installer.sh`. `checksum = sha256`, `unix-archive = .tar.gz`.
- **#7 gates real bytes, not the site.** The first cargo-dist release is not cut yet
  (current releases are Go-era `naba_darwin_amd64.tar.gz`). The `install.sh` mirror tooling
  must tolerate "no cargo-dist release yet"; the site can go live regardless.

## Approach

**Mirror the proven Pelican+AWS pattern; keep GitHub canonical; host only `install.sh`; gate
the live apply.**

- **Static engine:** Pelican, pinned in `web/requirements.txt`, driven by a `web/Makefile`
  cloned/adapted from the reference (`devserver`, `html`, `validate`, `publish`,
  `s3_upload`, `sync_installer`, `invalidate`, `provision`). All site source lives under
  `web/`; `web/output/` is gitignored build output.
- **Theme:** a bespoke, self-contained dark terminal/technical theme under
  `web/themes/naba-terminal/` (Jinja templates + one hand-authored CSS file; monospace,
  dark bg, green/cyan accent, faux-terminal command blocks). No external theme dependency.
- **Content:** Markdown pages under `web/content/` — home (name + description + hero
  command), install/setup, usage (commands with sample output images), configuration.
  GitHub link in header/footer; install page links to GitHub Releases for browsing versions.
  Sample output images are pre-generated and committed as static assets (no build-time API
  calls).
- **URL style (decided, pins C1 below):** the site uses **pretty, directory-style URLs**
  (`/install/` → `install/index.html`), matching the reference site. This makes subdirectory
  index resolution a first-class hosting requirement (see the CloudFront Function below).
- **Hosting topology (single origin, path-based):**
  - One **private** S3 bucket (`naba.ysapp.net`), read only via CloudFront **Origin Access
    Control** (not public website hosting).
  - **Subdirectory index resolution (required):** a private bucket + OAC does NOT append
    `index.html` to subdirectory requests — CloudFront's default-root-object only rewrites
    `/`. A **CloudFront viewer-request Function** rewrites path-terminating requests
    (`/install/` → `/install/index.html`, and extension-less paths) so pretty URLs resolve.
    This is an explicit deliverable of Issue 3.1. **CloudFront error behavior:** map origin
    403/404 to a custom 404 response served from `404.html` (private-bucket AccessDenied
    otherwise surfaces as a bare 403).
  - CloudFront distribution; **ACM cert in `us-east-1`** (DNS-validated via a Route53
    record) for `naba.ysapp.net`; Route53 **A/ALIAS** `naba.ysapp.net` → the distribution.
  - `/install.sh` is just one object key in the same bucket — no redirects or second origin.
- **Caching strategy (from the design discussion):**
  - `/install.sh` — the one mutable pointer: uploaded with an explicit
    `Cache-Control: max-age=300` (per-key, NOT inherited from the tree-wide `s3 sync`) **and**
    explicitly invalidated on each publish.
  - HTML/CSS/assets — invalidated on publish (small site; `/*` acceptable, or scoped paths).
- **`install.sh` bootstrap (human-facing, GH canonical):** a `web/scripts/sync_installer.sh`
  fetches cargo-dist's `naba-installer.sh` from the GitHub Release named by the latest
  `dist-manifest.json` (pinned tag + HTTPS fetch) and stages it as site-root `install.sh`.
  If that release publishes a checksum for the installer script, verify it; otherwise the
  integrity guarantee is the pinned-tag HTTPS fetch (the installer in turn fetches
  sha256-checksummed *tarballs* from GitHub Releases — see C2 in reviews/pass-1.md). It
  tolerates "no cargo-dist release yet" (stages a friendly placeholder `install.sh` that
  prints "no release yet — see github.com/dixson3/naba/releases" and exits non-zero, so a
  premature `curl | sh` fails safe). Thereafter `naba self update` runs off GitHub Releases.
- **Local-first, gated go-live:** iterate with `pelican devserver`; a human **look-approved**
  gate blocks provisioning; a human **go-live** gate blocks the billable AWS apply
  (bucket/cert/distribution/records + cert-validation wait).

## Epics

### Epic 1: Pelican site scaffold + bespoke dark-terminal theme (local)
- Issue 1.1: Scaffold `web/` — `requirements.txt` (pinned Pelican + sitemap/markdown/yaml),
  `pelicanconf.py`, `publishconf.py` (prod `SITEURL=https://naba.ysapp.net`), `Makefile`
  (adapted from reference), `content/` + `themes/` dirs, and `.gitignore` for `web/output/`.
  **Pin the URL style to pretty directory-style URLs** (`PAGE_URL`/`ARTICLE_URL` =
  `{slug}/`), the decision that C1's CloudFront index-rewrite Function depends on; the theme
  (1.2) authors nav against this style.
- Issue 1.2: Author the bespoke `web/themes/naba-terminal/` theme — `base.html`,
  `index.html`, `page.html`, header/footer partials, and one hand-authored CSS file (dark
  bg, monospace, green/cyan accent, terminal-style command blocks). GitHub link in nav.
  - depends-on: 1.1
- Issue 1.3: Author content pages under `web/content/` — home (name, description, hero
  command), install/setup guide (bootstrap `curl … install.sh`, Homebrew, cargo, from
  source, with a link to GitHub Releases), usage guide (generate/edit/restore commands with
  sample-output images), configuration guide (config file, env vars, provider/API-key
  setup), and a **`404.html`** page (the target of the CloudFront 403/404 error mapping in
  Issue 3.1 — Pelican emits none by default). Footer/header link back to
  `github.com/dixson3/naba`.
  - depends-on: 1.2
- Issue 1.4: Collect/commit sample output images as static assets for the usage page
  (naba's own generated output; pre-generated, no build-time API calls).
  - depends-on: 1.1
- Issue 1.5: **[Capability gate: look-approved]** Build clean (`make validate`) and run
  `make devserver`; operator reviews the local site and iterates on the look until approved.
  - depends-on: 1.2, 1.3, 1.4

### Epic 2: install.sh bootstrap tooling (GitHub canonical)
- Issue 2.1: Implement `web/scripts/sync_installer.sh` — resolve the latest release tag from
  `dist-manifest.json`, fetch cargo-dist's `naba-installer.sh` for that pinned tag over HTTPS
  (via `gh`/HTTP), and stage it as site-root `install.sh`. **Verify the installer's sha256
  only if the manifest publishes one for the installer script** (cargo-dist emits `.sha256`
  sidecars for tarballs; the installer script may not have one — if absent, the guarantee is
  the pinned-tag HTTPS fetch, and the script documents this). Tolerate "no cargo-dist release
  yet" by staging a fail-safe placeholder `install.sh` (prints a "no release yet" message +
  the GitHub Releases URL, exits non-zero). Wire a `Makefile` `sync_installer` target that
  uploads `install.sh` with explicit `--cache-control max-age=300` and includes it in the
  publish + invalidation.
- Issue 2.2: Ensure the install page's bootstrap command and the GitHub-Releases link are
  consistent with the hosted `install.sh` behavior (incl. the placeholder state); document
  the GH-canonical boundary inline on the page.
  - depends-on: 2.1, 1.3

### Epic 3: AWS provisioning + go-live (gated, billable)
- Issue 3.1: Write the provisioning runbook + script `web/scripts/provision_aws.sh`,
  **idempotent via check-before-create per resource**: for each of the bucket, OAC, ACM
  cert, DNS-validation record, CloudFront Function, distribution, and Route53 alias, look up
  an existing identifier first and reuse it, persisting the captured ids (cert ARN,
  distribution id, OAC id, function ARN) to a local config (also feeding `web/Makefile`
  `CF_DISTRIBUTION`) so a re-run never requests a second cert or builds a duplicate
  distribution. Steps: create the **private** `naba.ysapp.net` S3 bucket; create CloudFront
  **OAC** + bucket policy; request an **ACM cert in us-east-1** and add the Route53
  DNS-validation record, then **poll `wait certificate-validated` with an explicit timeout**
  (surface a clear message if validation stalls); author + attach the **CloudFront
  viewer-request Function** that appends `index.html` for pretty-URL subdirectory requests
  (C1); create the CloudFront distribution with the TTL cache behaviors (short-TTL
  `/install.sh`) and the **custom 403/404 → `404.html`** error responses; upsert Route53
  A/ALIAS `naba.ysapp.net` → distribution.
- Issue 3.2: **[Capability gate: go-live]** Operator authorizes the billable apply (creates
  real infra + involves the ACM cert-validation wait). Test: AWS creds present
  (`aws sts get-caller-identity`) AND operator confirmation.
  - depends-on: 3.1, 1.5 (look-approved)
- Issue 3.3: Run provisioning + first deploy: `make publish` → `s3_upload` (site) →
  `sync_installer` (uploads `install.sh` with `--cache-control max-age=300`) → CloudFront
  invalidation of the mutable paths (`/install.sh`, HTML). Enforce HTTPS.
  - depends-on: 3.2
- Issue 3.4: Verify live — `https://naba.ysapp.net` serves over HTTPS, the install/usage
  pages render, `/install.sh` is fetchable, and
  `curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh` returns the
  installer (well-formed; a real end-to-end install is meaningful once #7 is cut, otherwise
  the fail-safe placeholder).
  - depends-on: 3.3

### Epic 4: Documentation + repo integration
- Issue 4.1: `web/README.md` — how to develop (`make devserver`), validate, provision
  (`provision_aws.sh`), deploy (`s3_upload`/`sync_installer`/`invalidate`), the GH-canonical
  / site-bootstrap boundary, and a **teardown/rollback note** (how to remove the
  `naba.ysapp.net` stack: distribution, cert, bucket, Route53 alias — the create-only
  provisioning's inverse) since it is a billable operator-owned stack.
  - depends-on: 1.1
- Issue 4.2: Link the site from the project `README.md`, add the bootstrap install command,
  and cross-reference #7 as the trigger that makes `install.sh` carry real bytes.
  - depends-on: 3.4
- Issue 4.3: File a **follow-on bead** (discovered-from this plan, `depends-on` upstream #7)
  capturing the post-#7 activation: once the first `v<semver>` tag is cut, re-run
  `make sync_installer` + deploy + invalidate `/install.sh`, then verify
  `curl … install.sh | sh` end-to-end. This makes the "lights up after #7" trigger owned
  work rather than an assumption (pass-1 C3).

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: look-approved
- Type: human
- Approvers: operator
- Condition: operator has reviewed the local `pelican devserver` render and approved the
  theme/content look.
- Test: `cd web && make validate` exits 0 (clean production build) — plus operator sign-off.
- Blocks: Issue 3.2 (go-live), i.e. no public push before the look is approved.
- Instructions: run `cd web && make devserver`, open the local URL, iterate on
  `themes/naba-terminal/` + `content/` until satisfied, then release the gate.

### Capability Gate: go-live (billable AWS apply)
- Type: human
- Approvers: operator
- Condition: operator authorizes creating real, billable AWS infrastructure for
  `naba.ysapp.net`.
- Test: `aws sts get-caller-identity` succeeds (creds present) AND operator confirmation.
- Blocks: Issue 3.3 (provision + deploy).
- Instructions: confirm the AWS account/region and that the ACM cert-validation wait is
  acceptable, then release the gate. **Expected cost:** low — S3 storage + requests for a
  small static site, CloudFront egress at low traffic, ACM certs are free, and the Route53
  zone already exists; realistically a few dollars/month or less at expected traffic.

## Risks & Mitigations
- **#7 not cut → `install.sh` has no real installer.** Until the first cargo-dist release,
  there is no `naba-installer.sh` to mirror. *Mitigation:* `sync_installer.sh` stages a
  fail-safe placeholder `install.sh` (clear message + GitHub Releases link, non-zero exit)
  so a premature `curl | sh` fails safely; the site ships and, once #7 lands, is activated by
  the **owned follow-on** (Issue 4.3 / a bead `depends-on` #7) that re-runs `sync_installer` +
  deploy + invalidate + end-to-end verify — not by an unowned assumption. Plan does not block
  on #7.
- **ACM cert must be in us-east-1.** CloudFront only reads certs from us-east-1 regardless
  of bucket region. *Mitigation:* provisioning script pins `--region us-east-1` for the cert
  and hard-checks it.
- **`install.sh` served stale.** CloudFront caches it. *Mitigation:* short TTL + explicit
  invalidation of `/install.sh` on every publish.
- **Billable infra created during a plan.** *Mitigation:* the go-live gate; nothing billable
  is created before the operator releases it. Local iteration uses no AWS.
- **OAC/private-bucket misconfig locks out CloudFront.** *Mitigation:* follow the documented
  OAC + bucket-policy sequence; Issue 3.4 verifies end-to-end fetch before declaring live.
- **`curl | sh` trust/integrity.** The hosted `install.sh` is a byte-for-byte mirror of
  cargo-dist's installer, which itself fetches sha256-checksummed tarballs from GitHub
  Releases. *Mitigation:* `sync_installer.sh` fetches the installer for a **pinned release
  tag over HTTPS** and verifies its sha256 **if the manifest publishes one for the installer
  script** (tarballs always have sidecars; the installer may not — pinned-tag HTTPS fetch is
  the floor guarantee); document that GitHub Releases remains canonical for all bytes.
- **Subdirectory index resolution under private-bucket OAC.** Pretty URLs 403/404 without a
  rewrite. *Mitigation:* the CloudFront viewer-request Function (Issue 3.1) appends
  `index.html`; Issue 3.4 verifies non-root pages resolve before declaring live.

## Success Criteria
1. `web/` contains a self-contained Pelican site (pinned deps, `Makefile`, `pelicanconf.py`
   + `publishconf.py`, bespoke `naba-terminal` theme) that builds cleanly with
   `make validate` and serves locally via `make devserver`.
2. Content covers all required sections: name, description, example usage with sample
   output images, configuration guide, install/setup guide, and a GitHub project link.
3. `web/scripts/sync_installer.sh` stages a site-root `install.sh` mirrored from a pinned
   cargo-dist release tag over HTTPS (sha256-verified when the manifest publishes an installer
   checksum), and degrades to a fail-safe placeholder when no cargo-dist release exists — with
   **no changes to naba's self-update code** and **no `/downloads/` binary mirror or
   `latest.json`** (GitHub Releases remains canonical).
4. After the go-live gate: `https://naba.ysapp.net` serves the site over HTTPS from
   S3+CloudFront, and
   `curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh` returns the
   installer (well-formed; a real end-to-end install is meaningful once #7 is cut).
5. The look was reviewed locally and approved before any public push (look-approved gate
   honored).
6. `web/README.md` documents develop/validate/provision/deploy and the GH-canonical vs
   site-bootstrap boundary; the project README links the site and shows the bootstrap
   command.

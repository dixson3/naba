# Upstream Issue #7

- **Repo:** dixson3/naba
- **URL:** https://github.com/dixson3/naba/issues/7
- **Title:** Cut the first cargo-dist release of naba (activates self-update)
- **State:** OPEN
- **Disposition in this plan:** partial (related, not resolved). This plan builds the
  `install.sh` mirror tooling that depends on a cargo-dist release, and tolerates its
  absence; it does NOT cut the release. #7 stays open.

## Body (verbatim)

Deferred follow-on from plan-005. Execution built+tested the distribution config,
self-update pipeline (behind the Fetcher seam), and preflight, but did NOT cut a release.
Until a v<semver> tag exists, 'naba self update' and the preflight binary axis are inert
against a live endpoint (dist-manifest.json URL does not exist yet).

Do: (1) cut the first v<semver> tag so cargo-dist publishes tarballs + dist-manifest.json +
the curl|sh installer + Homebrew formula; (2) confirm the curl|sh installer writes
~/.config/naba/naba-receipt.json to ~/.local/bin; (3) verify 'naba self update' end-to-end
against the published manifest (fetch, sha256 verify, self_replace swap, post-update skills
upgrade); (4) confirm 'naba skills preflight' binary axis flips unknown -> up_to_date/
update_available once the update-check cache is populated. References plan-005 Success
Criteria 1 and 3.

## Relationship to this plan

The website's hosted `install.sh` (site root) is a mirror of the cargo-dist
`naba-installer.sh` that #7 will first publish. Until #7 lands:

- `web/scripts/sync_installer.sh` stages a fail-safe placeholder `install.sh` (prints a
  "no release yet" message + the GitHub Releases URL, exits non-zero) so a premature
  `curl … install.sh | sh` fails safely.
- Once #7 cuts the first `v<semver>` tag, re-running `sync_installer` + a deploy makes the
  bootstrap carry real bytes automatically. No plan re-work is needed.

This plan therefore does not block on #7 and does not resolve it.

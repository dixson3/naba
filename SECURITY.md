# Security Policy

## Supported versions

naba is pre-1.0 and ships from a single line of development. Security fixes land
on `main` and in the **latest release**; please upgrade to the latest release
(`brew upgrade naba`, `naba self update`, or a fresh install) before reporting.

## Reporting a vulnerability

**Please do not open a public issue for security problems.**

Report privately via GitHub's **private vulnerability reporting**:

> Repository → **Security** tab → **Report a vulnerability**
> (<https://github.com/dixson3/naba/security/advisories/new>)

This opens a private advisory visible only to you and the maintainer.

A useful report describes a **real, reproducible, independently verifiable** issue
in actual naba source, and includes:

- the affected version (`naba version`) and platform,
- a minimal reproduction (exact commands / inputs),
- the impact and, if known, the relevant code path.

This is a single-maintainer project — expect a best-effort response, not an SLA.
Once a fix is available it will be released and the advisory published with credit
(unless you prefer to remain anonymous).

## AI-generated security reports

AI-assisted research is welcome, but unverified AI output is not. If you used AI to
prepare a report you **must disclose that**, and you must have **verified the issue
yourself** against real naba source. Reports that appear to be unverified AI output
— plausible-sounding but not reproducible — will be closed without response. See
[`CONTRIBUTING.md`](CONTRIBUTING.md#ai-assisted-contributions) for the rationale.

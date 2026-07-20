<!-- Thanks for contributing to naba! Please read CONTRIBUTING.md first. -->

## Summary

<!-- What does this change and why? Keep the "why" in your own words. -->

Closes #<!-- issue number, if applicable -->

## Checklist

- [ ] I ran the full validation suite locally and it passes:
      `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
      `cargo test`, `uv run tests/parity/check_traceability.py`, and the parity
      suite (`make parity`).
- [ ] If this changes observable behavior, I updated the relevant
      `docs/specifications/*.md` spec (append-only clause IDs) **and** added/adjusted
      a parity test that cites the clause.
- [ ] No secrets or account-specific values (API keys, ARNs, account ids, zone/GA
      ids) are committed.
- [ ] **I understand this change and can explain, defend, and maintain it** —
      including any AI-assisted parts. (Human accountability.)
- [ ] If AI assisted this change, I disclosed it via a commit trailer
      (e.g. `Co-Authored-By: Claude <noreply@anthropic.com>` or
      `Assisted-by: <tool>`). *Recommended, not required.*

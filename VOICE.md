# Writing Voice & Style — user-facing docs

Guidelines for **user-facing documentation**: `README.md`, `CONTRIBUTING.md`, the website under
`web/content/`, and any similar prose meant for people evaluating or using `naba`. (Code comments,
specification files, and internal notes are out of scope — this is about docs a reader encounters.)

## 1. Verbose, human-friendly exposition

Favor clear explanation over terse reference. A reader should be able to learn the concept from the
prose, not just look up a flag.

- **Lead with what a thing is** before its mechanics. Introduce a feature by explaining what it is
  and *why the reader cares*, then show the commands. (For example: "A skill teaches an AI coding
  agent a new capability…" comes before the install flags.)
- **Explain the why, not just the how.** When you document a behavior, say what problem it solves
  or when to reach for it.
- **Use plain-language examples.** Concrete, relatable examples ("make me an app icon of a rocket
  ship") land better than abstractions.
- **Cross-link related concepts** so a reader can follow the thread — e.g. the skills page points
  to the MCP page and back, each with a one-line "when to use which".
- Prefer complete sentences and short paragraphs where exposition helps. Bullets are for
  enumerations, not for compressing an explanation to the point of obscurity.

## 2. Precedence & ordered relationships — explicit lists, not arrow chains

Do **not** write an ordered or precedence relationship as an inline chain like `A > B > C` (or
`A → B → C` when it means "beats"). It is too idiomatic and easy to misread.

Instead, introduce it and use an explicit, labeled list. For example, rather than
"read from `AWS_REGION` > `AWS_DEFAULT_REGION` > the default", write:

> The region is resolved in this order (highest precedence first):
>
> 1. `AWS_REGION`
> 2. `AWS_DEFAULT_REGION`
> 3. the built-in default (`us-east-1`)

(A lettered `a) b) c)` list is equally fine — the point is an explicit, labeled list a reader can
scan.)

**Exception:** `→` used as a *mapping* — "`fast` → `gemini-3.1-flash-image`", "request → subcommand"
— reads clearly and is fine. That is "maps to / produces", not a precedence chain.

## 3. Name the tool as `naba`, never bare

When referring to **naba the tool** in prose, format the name — either code-style `` `naba` `` or
bold **naba** (pick one and stay consistent within a document; `` `naba` `` is the default for this
CLI). Never leave a bare `naba` sitting in a sentence.

Leave the name unformatted where it is **not** prose:

- headings, page titles, and subtitles;
- inside command examples and code blocks (`naba generate …`);
- as part of an identifier, path, URL, or env var — `naba.ysapp.net`, `~/.config/naba`, `/naba`,
  `naba-installer.sh`, `NABA_OUTPUT_DIR`.

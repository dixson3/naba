# INV-1 — Rust provider-layer spike: BESPOKE vs library

**Verdict: BESPOKE.** No surveyed Rust crate exposes OpenRouter image generation
behind a unified multi-provider image abstraction — confirmed three ways (source
inspection, the compiler via `E0599`, and a crates.io/lib.rs scan). Any library
path is a *hybrid* that still hand-writes the OpenRouter client, so you pay a heavy
dependency for only the easy (Gemini) half.

## Method

Read naba's real provider surface (`internal/gemini/*.go` — the entire Go Gemini
layer is **477 LOC**), then built two throwaway cargo crates on **stable Rust
1.96.1**, added each candidate, wrote code exercising naba's exact needs (Gemini
image gen + aspect ratio + image size + input image; OpenRouter image gen),
compiled, and inspected vendored crate source.

## Evidence table (compiler-validated)

| Candidate | Compiles | Ver | License | Deps | naba knobs? | OpenRouter image gen? | Image input (edit)? |
|:--|:-:|:--|:--|:-:|:--|:--|:--|
| **rath-rs** (`rath`) | ✅ | 0.2.4 | MIT/Apache | **153** | ❌ ImageRequest = {prompt,model,size(str),provider_config}; no aspect/quality | ❌ `create_image_client` wires **only `Provider::Fal`**; gemini/openrouter → `UnsupportedCapability` | ❌ no input-image field |
| **edgequake-llm** | ✅ | 0.10.1 | Apache | **180** (MSRV 1.95) | ✅ exact: `AspectRatio::as_gemini_str()` = all 14 ratios; `ImageResolution` = `512/1K/2K/4K` | ❌ compiler-confirmed absent (no `openrouter_from_env` → `E0599`); imagegen = gemini/vertex/fal/openai/xai/… | ✅ Gemini only (`reference_images`) |
| **bespoke** (reqwest+serde+tokio) | — | — | your MIT | ~50–80 (async baseline the port needs anyway) | ✅ you define | ✅ you write it (both) | ✅ both |

## Disqualifications

- **rath-rs: disqualified outright.** Its `images` abstraction is a FAL-only shim —
  covers ZERO of naba's two providers, no knobs, no edit path. Pre-alpha (0.2.4,
  ~149 downloads), 153 deps (incl. two `reqwest` versions) to deliver nothing usable.
- **edgequake-llm: covers the Gemini side impressively** (`AspectRatio`/
  `ImageResolution` enums map to naba's exact strings; `reference_images` = the edit
  input path) **but covers NONE of the OpenRouter side** — its `openrouter.rs` is
  vision *input* only, not image *output*. So OpenRouter stays 100% bespoke while you
  carry **180 transitive deps** (async-openai, tiktoken-rs, tracing-opentelemetry,
  tower-http, aws-lc…) to buy only the Gemini half. Defeats the point and shatters
  naba's near-zero-dep posture.

## Other-crate scan

The only Rust crate with a real OpenRouter *image-generation* surface is
**`openrouter-rs`** (0.11.1, MIT, ~19k downloads, active) — dedicated
`images().create` against OpenRouter's **`/images` endpoint** (aligns with INV-2's
finding that the dedicated `/api/v1/images` API is the right target). But it is
**OpenRouter-only** (Gemini only via OpenRouter routing, not direct), and its
aspect/size/edit-input coverage is unconfirmed. At best it is the OpenRouter *half*
of a hybrid, not a unified aggregation crate.

## Recommendation

**Fully bespoke.** Define naba's own `Provider` trait — `generate(prompt, cfg)`,
`generate_with_image(prompt, input, cfg)`, `list_models` — with a `GeminiProvider`
and an `OpenRouterProvider` over `reqwest` + `serde` + `tokio`, plus a trivial
selector factory (env-key autodetect → provider, per scope #4).

Rationale:
1. No library meets the decisive requirement (OpenRouter image gen behind a unified
   abstraction — does not exist in Rust).
2. Bespoke is small: Go layer is 477 LOC; Rust equivalent ~**500–800 LOC** over the
   reqwest/serde/tokio baseline any async port needs anyway — a fraction of 153–180
   transitive deps, preserving naba's near-zero-dep identity.
3. Full parity needs the edit/restore image-input path for BOTH providers — bespoke
   gives that directly; libraries give it for Gemini only.

**Optional narrow reuse (not adoption):** if the OpenRouter client proves fiddly,
`openrouter-rs` is the only crate worth a second look as an *isolated* OpenRouter
dependency — confirm its aspect/size/edit-input coverage first (overlaps INV-2) and
weigh against ~150 LOC of bespoke reqwest. Default remains fully bespoke.

**Requirements no library can meet:** OpenRouter image generation, OpenRouter image
input for edit/restore, OpenRouter default image model — absent from every unified
Rust crate surveyed. Live key needed only to confirm runtime wire behavior (INV-2's
job), not the library-selection verdict, which is conclusive on API surface.

Source: `internal/gemini/{client,types,imageconfig,models}.go`.
Sources: rath-rs (github.com/vivsh/rath-rs), edgequake-llm
(github.com/raphaelmansuy/edgequake-llm), openrouter-rs
(github.com/realmorrisliu/openrouter-rs).

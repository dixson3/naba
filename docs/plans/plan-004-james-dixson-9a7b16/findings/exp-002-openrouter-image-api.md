# INV-2 — OpenRouter image-generation API capabilities

**Two plan-changing findings:**
1. **OpenRouter shipped a dedicated Unified Image API (`POST /api/v1/images`,
   2026-06-23)** that exposes naba's `imageConfig` knobs almost 1:1 — a *better*
   fit than the chat-completions `modalities` path the plan assumed.
2. **CRITICAL NEGATIVE: `openrouter/auto` does NOT support image output.** Scope
   decision #4 ("multiple keys → OpenRouter `auto`") is **invalid for image
   generation** and must be revised to a concrete default image model slug.

## Wire format — target the dedicated Image API (Path A)

```
POST https://openrouter.ai/api/v1/images
Authorization: Bearer $OPENROUTER_API_KEY
Content-Type: application/json
{ "model": "google/gemini-3.1-flash-image-preview",
  "prompt": "...", "aspect_ratio": "16:9", "resolution": "2K" }
```
Response: OpenAI-Images-style envelope — base64 in `data[].b64_json` + `media_type`
(exact field names need live-key confirmation). New image models land here
exclusively; the legacy chat-completions `modalities:["image","text"]` path (images
in `message.images[].image_url.url` as data-URLs) still works for existing models
but is community-confirmed, not first-party-documented.

**Recommendation: back naba's OpenRouter provider on `POST /api/v1/images`.**

## Auth + headers

`Authorization: Bearer $OPENROUTER_API_KEY` is the ONLY required header.
`HTTP-Referer` / `X-Title` are optional app-identification headers. This is the
per-provider header seam the plan anticipated (vs Gemini's `x-goog-api-key`).

## Knob mapping (naba/Gemini → OpenRouter Image API)

| naba/Gemini knob | OpenRouter `/api/v1/images` | Clean? |
|:--|:--|:--|
| `imageConfig.aspectRatio` | `aspect_ratio` (`"16:9"`…) | Yes |
| `imageConfig.imageSize` (512/1K/2K/4K) | `resolution` (`512`/`1K`/`2K`/`4K`) | Yes — same vocabulary |
| quality → model tier | `quality` (`auto`/`low`/`medium`/`high`) native param | Yes — cleaner (first-class) |
| model selection | `model` slug (required) | Yes |
| image input (edit/restore) | `input_references[].image_url.url` (data-URL or HTTP) | Yes |
| `auto`/default | **no image `auto`**; default slug `google/gemini-3.1-flash-image-preview` | **No — must name a concrete model** |

Also available: `size` (tier or `WxH`), `output_format` (png/jpeg/webp/svg),
`background` (auto/transparent/opaque), `n` (1–10), `seed`, `stream`. Knobs are
**uniform in the API surface but capability-gated per model** — naba must handle
400s when a model rejects a knob (or pre-check via the per-model endpoint).

## Image input (edit/restore) — supported

`input_references[]` array of `image_url` blocks; `url` is an HTTP(S) URL or a
base64 data URL. Requires a model with `image` in `input_modalities`. Formats:
png/jpg/jpeg/webp/gif. Cleanly backs naba `edit`/`restore`. Per-model size limits
need a live key to confirm.

## `openrouter/auto` for images — NOT viable (scope decision #4 correction)

`openrouter/auto` is a **chat-completions meta-model** routing text prompts among
~39 **text** models; docs make no mention of image output, and `/api/v1/images`
requires an explicit image `model` slug (documented default:
`google/gemini-3.1-flash-image-preview`, "Nano Banana 2"). There is no documented
"auto image" router. **→ Revise scope #4: "multiple keys + no config default"
must resolve to a concrete image model slug, not `auto`.** (Live-key confirmation
that `/api/v1/images` rejects `model:"openrouter/auto"` is advisable but docs
strongly imply it.)

## Model discovery

- `GET /api/v1/models?output_modalities=image` (general API filter).
- `GET /api/v1/images/models` and per-model
  `GET /api/v1/images/models/{author}/{slug}/endpoints`.
- Capability fields: `input_modalities` / `output_modalities` arrays. 30+ image
  models (Google, OpenAI, Black Forest Labs, Recraft, ByteDance, xAI…). Slugs seen:
  `google/gemini-3.1-flash-image-preview`, `google/gemini-3.1-flash-lite-image`,
  `openai/gpt-image-1`, `bytedance-seed/seedream-4.5`.

## Errors → naba exit codes

401 (auth) → auth; 402 (credits); 403 (perms / **moderation** with `reasons`,
`flagged_input`) → content-policy; 408 timeout; 429 (platform or upstream rate
limit, honor `Retry-After`) → rate-limit; 500/502/503 (503 honor `Retry-After`) →
server. Maps cleanly onto naba's 1/2/3/4/5/10 matrix.

## Implications for INV-1 / approach

- **Strengthens BESPOKE.** A single typed HTTP client against `/api/v1/images`
  gives naba the exact knobs, small. Any pre-2026-06-23 aggregation library
  (dracory/llm, openrouter-agent-sdk-go, etc.) targets the LEGACY chat-completions
  image path and will NOT use the dedicated Image API or expose the new knobs —
  weakening the "adopt a library" case.
- **Retire the plan's chat-completions + `modalities` assumption** for the
  OpenRouter provider; target `/api/v1/images`.

## Needs a live key to confirm

Exact response field names; that `model:"openrouter/auto"` is rejected by
`/api/v1/images`; per-model `input_references` size/format limits + which models
honor which knobs; whether legacy `message.images` shape still returns for targeted
models.

## Sources

- https://openrouter.ai/blog/announcements/image-api/
- https://openrouter.ai/docs/guides/overview/multimodal/image-generation
- https://openrouter.ai/docs/api/api-reference/images/create-images
- https://openrouter.ai/openrouter/auto
- https://openrouter.ai/docs/api/reference/errors-and-debugging
- https://github.com/OpenRouterTeam/skills/blob/main/skills/openrouter-images/README.md

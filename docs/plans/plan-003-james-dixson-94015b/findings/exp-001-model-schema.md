# Finding: Gemini image model access + imageConfig schema (E1, E2)

Live verification against `https://generativelanguage.googleapis.com/v1beta` with the
maintainer's `GEMINI_API_KEY` on 2026-06-14. naba uses a hand-rolled HTTP client with
explicit Go structs (`internal/gemini/types.go`), so the **exact** JSON schema matters.

## Model availability (authoritative — `models.list`)

`GET /v1beta/models` for this key lists these image models, all with `generateContent`:

- `gemini-2.5-flash-image` (current naba config value)
- `gemini-3.1-flash-image` + `gemini-3.1-flash-image-preview`
- `gemini-3-pro-image` + `gemini-3-pro-image-preview`
- (Imagen 4.x models exist but use the `:predict` method, not `generateContent` — not a
  drop-in for naba's client.)

The repo default `gemini-2.0-flash-exp-image-generation` is **absent** from the list
(shut down 2025-11-14) → confirms the dead-default bug.

## E1 — gemini-3.1-flash-image works through naba's request shape ✓

Clean `POST …/models/gemini-3.1-flash-image:generateContent` with naba's current body
(`contents` + `generationConfig.responseModalities:[TEXT,IMAGE]`) → **HTTP 200**, returns
inline image. Adding `imageConfig` (below) also → **HTTP 200** with image.

## E2 — gemini-3-pro-image parity ✓

Same request shape + `imageConfig` against `gemini-3-pro-image` → **HTTP 200** with image.
So selecting Pro via raw `--model gemini-3-pro-image` needs **no client changes** beyond
passing the id. Document-only Pro exposure is sound.

## imageConfig schema (the load-bearing detail)

The accepted schema is **Schema A**, nested in `generationConfig`:

```json
{
  "contents": [{"parts": [{"text": "..."}]}],
  "generationConfig": {
    "responseModalities": ["TEXT", "IMAGE"],
    "imageConfig": { "aspectRatio": "16:9", "imageSize": "512" }
  }
}
```

- JSON keys: `generationConfig.imageConfig.aspectRatio` and `…imageConfig.imageSize`.
- The official-docs paraphrase suggesting `generationConfig.responseFormat.image{…}` is
  **wrong** — disproven by live 200s with `imageConfig`. (Do not trust the doc summary;
  the live API is authoritative.)
- Valid `aspectRatio`: `1:1, 1:4, 1:8, 2:3, 3:2, 3:4, 4:1, 4:3, 4:5, 5:4, 8:1, 9:16, 16:9, 21:9`.
- Valid `imageSize`: `512, 1K, 2K, 4K` (uppercase `K`).

### Go struct change implied (`internal/gemini/types.go`)

```go
type GenerationConfig struct {
    ResponseModalities []string     `json:"responseModalities"`
    ImageConfig        *ImageConfig `json:"imageConfig,omitempty"` // omitempty: bare calls unchanged
}
type ImageConfig struct {
    AspectRatio string `json:"aspectRatio,omitempty"`
    ImageSize   string `json:"imageSize,omitempty"`
}
```

`omitempty` keeps the current bare request byte-identical when no aspect/resolution is set
(control call confirmed 200).

## Surprise finding — API silently ignores invalid values (client-side validation REQUIRED)

`imageConfig.imageSize: "1k"` (lowercase, invalid per docs) returned **HTTP 200 with an
image, not an error**. The API does **not** validate these enum values — it silently
ignores a bad value and returns a default-size image. Implication: **naba must validate
`--aspect`/`--resolution` against the allowed enums client-side** (reject bad input with a
usage error, exit `ExitUsage`), because the server won't. Otherwise users get
silently-wrong output. This is a hard requirement, not a nicety.

## Response MIME — JPEG, not PNG

All 200 responses returned `inlineData.mimeType = image/jpeg` (with and without
imageConfig). naba's `extractImages` already passes through the response mimeType, but the
**output writer / extension handling must not assume PNG** — verify it derives the file
extension from the returned mimeType (or `-o` path), and that the smoke tests don't hardcode
`.png`. Add to the validation epic.

## Implications for the plan

1. Default → `gemini-3.1-flash-image` (callable, GA). ✓
2. Add `ImageConfig` struct + plumb `--aspect`/`--resolution` (Schema A). ✓
3. Pro via `--model gemini-3-pro-image` — no client change. ✓
4. **NEW:** client-side enum validation for aspect/resolution (API is permissive). 
5. **NEW:** confirm JPEG output handling (mimeType-driven extension), not PNG-hardcoded.

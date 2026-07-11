# Go multi-provider LLM/image libraries — landscape scan

Captured 2026-07-11 (exa web search). **naba is Go** (`github.com/dixson3/naba`,
go 1.25.7, cobra + stdlib `net/http`), so this Go scan supersedes the Rust scan
in `rust-multiprovider-libraries.md` (kept only as a record — the operator's
request said "rust library," but the codebase is Go; see plan.md open question).

**Decisive axis (unchanged):** image generation, with both **Gemini** and
**OpenRouter** as backends, ideally exposing naba's `imageConfig` knobs
(aspectRatio, imageSize) + quality tier, and — for full parity — image *input*
(edit/restore). naba is currently **near-zero-dependency** (cobra, yaml.v3,
mcp-go, stdlib HTTP/JSON). Any library adoption is a material dependency shift.

| Library | Image gen? | Gemini image | OpenRouter image | imageConfig knobs? | Notes |
|:--|:-:|:-:|:-:|:-:|:--|
| **`dracory/llm`** | ✅ | ✅ (via `google.golang.org/genai`) | ✅ (`ImageModel(ProviderOpenRouter)`, `GenerateImage([]byte)`) | ❓ unknown — simple `GenerateImage(prompt, options...)` | Closest fit: unified `LlmInterface` with `GenerateImage`; OpenRouter image via chat-completions `modalities:["image","text"]`. Gemini uses genai SDK (heavier than naba's bespoke HTTP). Text/JSON/XML/YAML + PNG/JPEG. |
| **`rcarmo/go-ai`** | ✅ | ✅ (google generative-ai) | ✅ (`images/openrouter`, 28 image models) | ❓ | Dedicated `images/` API + registry + provider interfaces; explicit OpenRouter image provider. Port of a TS lib (pi-ai). |
| `Vedanshu7/llmbridge` | ✅ (`ImageGenerator` iface) | ❓ | ❓ (OpenAI image confirmed) | ❓ | Big surface (router, proxy, caching, budgets). Heavy. Image gen shown for OpenAI. |
| `JoakimCarlsson/ai` | ✅ | ✅ | ❌ (OpenRouter = LLM only) | — | Image gen: OpenAI/Gemini/xAI. OpenRouter has no image row. |
| `elloloop/llmrouter` | ❌ | — | — (OpenRouter chat only) | — | Chat/embed/TTS/STT/rerank/realtime — NO image gen. |
| `Dragon-Born/go-llm` | ❌ | — | OpenRouter default (chat) | — | Fluent chat SDK; vision = image *input* only, no image *output*. |
| `dracory` OpenRouter image consts | — | — | — | — | Ships `OPENROUTER_MODEL_GEMINI_2_5_FLASH_IMAGE`, `OPENROUTER_MODEL_GPT_5_IMAGE`. |
| `ethpandaops/openrouter-agent-sdk-go` | ✅ (image *output* blocks) | — | ✅ (`ImageBlock.Save`, `OPENROUTER_IMAGE_MODEL`) | ❓ | OpenRouter-only (not multi-provider). Confirms OpenRouter image-gen via chat-completions works from Go. |

## OpenRouter image-generation: confirmed feasible

Multiple independent Go libs confirm OpenRouter does image generation via the
**chat-completions endpoint with `modalities: ["image","text"]`**, returning
images as data-URLs in the assistant message (`dracory/llm`,
`ethpandaops/openrouter-agent-sdk-go`, `rcarmo/go-ai`). Auth is
`Authorization: Bearer $OPENROUTER_API_KEY` (+ optional `HTTP-Referer` / `X-Title`).
This is a **different wire shape** from Gemini's `x-goog-api-key` +
`:generateContent` — confirming the architecture map's "per-provider translation
layer" seam.

## Takeaways

1. **A library CAN cover the `generate` (text-to-image) path** for both Gemini and
   OpenRouter — `dracory/llm` and `rcarmo/go-ai` are the two realistic candidates.
2. **Open risks that gate library adoption** (need investigation): (a) do they
   expose naba's `imageConfig` — aspectRatio + imageSize + quality tier — or only
   `GenerateImage(prompt)`? (b) do they support image *input* for edit/restore, or
   is that path still bespoke? (c) dependency weight — `dracory/llm` pulls
   `google.golang.org/genai`, replacing naba's lean bespoke Gemini HTTP client and
   possibly regressing the current imageConfig behavior (plan-003 work).
3. **A bespoke OpenRouter handler is small and low-risk**: one HTTP client
   (Bearer auth, chat-completions + modalities, data-URL decode), plus a provider
   interface over the existing `gemini.Client`. Keeps naba's zero-heavy-dep
   posture and preserves the plan-003 imageConfig wiring untouched. The provider
   selector is trivial (a factory over `NewClient`).

## Sources

- https://github.com/dracory/llm
- https://github.com/rcarmo/go-ai
- https://github.com/Vedanshu7/llmbridge
- https://github.com/JoakimCarlsson/ai
- https://github.com/elloloop/llmrouter
- https://github.com/Dragon-Born/go-llm
- https://github.com/ethpandaops/openrouter-agent-sdk-go

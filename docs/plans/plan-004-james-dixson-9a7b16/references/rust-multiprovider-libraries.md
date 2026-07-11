# Rust multi-provider LLM libraries — landscape scan

Captured 2026-07-11 (exa web search) to inform the "general Rust aggregation
library vs bespoke OpenRouter handler" decision.

**Critical framing:** naba is an **image-generation** tool. The decisive axis is
**image-generation** support *through OpenRouter*, not chat/text. Most unified
crates are chat-first; image-gen coverage is uneven and, crucially, **none of the
surveyed crates expose OpenRouter image generation** — their image backends are
OpenAI / Gemini / Vertex / FAL direct.

| Crate | Providers | Image gen | OpenRouter | OpenRouter image? | Notes |
|:--|:--|:-:|:-:|:-:|:--|
| `edgequake-llm` (0.6.14) | OpenAI, Azure, Anthropic, Gemini, Vertex, xAI, OpenRouter, NVIDIA, Mistral, Bedrock, Ollama, LM Studio, Copilot | ✅ (Gemini, Vertex Imagen, FAL, mock) | ✅ chat | ❌ | Broadest. Image gen via separate `ImageGenProvider`/`ImageGenFactory`, backends do NOT include OpenRouter. Pinned Rust 1.95. |
| `llmrust` (0.1.0) | OpenAI, Anthropic, DeepSeek, Gemini, Ollama, Moonshot, OpenRouter | ❌ (chat/embeddings/proxy) | ✅ chat | ❌ | LiteLLM-inspired; no image generation. |
| `litellm-rust` (avivsinai) | OpenAI-compat, Anthropic, Gemini, xAI | ✅ (OpenAI DALL-E/GPT-Image), Gemini/OpenAI video | via `provider/model` routing | ❌ | Image gen = OpenAI only; Gemini is video, not image. |
| `lmkit-rs` / `lmkit` (0.1.1) | OpenAI, Anthropic, Gemini, Aliyun, Ollama, Zhipu | partial (OpenAI, Aliyun; image "stubs") | ❌ | ❌ | No OpenRouter; image support thin. |
| `llm-kit-provider` | 12 providers (OpenAI, Anthropic, Azure, Groq, DeepSeek, xAI, TogetherAI, …) | partial (Azure, xAI, TogetherAI, OpenAI-compat) | ❌ | ❌ | No OpenRouter, no Gemini image. |
| `rath-rs` | OpenAI, OpenRouter, Gemini, Anthropic, Ollama, FAL | ✅ (`rath::images`, adapters incl. openrouter + gemini + fal) | ✅ | **maybe** | Only surveyed crate with BOTH an `images` capability AND an openrouter adapter. Uses `ModelUrl` locators (`openrouter:///…`, `gemini:///…`). Worth a focused probe. |

## Takeaways for scoping

1. **The library question hinges on image-gen-through-OpenRouter**, which is
   exactly where the ecosystem is weakest. A broad chat library (edgequake,
   llmrust) does not solve naba's problem — its image path wouldn't route through
   OpenRouter anyway.
2. **`rath-rs` is the one candidate** whose shape (capability-focused `images`
   module + openrouter/gemini/fal adapters + `ModelUrl` locators) could plausibly
   replace both a bespoke OpenRouter handler AND the provider selector. Needs a
   focused investigation: does its image client actually support OpenRouter image
   generation, is it published/maintained, license, dep weight, and does its
   image request/response model carry aspectRatio + imageSize (naba's imageConfig)?
3. **Open question — does OpenRouter even do image generation the way naba needs?**
   OpenRouter proxies image-capable models (e.g. Gemini image) via chat-completions
   with image modalities. Whether it exposes naba's knobs (aspect ratio, image size,
   quality tier) is unverified. This gates the whole plan and must be investigated
   before committing to an approach.

## Sources

- https://github.com/raphaelmansuy/edgequake-llm
- https://crates.io/crates/llmrust
- https://github.com/avivsinai/litellm-rust
- https://github.com/Zoranner/lmkit-rs
- https://lib.rs/crates/llm-kit-provider
- https://github.com/vivsh/rath-rs

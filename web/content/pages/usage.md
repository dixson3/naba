Title: usage
Slug: usage
Subtitle: commands, with the images they make

Every image command routes through one of naba's providers (Gemini, OpenRouter, or AWS
Bedrock) — see [config](/config/) for provider and key setup. Each example below shows the
exact command
**and the image it produced**; captions note the prompt and the model used. (These are real
naba outputs, mostly on the fast `gemini-3.1-flash-image` tier, with one on the higher-quality
`gemini-3-pro-image` tier.)

## generate

Turn a text prompt into an image.

```bash
naba generate "a red apple on a white background"
```

<figure class="sample">
  <img src="/images/samples/generate.jpg" alt="A red apple on a white background">
  <figcaption><span class="cap-prompt">"a red apple on a white background"</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
</figure>

```bash
naba generate "a serene mountain lake" --style watercolor
```

<figure class="sample">
  <img src="/images/samples/watercolor.jpg" alt="A serene mountain lake in watercolor style">
  <figcaption><span class="cap-prompt">"a serene mountain landscape lake, soft watercolor painting style"</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
</figure>

```bash
naba generate "a futuristic city skyline at dusk" --style pixel-art
```

<figure class="sample">
  <img src="/images/samples/pixelart.jpg" alt="A futuristic city skyline at dusk in pixel-art style">
  <figcaption><span class="cap-prompt">"a futuristic city skyline at dusk, pixel-art style, 16-bit"</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
</figure>

```bash
naba generate "a sweeping desert canyon vista at golden hour" --quality high --aspect 16:9
```

<figure class="sample">
  <img src="/images/samples/vista.jpg" alt="A sweeping desert canyon vista at golden hour">
  <figcaption><span class="cap-prompt">"a sweeping desert canyon vista at golden hour, ultra detailed" · --aspect 16:9</span><span class="cap-model">gemini-3-pro-image (--quality high)</span></figcaption>
</figure>

**Aspect ratio & resolution.** `--aspect` and `--resolution` set the Gemini `imageConfig`
and are available on all generative commands. Valid `--aspect`: `1:1, 2:3, 3:2, 3:4, 4:3,
4:5, 5:4, 9:16, 16:9, 21:9` (and the wide `1:4 … 8:1` extremes). Valid `--resolution`:
`512, 1K, 2K, 4K` (uppercase `K`). On Gemini, `--quality high` selects the
`gemini-3-pro-image` tier; `--quality fast` (default) is `gemini-3.1-flash-image`.

## edit

Modify an existing image with a natural-language instruction.

```bash
naba edit lake.jpg "make the sky dramatic and stormy"
```

<div class="io">
  <figure class="sample">
    <img src="/images/samples/edit-before.jpg" alt="Calm mountain lake under a clear sky, before edit">
    <figcaption><span class="io-tag">before</span> input image</figcaption>
  </figure>
  <figure class="sample">
    <img src="/images/samples/edit-after.jpg" alt="The same lake with a dramatic stormy sky, after edit">
    <figcaption><span class="io-tag io-after">after</span><span class="cap-prompt">"make the sky dramatic and stormy with heavy clouds"</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
  </figure>
</div>

## restore

Enhance or repair an old or degraded image.

```bash
naba restore old-photo.jpg "sharpen and improve colors"
```

<div class="io">
  <figure class="sample">
    <img src="/images/samples/restore-before.jpg" alt="A low-quality, degraded vintage portrait, before restore">
    <figcaption><span class="io-tag">before</span> degraded input</figcaption>
  </figure>
  <figure class="sample">
    <img src="/images/samples/restore-after.jpg" alt="The restored, sharpened portrait, after restore">
    <figcaption><span class="io-tag io-after">after</span><span class="cap-prompt">"sharpen, denoise, and improve colors"</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
  </figure>
</div>

## icon

Generate app icons in one or more sizes.

```bash
naba icon "rocket ship" --style flat --size 512
```

<figure class="sample">
  <img src="/images/samples/icon.jpg" alt="A flat-style rocket ship app icon">
  <figcaption><span class="cap-prompt">"rocket ship" · --style flat</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
</figure>

## pattern

Seamless, tileable textures and backgrounds.

```bash
naba pattern "circuit board" --style tech --colors mono
```

<figure class="sample">
  <img src="/images/samples/pattern.jpg" alt="A seamless monochrome circuit-board pattern">
  <figcaption><span class="cap-prompt">"circuit board" · --style tech --colors mono</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
</figure>

## story

A sequence of images that tell a visual narrative.

```bash
naba story "a small sailboat's voyage from calm harbor to open sea at sunset" --steps 3
```

<div class="sample-grid">
  <figure class="sample"><img src="/images/samples/story.jpg" alt="Story frame 1: sailboat in a calm harbor"><figcaption>frame 1</figcaption></figure>
  <figure class="sample"><img src="/images/samples/story-2.jpg" alt="Story frame 2: sailboat heading out"><figcaption>frame 2</figcaption></figure>
  <figure class="sample"><img src="/images/samples/story-3.jpg" alt="Story frame 3: sailboat on the open sea at sunset"><figcaption>frame 3</figcaption></figure>
</div>

<p class="cap-model" style="margin-top:6px">"a small sailboat's voyage from calm harbor to open sea at sunset" · --steps 3 · gemini-3.1-flash-image</p>

## diagram

Rendered technical diagrams from a description.

```bash
naba diagram "user authentication flow" --type flowchart
```

<figure class="sample">
  <img src="/images/samples/diagram.jpg" alt="A user authentication flowchart">
  <figcaption><span class="cap-prompt">"user authentication flow" · --type flowchart</span><span class="cap-model">gemini-3.1-flash-image</span></figcaption>
</figure>

## provider

List the registered providers and which have resolvable credentials. A `*` marks the
provider a bare image call would use (the effective default); each row shows its credential
status and effective default model.

```bash
naba provider
naba provider --json          # machine-readable {status, data} envelope
```

## models

List a provider's available models via a live API call. With no `--provider`, it lists the
resolved default provider's models; pass `--provider` to target another (it needs a resolvable
key for that provider).

```bash
naba models                       # the default provider's models
naba models --provider bedrock    # a specific provider's models
naba models --provider openrouter --json
```

---

Managing the binary itself — [`naba self update`](/config/#self-update),
[`naba doctor`](/config/#health-check-naba-doctor), and the
[`naba skills`](/config/#claude-code-skills) lifecycle — lives on the [config](/config/) page.

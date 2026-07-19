Title: usage
Slug: usage
Subtitle: commands, with the images they make

Every image command routes through one of two providers (Gemini or OpenRouter) — see
[config](/config/) for provider and key setup. The examples below show the command and a
sample of the kind of output it produces.

## generate

Turn a text prompt into an image.

```bash
naba generate "a red apple on a white background"
naba generate "mountain landscape" --style watercolor
naba generate "city skyline" -n 4 --style pixel-art
naba generate "wide vista" --aspect 16:9 --resolution 2K
naba generate "abstract art" -v lighting -v color-palette -o art.png
```

<figure class="sample">
  <img src="/images/samples/generate.jpg" alt="Sample generated image from naba generate">
  <figcaption>naba generate "a red apple on a white background"</figcaption>
</figure>

**Aspect ratio & resolution.** `--aspect` and `--resolution` set the Gemini `imageConfig`
and are available on all generative commands. Valid `--aspect`: `1:1, 2:3, 3:2, 3:4, 4:3,
4:5, 5:4, 9:16, 16:9, 21:9` (and the wide `1:4 … 8:1` extremes). Valid `--resolution`:
`512, 1K, 2K, 4K` (uppercase `K`). Invalid values are rejected before the API call.

## edit

Modify an existing image with a natural-language instruction.

```bash
naba edit photo.png "make the sky more dramatic"
naba edit portrait.jpg "add a hat" -o portrait-hat.png
```

<figure class="sample">
  <img src="/images/samples/edit.jpg" alt="Sample edited image from naba edit">
  <figcaption>naba edit photo.png "make the sky more dramatic"</figcaption>
</figure>

## restore

Enhance or repair an old or degraded image.

```bash
naba restore old-photo.jpg
naba restore blurry.png "sharpen and improve colors"
```

## icon

Generate app icons in one or more sizes.

```bash
naba icon "a music note" --size 64 --size 256 --size 512
naba icon "rocket ship" --style flat --background white --corners sharp
```

<figure class="sample">
  <img src="/images/samples/icon.jpg" alt="Sample icon set from naba icon">
  <figcaption>naba icon "rocket ship" --style flat</figcaption>
</figure>

## pattern

Seamless, tileable textures and backgrounds.

```bash
naba pattern "tropical leaves" --style floral --colors colorful
naba pattern "circuit board" --style tech --density dense --colors mono
```

<figure class="sample">
  <img src="/images/samples/pattern.jpg" alt="Sample seamless pattern from naba pattern">
  <figcaption>naba pattern "circuit board" --style tech --colors mono</figcaption>
</figure>

## story

A sequence of images that tell a visual narrative.

```bash
naba story "a cat's journey through a magical forest" --steps 6
naba story "sunrise to sunset" --steps 4 --transition dramatic
```

## diagram

Rendered technical diagrams from a description.

```bash
naba diagram "user authentication flow" --type flowchart
naba diagram "microservices architecture" --type architecture --complexity comprehensive
naba diagram "database schema for blog" --type database --style clean
```

<figure class="sample">
  <img src="/images/samples/diagram.jpg" alt="Sample diagram from naba diagram">
  <figcaption>naba diagram "user authentication flow" --type flowchart</figcaption>
</figure>

## self-update

A **vendor** install (the `curl | sh` bootstrap) updates itself in place. Homebrew installs
are refused with a pointer to `brew upgrade naba`.

```bash
naba self update            # fetch the latest release, verify sha256, swap in place
naba self update --check    # report whether an update is available; change nothing
naba self update --json     # machine-readable envelope
```

GitHub Releases is the canonical source for every binary and for the self-update manifest —
this site does not host or mirror binaries.

## health check

```bash
naba doctor                 # checks skills install, API key, model, config
naba doctor --json          # structured output
```

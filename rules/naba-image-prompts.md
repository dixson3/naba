# Naba Image Prompt Engineering

When crafting prompts for naba image generation commands, follow these guidelines to get the best results from the Gemini API.

## Prompt Structure

Build prompts in this order: **subject + composition + style + lighting + details**.

1. **Subject**: What is the main focus? Be specific — "a tabby cat sitting on a wooden fence" not "a cat"
2. **Composition**: Camera angle, framing, depth of field — "close-up shot", "bird's eye view", "centered with negative space"
3. **Style**: Art style or medium — maps to `--style` flag values (photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist)
4. **Lighting**: "golden hour", "soft diffused", "dramatic side lighting", "studio lighting"
5. **Details**: Color palette, mood, texture, atmosphere — "warm earth tones", "moody and atmospheric"

## Per-Command Guidance

### generate
General-purpose image creation. Prompts can be descriptive and open-ended. Use `--style` to anchor the visual treatment. Use `--variation` for systematic exploration of lighting, angle, color-palette, composition, mood, season, or time-of-day.

### edit
Prompts should describe the **desired change**, not the full image. Be surgical: "remove the background and replace with a sunset sky" or "change the shirt color to blue". The source image provides context.

### restore
Minimal prompting — the source image drives the output. Optional prompt refines the enhancement: "increase sharpness", "fix color balance", "remove noise". Omit the prompt for general restoration.

### icon
Prompts should focus on the **symbol or concept**, not composition (naba handles icon framing). Good: "a lightning bolt with circuit traces". Bad: "a 256x256 icon centered on a white background of a lightning bolt". Use `--style` for visual treatment (flat, skeuomorphic, minimal, modern).

### pattern
Describe the **motif and feel**, not the tiling mechanics. Good: "tropical leaves with monstera and palm fronds". The `--style`, `--colors`, and `--density` flags handle the technical pattern attributes.

### story
Write the **narrative arc**, not individual frames. Good: "a seed growing into a towering oak tree through the seasons". Naba splits this into `--steps` frames automatically. Use `--transition` to control how frames relate visually.

### diagram
Describe the **system or process** to visualize. Good: "microservices architecture with API gateway, auth service, and database layer". The `--type` flag selects the diagram format (flowchart, architecture, network, database, wireframe, mindmap, sequence).

## Anti-Patterns

- **Avoid negatives**: "no text" or "without watermarks" often backfire. Instead, describe what you want.
- **Avoid resolution specs in prompts**: Use CLI flags (`--size`, `--tile-size`) instead of "4K" or "1024x1024" in the prompt text.
- **Avoid overly long prompts**: 1-3 sentences is the sweet spot. Beyond that, details compete and quality drops.
- **Avoid generic prompts**: "a beautiful landscape" produces generic results. Add specifics: "a misty fjord at dawn with a lone fishing boat".

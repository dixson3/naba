# Implementation Guide: Output Handling

## 1. Overview

The output package handles three concerns: writing image files to disk, formatting JSON metadata for machine consumption, and launching system image viewers. Output behavior adapts based on TTY detection and flags.

## 2. Use Cases

| ID | Name | Actor | Preconditions | Flow | Postconditions |
|----|------|-------|---------------|------|----------------|
| UC-012 | Write image with auto-naming | CLI user | No --output flag | 1. Command generates image 2. Filename generated as `naba-{command}-{timestamp}.{ext}` 3. File written to current directory | Image at auto-generated path |
| UC-013 | Write image with custom path | CLI user | --output flag set | 1. Command generates image 2. Directory created if needed 3. File written to specified path 4. Dedup applied if file exists | Image at specified path (possibly with -N suffix) |
| UC-014 | JSON output for scripting | Script/LLM | --json flag or piped stdout | 1. Result struct populated (path, command, prompt, elapsed_ms, params, requested_format, actual_format) 2. MarshalIndent for readable JSON 3. Printed to stdout | JSON object (single) or JSON array (multi) on stdout |
| UC-015 | Preview in system viewer | CLI user | --preview flag | 1. Image written to disk 2. `open` (macOS) / `xdg-open` (Linux) / `start` (Windows) launched 3. Process started non-blocking | System default image viewer opens |
| UC-023 | Reconcile output extension to response format | CLI user / MCP client | -o path extension differs from response mimeType | 1. API returns JPEG 2. WriteImageResult compares requested ext to response mimeType 3. On mismatch, writes the corrected extension, sets Corrected 4. CLI warns on stderr and reports requested_format/actual_format; MCP result notes the format | File on disk has the correct extension for its bytes |

## 3. Implementation Notes

### File Naming

- Auto: `naba-{command}-{YYYYMMDD-HHMMSS}.{ext}` (e.g., `naba-generate-20260221-143022.png`)
- Multi-output: appends `-{N}` to base name for index > 0
- Dedup: if file exists, appends `-1`, `-2`, ... up to `-999`

### MIME Type Handling

- Extension mapping in `mimeTypeToExt()`: png, jpg, gif, webp
- Reverse mapping in `detectMIMEType()`: extension-based detection for input files
- Default: `image/png` / `.png` when unknown

### Extension Reconciliation (JPEG responses)

The Gemini image API returns `image/jpeg`. `WriteImageResult()` (the rich form of
`WriteImage()`) reconciles the on-disk extension to the response mimeType:

- Auto-named output (`-o` unset): the filename is generated from the response mimeType, so
  it is already correct (`.jpg`); no correction, no `requested_format`.
- Explicit `-o path`: the path's extension implies a `requested_format`. When it differs
  from the response format (e.g. `-o hero.png` for a JPEG), the extension is corrected on
  disk (`hero.jpg`), `Corrected` is set, and `actual_format` reflects the real format.
  `.jpg` and `.jpeg` are treated as equivalent (both `jpeg`), so neither is "corrected".

The CLI emits a stderr warning on a correction and surfaces `requested_format`/
`actual_format` in JSON. The `icon` command builds its path outside `WriteImage` but routes
through the same reconciliation. MCP handlers defer the extension to the actual mimeType
and add a `Format: <mimeType>` note to the tool result (MCP emits no CLI `Result` JSON).

### JSON Output Structure

```json
{
  "path": "/absolute/path/to/image.jpg",
  "command": "generate",
  "prompt": "original user prompt",
  "elapsed_ms": 3200,
  "params": {
    "style": "watercolor",
    "variations": ["lighting"],
    "aspect": "16:9",
    "resolution": "2K"
  },
  "requested_format": "png",
  "actual_format": "jpeg"
}
```

`requested_format`/`actual_format` are `omitempty`: `requested_format` appears only when
`-o` implied a format. Single results use `PrintJSON()`. Multiple results (count > 1,
story, multi-size icons) use `PrintJSONMulti()` which outputs a JSON array.

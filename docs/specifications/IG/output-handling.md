# Implementation Guide: Output Handling

## 1. Overview

The output package handles three concerns: writing image files to disk, formatting JSON metadata for machine consumption, and launching system image viewers. Output behavior adapts based on TTY detection and flags.

## 2. Use Cases

| ID | Name | Actor | Preconditions | Flow | Postconditions |
|----|------|-------|---------------|------|----------------|
| UC-012 | Write image with auto-naming | CLI user | No --output flag | 1. Command generates image 2. Filename generated as `naba-{command}-{timestamp}.{ext}` 3. File written to current directory | Image at auto-generated path |
| UC-013 | Write image with custom path | CLI user | --output flag set | 1. Command generates image 2. Directory created if needed 3. File written to specified path 4. Dedup applied if file exists | Image at specified path (possibly with -N suffix) |
| UC-014 | JSON output for scripting | Script/LLM | --json flag or piped stdout | 1. Result struct populated (path, command, prompt, elapsed_ms, params) 2. MarshalIndent for readable JSON 3. Printed to stdout | JSON object (single) or JSON array (multi) on stdout |
| UC-015 | Preview in system viewer | CLI user | --preview flag | 1. Image written to disk 2. `open` (macOS) / `xdg-open` (Linux) / `start` (Windows) launched 3. Process started non-blocking | System default image viewer opens |

## 3. Implementation Notes

### File Naming

- Auto: `naba-{command}-{YYYYMMDD-HHMMSS}.{ext}` (e.g., `naba-generate-20260221-143022.png`)
- Multi-output: appends `-{N}` to base name for index > 0
- Dedup: if file exists, appends `-1`, `-2`, ... up to `-999`

### MIME Type Handling

- Extension mapping in `mimeTypeToExt()`: png, jpg, gif, webp
- Reverse mapping in `detectMIMEType()`: extension-based detection for input files
- Default: `image/png` / `.png` when unknown

### JSON Output Structure

```json
{
  "path": "/absolute/path/to/image.png",
  "command": "generate",
  "prompt": "original user prompt",
  "elapsed_ms": 3200,
  "params": {
    "style": "watercolor",
    "variations": ["lighting"]
  }
}
```

Single results use `PrintJSON()`. Multiple results (count > 1, story, multi-size icons) use `PrintJSONMulti()` which outputs a JSON array.

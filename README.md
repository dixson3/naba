# naba

A standalone CLI for AI image generation using Google Gemini. Generate, edit, and transform images from the command line.

## Install

```bash
go install github.com/dixson3/naba/cmd/naba@latest
```

Or build from source:

```bash
git clone https://github.com/dixson3/naba.git
cd naba
go build -o naba ./cmd/naba
```

## Setup

Set your Gemini API key:

```bash
export GEMINI_API_KEY=<your-key>
```

Or save it to config:

```bash
naba config set api_key <your-key>
```

## Usage

### Generate images

```bash
naba generate "a red apple on a white background"
naba generate "mountain landscape" --style watercolor
naba generate "city skyline" -n 4 --style pixel-art
naba generate "abstract art" -v lighting -v color-palette -o art.png
```

### Edit images

```bash
naba edit photo.png "make the sky more dramatic"
naba edit portrait.jpg "add a hat" -o portrait-hat.png
```

### Restore/enhance images

```bash
naba restore old-photo.jpg
naba restore blurry.png "sharpen and improve colors"
```

### Generate icons

```bash
naba icon "a music note" --size 64 --size 256 --size 512
naba icon "rocket ship" --style flat --background white --corners sharp
```

### Generate patterns

```bash
naba pattern "tropical leaves" --style floral --colors colorful
naba pattern "circuit board" --style tech --density dense --colors mono
```

### Generate stories

```bash
naba story "a cat's journey through a magical forest" --steps 6
naba story "sunrise to sunset" --steps 4 --transition dramatic
```

### Generate diagrams

```bash
naba diagram "user authentication flow" --type flowchart
naba diagram "microservices architecture" --type architecture --complexity comprehensive
naba diagram "database schema for blog" --type database --style clean
```

### Configuration

```bash
naba config set api_key <key>
naba config set model gemini-2.0-flash-exp
naba config get model
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output structured JSON (auto-enabled when piped) |
| `-o, --output` | Output file path |
| `-q, --quiet` | Suppress progress output |
| `-m, --model` | Override Gemini model |
| `--no-input` | Disable interactive prompts |

## JSON Output

When `--json` is used (or stdout is piped), output includes:

```json
{
  "path": "/absolute/path/to/image.png",
  "command": "generate",
  "prompt": "a red apple",
  "elapsed_ms": 3200,
  "params": {
    "style": "watercolor"
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 3 | Authentication error |
| 4 | Rate limit exceeded |
| 5 | API error |
| 10 | File I/O error |

## License

MIT - see [LICENSE](LICENSE)

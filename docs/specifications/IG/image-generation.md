# Implementation Guide: Image Generation Commands

## 1. Overview

The core image generation pipeline is shared across all 7 generation commands. Each command follows the same flow: resolve API key -> enrich prompt -> call Gemini -> write output -> print result. This guide covers the shared pipeline and per-command specifics.

## 2. Use Cases

| ID | Name | Actor | Preconditions | Flow | Postconditions |
|----|------|-------|---------------|------|----------------|
| UC-001 | Generate image from text | CLI user | GEMINI_API_KEY set or in config | 1. User runs `naba generate "prompt"` 2. CLI resolves API key 3. Prompt enriched with optional style/variations 4. Gemini API called 5. Image decoded from base64 6. Written to file 7. Path printed | Image file exists at output path; JSON metadata printed if --json |
| UC-002 | Edit existing image | CLI user | GEMINI_API_KEY set; input image file exists | 1. User runs `naba edit photo.png "make sky blue"` 2. Input file validated 3. Image read and base64-encoded 4. Edit prompt enriched 5. Gemini API called with image+text 6. Result written to file | Edited image at output path |
| UC-003 | Restore/enhance image | CLI user | GEMINI_API_KEY set; input image file exists | 1. User runs `naba restore old-photo.jpg` 2. Default restoration prompt used if none provided 3. Gemini API called with image 4. Enhanced image written | Restored image at output path |
| UC-004 | Generate multi-size icons | CLI user | GEMINI_API_KEY set | 1. User runs `naba icon "rocket" --size 64 --size 256` 2. One API call per size 3. Icon prompt enriched with style/background/corners 4. Each result written with size in filename | One icon file per requested size |
| UC-005 | Generate story sequence | CLI user | GEMINI_API_KEY set; steps between 2-8 | 1. User runs `naba story "cat adventure" --steps 4` 2. Steps validated (2-8 range) 3. One API call per frame with step-aware prompt 4. Each frame written sequentially | N image files for N steps; API called N times |
| UC-006 | Generate diagram | CLI user | GEMINI_API_KEY set | 1. User runs `naba diagram "auth flow" --type flowchart` 2. Diagram prompt enriched with type/style/layout/complexity/colors 3. Single API call 4. Result written | Diagram image at output path |
| UC-007 | Script with JSON output | CI/automation | Piped stdout or --json flag | 1. Script runs `naba generate "prompt" \| jq .path` 2. Piped stdout auto-detected 3. JSON output with absolute path, elapsed_ms, params 4. Exit code indicates success/failure type | JSON on stdout; semantic exit code for error handling |
| UC-008 | Handle API errors gracefully | CLI user | Various error conditions | 1. User runs command 2. API returns error (401/429/5xx) 3. Error mapped to semantic exit code 4. Actionable message printed to stderr | Specific exit code; message with fix instructions |

## 3. Implementation Notes

### Shared Pipeline (all commands)

Every generation command in `internal/cli/` follows this pattern:

```go
func runCommand(cmd *cobra.Command, args []string) error {
    // 1. Parse args
    prompt := args[0]
    start := time.Now()

    // 2. Resolve API key (env var > config file)
    apiKey := resolveAPIKey()
    if apiKey == "" {
        return exitError(gemini.ExitAuth, "GEMINI_API_KEY not set...")
    }

    // 3. Resolve model (flag > config > default)
    model := flagModel
    if model == "" {
        cfg, _ := config.Load()
        model = cfg.Model
    }

    // 4. Create client and enrich prompt
    client := gemini.NewClient(apiKey, model)
    enrichedPrompt := gemini.EnrichXxxPrompt(prompt, ...flags)

    // 5. Call API
    images, err := client.Generate(enrichedPrompt)
    if err != nil {
        return handleAPIError(err)
    }

    // 6. Write results and collect metadata
    for i, img := range images {
        path, err := output.WriteImage(img.Data, img.MIMEType, flagOutput, "command", i)
        result := output.NewResult(path, "command", prompt, start)
        // ... set result.Params
    }

    // 7. Print JSON or human output
    if flagJSON {
        return output.PrintJSON(result) // or PrintJSONMulti
    }
    return nil
}
```

### Multi-Call Commands

`icon` and `story` make multiple API calls in a loop:
- `icon`: iterates over `iconSizes` slice, one call per size
- `story`: iterates from step 1 to `storySteps`, one call per frame
- `generate` with `--count > 1`: iterates `genCount` times

### Image Input Commands

`edit` and `restore` validate input file existence before API call, then use `client.GenerateWithImage()` which reads the file, base64-encodes it, and includes it as `inlineData` in the request.

### Testing Pattern

All CLI tests in `internal/cli/cli_test.go` follow:
1. Call `resetFlags()` to clear cobra state
2. Set env vars with `t.Setenv()`
3. Create mock server with `newMockServer(t)` or variants
4. Set `rootCmd.SetArgs([]string{...})`
5. Call `rootCmd.Execute()` and assert error/success
6. Verify output files or exit codes

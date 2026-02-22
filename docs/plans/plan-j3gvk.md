# Plan: Update README with MCP Usage Documentation

## Context

The `naba mcp` subcommand was just implemented but the README has no mention of MCP. Users need to know how to configure naba as an MCP server in their AI assistant (Claude Desktop, Cursor, etc.).

## Changes

**File:** `README.md`

Add an "MCP Server" section after the existing "Usage" subsections (after "### Configuration") and before "## Global Flags". Content:

### MCP Server section should include:

1. **Brief description** — what `naba mcp` does (stdio MCP server, 7 tools)
2. **Claude Desktop config** — JSON snippet for `claude_desktop_config.json`:
   ```json
   {
     "mcpServers": {
       "naba": {
         "command": "naba",
         "args": ["mcp"],
         "env": {
           "GEMINI_API_KEY": "<your-key>"
         }
       }
     }
   }
   ```
3. **Available tools table** — 7 tools with their names and one-line descriptions
4. **Manual test command** — show the initialize handshake for verification

## Verification

- Read the updated README to confirm formatting and accuracy
- Verify the JSON config snippet is valid

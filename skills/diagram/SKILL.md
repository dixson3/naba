# Generate Diagram

Generate technical diagrams using the naba CLI.

## Usage

```
/diagram <prompt> [--type <type>] [--style <style>] [--layout <layout>] [--complexity <level>] [--colors <scheme>]
```

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Refine the prompt**: Describe the **system or process** to visualize, not the visual layout. Apply the prompt engineering guidance below. Include key components, relationships, and data flow.

3. **Build and run the command**:
   ```bash
   naba diagram "<system description>" [--type <type>] [--style <style>] [--layout <layout>] [--complexity <level>] [--colors <scheme>]
   ```

4. **Present the result**: Show the output file path. Use the Read tool to display the generated diagram.

5. **Offer iteration**: Ask if the user wants to adjust complexity, layout, or add/remove components.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--type` | flowchart | flowchart, architecture, network, database, wireframe, mindmap, sequence |
| `--style` | professional | professional, clean, hand-drawn, technical |
| `--layout` | hierarchical | horizontal, vertical, hierarchical, circular |
| `--complexity` | detailed | simple, detailed, comprehensive |
| `--colors` | accent | mono, accent, categorical |
| `--output` | (auto) | Output file path |
| `--preview` | false | Open result in system viewer |

## Examples

```bash
# Architecture diagram
naba diagram "microservices with API gateway, auth service, user service, and PostgreSQL" --type architecture

# Simple flowchart
naba diagram "user login flow with MFA" --type flowchart --complexity simple

# Network topology
naba diagram "AWS VPC with public and private subnets, load balancer, and RDS" --type network --style technical

# Database schema
naba diagram "e-commerce schema with users, orders, products, and reviews" --type database --layout horizontal
```

## Prompt Engineering

Describe the **system or process** to visualize. Good: "microservices architecture with API gateway, auth service, and database layer". The `--type` flag selects the diagram format (flowchart, architecture, network, database, wireframe, mindmap, sequence).

### Anti-Patterns

- **Avoid negatives**: "no text" or "without watermarks" often backfire. Instead, describe what you want.
- **Avoid resolution specs in prompts**: Use CLI flags (`--size`, `--tile-size`) instead of "4K" or "1024x1024" in the prompt text.
- **Avoid overly long prompts**: 1-3 sentences is the sweet spot. Beyond that, details compete and quality drops.
- **Avoid generic prompts**: "a beautiful landscape" produces generic results. Add specifics: "a misty fjord at dawn with a lone fishing boat".

## Global Flags

| Flag | Short | Purpose |
|------|-------|---------|
| `--output` | `-o` | Output file path or directory |
| `--json` | | Structured JSON output (auto-enabled when piped) |
| `--quiet` | `-q` | Suppress progress output |
| `--model` | `-m` | Override Gemini model |
| `--preview` | | Open result in system viewer |

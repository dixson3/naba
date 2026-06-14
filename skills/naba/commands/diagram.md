# /naba diagram — rendered technical diagram image

**Tier:** inline. Generate a technical diagram **image** (not editable d2/mermaid source —
for that, use the `diagram-authoring` or `mermaid` skills).

## Usage

```
/naba diagram <prompt> [--type <type>] [--style <style>] [--layout <layout>] [--complexity <level>] [--colors <scheme>]
```

## Workflow

1. Refine the prompt: describe the **system or process** to visualize (key components,
   relationships, data flow) — not the visual layout.
2. Run: `naba diagram "<system description>" [--type <type>] [--style <style>] [--layout <layout>] [--complexity <level>] [--colors <scheme>]`
3. Present the output path; offer to adjust complexity, layout, or components.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--type` | flowchart | flowchart, architecture, network, database, wireframe, mindmap, sequence |
| `--style` | professional | professional, clean, hand-drawn, technical |
| `--layout` | hierarchical | horizontal, vertical, hierarchical, circular |
| `--complexity` | detailed | simple, detailed, comprehensive |
| `--colors` | accent | mono, accent, categorical |

(Plus the global flags in SKILL.md.)

## Notes

Describe the **system or process**, e.g. "microservices architecture with API gateway, auth
service, and database layer". The `--type` flag selects the diagram format.

## Examples

```bash
naba diagram "microservices with API gateway, auth service, user service, and PostgreSQL" --type architecture
naba diagram "user login flow with MFA" --type flowchart --complexity simple
naba diagram "AWS VPC with public and private subnets, load balancer, and RDS" --type network --style technical
naba diagram "e-commerce schema with users, orders, products, and reviews" --type database --layout horizontal
```

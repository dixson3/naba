# Working with input images

Two naba MCP tools consume an existing image through a required or optional `file` parameter:

- `edit_image` — `file` (required): the image to modify. `prompt` describes the desired change,
  not the whole image.
- `restore_image` — `file` (required): the image to restore or enhance. `prompt` is optional;
  restoration often needs little or no prompt.

## The `file` parameter is a server-side path

`file` is a filesystem path **the naba MCP server can read**, not an upload. Pass an absolute
path (or one resolvable from the server's working directory). The path must exist on the machine
running the server; if it does not, the tool returns `file not found: <path>` as a tool-level
error result (not a crash).

A natural source of input paths is a prior generation: every generation tool returns a written
path plus a `file://` resource link. Feed that same path back into `edit_image` or `restore_image`
to iterate on a result. `list_images` enumerates recent outputs in the MCP output directory when
you need to rediscover a path.

## Output

Like every naba tool, `edit_image` and `restore_image` write their result into the MCP output
directory (`NABA_OUTPUT_DIR` / configured dir / XDG default) and return the new path plus a
`file://` resource link — the input image is never modified in place.

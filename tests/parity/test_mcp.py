"""MCP-protocol conformance harness for ``naba mcp`` (Issue 1.4, SPEC §11).

Drives the ``$NABA_BIN mcp`` stdio server through the official MCP Python SDK
(``mcp``) and validates the tool + resource surface against SPEC §11
(SPEC-MCP-001..013). The same suite runs against the Go build (default) and the
future Rust build; ``$NABA_BIN`` selects the implementation (see
``harness.runner``).

What this file covers
---------------------
- **initialize** — server identity ``naba`` + a version; tool & resource
  capabilities registered (SPEC-MCP-001).
- **tools/list** — exactly the 8 pinned tools; per-tool params / enums /
  defaults / required-ness structurally asserted (SPEC-MCP-002..011) *and*
  snapshotted to a normalized golden ``golden/mcp/tools.json`` so the Rust
  server can be byte-diffed against the Go-captured expectation. The ``quality``
  param description is normalized to ``<QUALITY_DESC>`` because it is a
  [DIVERGENCE] under multi-provider (SPEC-MCP-003) — inventory/enums are pinned,
  the exact prose is not.
- **tools/call** — a happy-path ``generate_image`` against the recording
  provider mock (path + ``Format:`` text + ``file://`` resource link, image
  written under ``NABA_OUTPUT_DIR``; SPEC-MCP-004/013), ``edit_image`` /
  ``restore_image`` over a temp input file (SPEC-MCP-005/006), the missing-key
  error (SPEC-MCP-013), and the validation-error results — missing prompt,
  ``count`` out of range, ``steps`` out of range, missing ``file``, ``file not
  found`` — all as tool-level error results, not process crashes
  (SPEC-MCP-004/005/009/013).
- **list_images** (MCP-only) — newest-first ordering, ``limit`` default 20 and
  clamp, and the empty / missing-dir / no-output-dir messages (SPEC-MCP-011).
- **resources** — ``resources/templates/list`` pins the ``file:///{path}``
  template metadata (SPEC-MCP-012); ``resources/read`` of a real generated path
  is attempted and asserts blob+MIME on a binary that can serve it, but
  ``xfail``s on the Go build (see the resource-read test docstring).

Skips gracefully (pytest skip, not error) when the ``mcp`` SDK import fails.
"""

from __future__ import annotations

import asyncio
import base64
import json
import os
from pathlib import Path

import pytest

# --- graceful skip if the MCP SDK is unavailable --------------------------------------
mcp = pytest.importorskip("mcp", reason="MCP Python SDK ('mcp') not installed")
from mcp import ClientSession, StdioServerParameters  # noqa: E402
from mcp.client.stdio import stdio_client  # noqa: E402
from mcp.shared.exceptions import McpError  # noqa: E402

GOLDEN_DIR = Path(__file__).resolve().parent / "golden" / "mcp"

# Env keys the runner scrubs so a case is hermetic regardless of the host shell.
_SCRUB = (
    "GEMINI_API_KEY",
    "OPENROUTER_API_KEY",
    "GEMINI_BASE_URL",
    "OPENROUTER_BASE_URL",
    "NABA_CONFIG_DIR",
    "NABA_OUTPUT_DIR",
)


# --------------------------------------------------------------------------------------
# Env + async session plumbing
# --------------------------------------------------------------------------------------
def make_env(
    *,
    output_dir: str | os.PathLike | None,
    config_dir: str | os.PathLike | None = None,
    gemini_api_key: str | None = "mcp-test-key",
    gemini_base_url: str | None = None,
    openrouter_base_url: str | None = None,
    drop_home: bool = False,
) -> dict[str, str]:
    """Compose a hermetic child environment for the MCP server subprocess."""
    env = dict(os.environ)
    for key in _SCRUB:
        env.pop(key, None)
    if drop_home:
        # Force HOME="" (not merely absent): the MCP SDK re-seeds a default HOME
        # into the child env, and Go's os.UserHomeDir only errors on an *empty*
        # HOME. This drives DefaultOutputDir -> "" (SPEC-MCP-011 no-output-dir).
        env.pop("XDG_DATA_HOME", None)
        env["HOME"] = ""
    if output_dir is not None:
        env["NABA_OUTPUT_DIR"] = str(output_dir)
    if config_dir is not None:
        env["NABA_CONFIG_DIR"] = str(config_dir)
    if gemini_api_key is not None:
        env["GEMINI_API_KEY"] = gemini_api_key
    if gemini_base_url is not None:
        env["GEMINI_BASE_URL"] = gemini_base_url
    if openrouter_base_url is not None:
        env["OPENROUTER_BASE_URL"] = openrouter_base_url
    return env


async def _with_session(naba_bin: str, env: dict[str, str], body):
    """Open one ``naba mcp`` stdio session, run ``body(session)``, return its value."""
    params = StdioServerParameters(command=naba_bin, args=["mcp"], env=env)
    async with stdio_client(params) as (read, write):
        async with ClientSession(read, write) as session:
            init = await session.initialize()
            return await body(session, init)


def run_session(naba_bin: str, env: dict[str, str], body):
    """Synchronous driver: run an async ``body(session, init)`` to completion."""
    return asyncio.run(_with_session(naba_bin, env, body))


# --------------------------------------------------------------------------------------
# Content helpers
# --------------------------------------------------------------------------------------
def texts(result) -> list[str]:
    return [c.text for c in result.content if getattr(c, "type", None) == "text"]


def resource_links(result) -> list:
    return [c for c in result.content if getattr(c, "type", None) == "resource_link"]


# --------------------------------------------------------------------------------------
# --update-golden wiring (the pytest_addoption hook lives in conftest.py)
# --------------------------------------------------------------------------------------
def _update_golden(request) -> bool:
    if request.config.getoption("--update-golden"):
        return True
    return os.environ.get("UPDATE_GOLDEN", "") not in ("", "0", "false", "False")


def _read_or_update(path: Path, actual: str, *, update: bool) -> None:
    if update:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(actual)
        return
    if not path.exists():
        pytest.fail(
            f"missing golden {path} -- run with --update-golden to capture it first"
        )
    expected = path.read_text()
    assert actual == expected, (
        f"golden mismatch for {path.name}\n--- expected ---\n{expected}\n"
        f"--- actual ---\n{actual}"
    )


# --------------------------------------------------------------------------------------
# Pinned tool inventory + the SPEC-load-bearing shape of each tool (SPEC-MCP-002..011)
# --------------------------------------------------------------------------------------
TOOL_NAMES = [
    "generate_image",
    "edit_image",
    "restore_image",
    "generate_icon",
    "generate_pattern",
    "generate_story",
    "generate_diagram",
    "list_images",
]

# aspect/resolution/quality descriptions (SPEC-MCP-003). quality prose is a divergence,
# so it is pinned only in the structural check, never in the normalized golden.
ASPECT_DESC = "Aspect ratio (generationConfig.imageConfig.aspectRatio)"
RESOLUTION_DESC = "Image resolution (generationConfig.imageConfig.imageSize)"
QUALITY_ENUM = ["fast", "high"]

# description, required set, and the params whose enum/default/bounds SPEC pins.
EXPECTED = {
    "generate_image": {
        "description": "Generate an image from a text prompt",
        "required": ["prompt"],
        "imageconfig": True,
        "params": {
            "style": {"enum": ["photorealistic", "watercolor", "oil-painting",
                               "sketch", "pixel-art", "anime", "vintage",
                               "modern", "abstract", "minimalist"]},
            "count": {"type": "number", "default": 1, "minimum": 1, "maximum": 8},
            "seed": {"type": "number"},
            "variations": {"type": "array",
                          "items_enum": ["lighting", "angle", "color-palette",
                                         "composition", "mood", "season",
                                         "time-of-day"]},
        },
    },
    "edit_image": {
        "description": "Edit an existing image based on a text prompt",
        "required": ["prompt", "file"],
        "imageconfig": True,
        "params": {},
    },
    "restore_image": {
        "description": "Restore or enhance an existing image",
        "required": ["file"],
        "imageconfig": True,
        "params": {},
    },
    "generate_icon": {
        "description": "Generate app icons in multiple sizes",
        "required": ["prompt"],
        "imageconfig": False,  # quality only, no aspect/resolution
        "quality_only": True,
        "params": {
            "sizes": {"type": "array", "items_min": 16, "items_max": 1024},
            "style": {"default": "modern",
                      "enum": ["flat", "skeuomorphic", "minimal", "modern"]},
            "background": {"default": "transparent"},
            "corners": {"default": "rounded", "enum": ["rounded", "sharp"]},
            "format": {"default": "png", "enum": ["png", "jpeg"]},
        },
    },
    "generate_pattern": {
        "description": "Generate seamless patterns and textures",
        "required": ["prompt"],
        "imageconfig": True,
        "params": {
            "style": {"default": "abstract",
                      "enum": ["geometric", "organic", "abstract", "floral", "tech"]},
            "colors": {"default": "colorful", "enum": ["mono", "duotone", "colorful"]},
            "density": {"default": "medium", "enum": ["sparse", "medium", "dense"]},
            "size": {"default": "256x256"},
            "repeat": {"default": "tile", "enum": ["tile", "mirror"]},
        },
    },
    "generate_story": {
        "description": "Generate a sequence of images that tell a visual story",
        "required": ["prompt"],
        "imageconfig": True,
        "params": {
            "steps": {"type": "number", "default": 4, "minimum": 2, "maximum": 8},
            "style": {"default": "consistent", "enum": ["consistent", "evolving"]},
            "transition": {"default": "smooth",
                           "enum": ["smooth", "dramatic", "fade"]},
            "layout": {"default": "separate",
                       "enum": ["separate", "grid", "comic"]},
        },
    },
    "generate_diagram": {
        "description": "Generate technical diagrams and flowcharts",
        "required": ["prompt"],
        "imageconfig": True,
        "params": {
            "type": {"default": "flowchart",
                     "enum": ["flowchart", "architecture", "network", "database",
                              "wireframe", "mindmap", "sequence"]},
            "style": {"default": "professional",
                      "enum": ["professional", "clean", "hand-drawn", "technical"]},
            "layout": {"default": "hierarchical",
                       "enum": ["horizontal", "vertical", "hierarchical", "circular"]},
            "complexity": {"default": "detailed",
                           "enum": ["simple", "detailed", "comprehensive"]},
            "colors": {"default": "accent",
                       "enum": ["mono", "accent", "categorical"]},
        },
    },
    "list_images": {
        "description": "List recently generated images in the output directory",
        "required": [],
        "imageconfig": False,
        "params": {
            "limit": {"type": "number", "default": 20},
        },
    },
}


def canonical_tools(tools) -> str:
    """Normalized, diffable JSON snapshot of tools/list for the golden.

    Keeps name / description / inputSchema (the SPEC-pinned surface); drops
    library-specific fields (annotations, meta, outputSchema, icons, title). The
    ``quality`` param description is stabilized to ``<QUALITY_DESC>`` because it
    is a [DIVERGENCE] under multi-provider (SPEC-MCP-003).
    """
    out = []
    for tool in sorted(tools, key=lambda t: t.name):
        schema = json.loads(json.dumps(tool.inputSchema))  # deep copy
        props = schema.get("properties", {})
        if "quality" in props and "description" in props["quality"]:
            props["quality"]["description"] = "<QUALITY_DESC>"
        out.append({
            "name": tool.name,
            "description": tool.description,
            "inputSchema": schema,
        })
    return json.dumps(out, indent=2, sort_keys=True) + "\n"


# --------------------------------------------------------------------------------------
# initialize + tools/list
# --------------------------------------------------------------------------------------
def test_initialize_identity_and_capabilities(naba_bin, output_dir):
    """SPEC-MCP-001: server identity ``naba`` + a version; tool/resource caps."""
    async def body(session, init):
        return init

    init = run_session(naba_bin, make_env(output_dir=output_dir), body)
    assert init.serverInfo.name == "naba"
    assert init.serverInfo.version  # build-dependent; just require non-empty
    assert init.capabilities.tools is not None, "tool capability must be registered"
    assert init.capabilities.resources is not None, "resource capability must be registered"


def test_tools_list_inventory_and_schema(naba_bin, output_dir, request):
    """SPEC-MCP-002..011: exactly 8 pinned tools with the pinned param surface.

    Structural per-tool assertions plus a normalized golden snapshot.
    """
    async def body(session, init):
        return (await session.list_tools()).tools

    tools = run_session(naba_bin, make_env(output_dir=output_dir), body)
    by_name = {t.name: t for t in tools}

    # SPEC-MCP-002: exactly the 8 tools, no more, no fewer.
    assert sorted(by_name) == sorted(TOOL_NAMES), (
        f"tool inventory drift: {sorted(by_name)}"
    )

    for name, exp in EXPECTED.items():
        tool = by_name[name]
        schema = tool.inputSchema
        props = schema.get("properties", {})
        req = set(schema.get("required", []))

        # tool description (SPEC-MCP-004..011) — pinned verbatim.
        assert tool.description == exp["description"], f"{name} description"
        # required-ness.
        assert req == set(exp["required"]), f"{name} required set: {req}"

        # per-param enum / default / bounds.
        for pname, spec in exp["params"].items():
            assert pname in props, f"{name}.{pname} missing"
            prop = props[pname]
            if "type" in spec:
                assert prop.get("type") == spec["type"], f"{name}.{pname} type"
            if "enum" in spec:
                assert prop.get("enum") == spec["enum"], f"{name}.{pname} enum"
            if "default" in spec:
                assert prop.get("default") == spec["default"], f"{name}.{pname} default"
            if "minimum" in spec:
                assert prop.get("minimum") == spec["minimum"], f"{name}.{pname} min"
            if "maximum" in spec:
                assert prop.get("maximum") == spec["maximum"], f"{name}.{pname} max"
            if "items_enum" in spec:
                assert prop.get("items", {}).get("enum") == spec["items_enum"], (
                    f"{name}.{pname} items enum"
                )
            if "items_min" in spec:
                assert prop.get("items", {}).get("minimum") == spec["items_min"], (
                    f"{name}.{pname} items min"
                )
            if "items_max" in spec:
                assert prop.get("items", {}).get("maximum") == spec["items_max"], (
                    f"{name}.{pname} items max"
                )

        # shared imageConfig options (SPEC-MCP-003).
        if exp.get("imageconfig"):
            assert props["aspect"]["description"] == ASPECT_DESC, f"{name} aspect desc"
            assert props["aspect"].get("enum"), f"{name} aspect enum nonempty"
            assert props["resolution"]["description"] == RESOLUTION_DESC, (
                f"{name} resolution desc"
            )
            assert props["resolution"].get("enum"), f"{name} resolution enum nonempty"
            assert props["quality"]["enum"] == QUALITY_ENUM, f"{name} quality enum"
        if exp.get("quality_only"):
            assert props["quality"]["enum"] == QUALITY_ENUM, f"{name} quality enum"
            assert "aspect" not in props, f"{name} must not carry aspect"
            assert "resolution" not in props, f"{name} must not carry resolution"

    # Golden snapshot (exhaustive, normalized). Diffable Go-vs-Rust expectation.
    actual = canonical_tools(tools)
    _read_or_update(GOLDEN_DIR / "tools.json", actual, update=_update_golden(request))


# --------------------------------------------------------------------------------------
# tools/call — happy path
# --------------------------------------------------------------------------------------
def test_generate_image_happy_path(naba_bin, output_dir, provider_mock):
    """SPEC-MCP-004/013: generate_image -> path + ``Format:`` + file:// link; file on disk."""
    env = make_env(
        output_dir=output_dir,
        gemini_base_url=provider_mock.gemini_base_url,
        openrouter_base_url=provider_mock.openrouter_base_url,
    )

    async def body(session, init):
        return await session.call_tool("generate_image", {"prompt": "an apple"})

    result = run_session(naba_bin, env, body)
    assert not result.isError, texts(result)
    tx = texts(result)
    # First text is the path; a "Format: <mime>" note follows (SPEC-MCP-013).
    path = tx[0]
    assert any(t.startswith("Format: ") for t in tx), tx
    assert path.startswith(str(output_dir)), f"image not under NABA_OUTPUT_DIR: {path}"
    assert Path(path).is_file(), "generated image was not written to disk"
    # A file:// resource link points at the written image (SPEC-MCP-013).
    links = resource_links(result)
    assert links, "expected a resource_link content item"
    assert str(links[0].uri).startswith("file://"), links[0].uri
    # The mock recorded the enriched outgoing prompt (provider was actually called).
    assert provider_mock.last_prompt() is not None
    assert "an apple" in provider_mock.last_prompt()


def test_edit_image_happy_path(naba_bin, output_dir, provider_mock, tmp_path):
    """SPEC-MCP-005: edit_image over a temp input file -> written result."""
    src = tmp_path / "input.png"
    src.write_bytes(provider_mock.image_bytes)
    env = make_env(
        output_dir=output_dir,
        gemini_base_url=provider_mock.gemini_base_url,
        openrouter_base_url=provider_mock.openrouter_base_url,
    )

    async def body(session, init):
        return await session.call_tool(
            "edit_image", {"prompt": "make it blue", "file": str(src)}
        )

    result = run_session(naba_bin, env, body)
    assert not result.isError, texts(result)
    path = texts(result)[0]
    assert path.startswith(str(output_dir)) and Path(path).is_file()
    assert resource_links(result)


def test_restore_image_happy_path(naba_bin, output_dir, provider_mock, tmp_path):
    """SPEC-MCP-006: restore_image (prompt optional) over a temp input file."""
    src = tmp_path / "old.png"
    src.write_bytes(provider_mock.image_bytes)
    env = make_env(
        output_dir=output_dir,
        gemini_base_url=provider_mock.gemini_base_url,
        openrouter_base_url=provider_mock.openrouter_base_url,
    )

    async def body(session, init):
        return await session.call_tool("restore_image", {"file": str(src)})

    result = run_session(naba_bin, env, body)
    assert not result.isError, texts(result)
    path = texts(result)[0]
    assert path.startswith(str(output_dir)) and Path(path).is_file()


# --------------------------------------------------------------------------------------
# tools/call — validation error results (tool-level, not process crashes)
# --------------------------------------------------------------------------------------
@pytest.mark.parametrize(
    "tool,args,message",
    [
        ("generate_image", {}, "missing required parameter: prompt"),
        ("generate_image", {"prompt": "x", "count": 9}, "count must be between 1 and 8"),
        ("generate_image", {"prompt": "x", "count": 0}, "count must be between 1 and 8"),
        ("generate_story", {"prompt": "x", "steps": 9}, "steps must be between 2 and 8"),
        ("generate_story", {"prompt": "x", "steps": 1}, "steps must be between 2 and 8"),
        ("edit_image", {"prompt": "x"}, "missing required parameter: file"),
        ("restore_image", {}, "missing required parameter: file"),
    ],
)
def test_validation_error_results(naba_bin, output_dir, tool, args, message):
    """SPEC-MCP-004/005/006/009/013: validations surface as tool-level error results."""
    async def body(session, init):
        return await session.call_tool(tool, args)

    result = run_session(naba_bin, make_env(output_dir=output_dir), body)
    assert result.isError, f"expected an error result, got {texts(result)}"
    assert message in " ".join(texts(result)), texts(result)


def test_edit_missing_file_not_found(naba_bin, output_dir, provider_mock, tmp_path):
    """SPEC-MCP-005: a nonexistent file yields ``file not found: <path>``."""
    missing = tmp_path / "nope.png"
    env = make_env(
        output_dir=output_dir,
        gemini_base_url=provider_mock.gemini_base_url,
        openrouter_base_url=provider_mock.openrouter_base_url,
    )

    async def body(session, init):
        return await session.call_tool(
            "edit_image", {"prompt": "x", "file": str(missing)}
        )

    result = run_session(naba_bin, env, body)
    assert result.isError
    assert f"file not found: {missing}" in " ".join(texts(result)), texts(result)


def test_missing_api_key_error(naba_bin, output_dir, tmp_path):
    """SPEC-MCP-013: a call with no API key returns the missing-key error result.

    [DIVERGENCE] for the openrouter provider — the Go server is Gemini-only, so
    the message is the ``GEMINI_API_KEY`` form.
    """
    cfg = tmp_path / "config"
    cfg.mkdir()
    env = make_env(output_dir=output_dir, config_dir=cfg, gemini_api_key=None)

    async def body(session, init):
        return await session.call_tool("generate_image", {"prompt": "x"})

    result = run_session(naba_bin, env, body)
    assert result.isError
    assert "GEMINI_API_KEY not set" in " ".join(texts(result)), texts(result)


# --------------------------------------------------------------------------------------
# list_images (MCP-only, SPEC-MCP-011)
# --------------------------------------------------------------------------------------
def _seed_images(output_dir: Path, names_oldest_first: list[str]) -> list[str]:
    """Create ``naba-*`` files with strictly increasing mtimes; return newest-first."""
    paths = []
    for i, name in enumerate(names_oldest_first):
        p = output_dir / name
        p.write_bytes(b"x")
        os.utime(p, (1_000_000 + i, 1_000_000 + i))  # strictly increasing mtime
        paths.append(str(p))
    return list(reversed(paths))  # newest-first


def test_list_images_newest_first(naba_bin, output_dir):
    """SPEC-MCP-011: naba-* image files, newest-first by modtime, one text per path."""
    expected = _seed_images(
        Path(output_dir),
        ["naba-generate-1.png", "naba-edit-2.jpg", "naba-story-3.webp"],
    )
    # A non-naba file and a non-image extension are both ignored.
    (Path(output_dir) / "other.png").write_bytes(b"x")
    (Path(output_dir) / "naba-note.txt").write_bytes(b"x")

    async def body(session, init):
        return await session.call_tool("list_images", {})

    result = run_session(naba_bin, make_env(output_dir=output_dir), body)
    assert not result.isError, texts(result)
    assert texts(result) == expected, texts(result)


def test_list_images_limit_clamp_and_default(naba_bin, output_dir):
    """SPEC-MCP-011: limit caps the list; limit<1 is treated as the default 20."""
    names = [f"naba-generate-{i}.png" for i in range(25)]
    expected = _seed_images(Path(output_dir), names)

    def run(args):
        async def body(session, init):
            return await session.call_tool("list_images", args)
        return run_session(naba_bin, make_env(output_dir=output_dir), body)

    capped = run({"limit": 2})
    assert texts(capped) == expected[:2]

    # limit < 1 -> treated as 20 (SPEC-MCP-011).
    clamped = run({"limit": 0})
    assert texts(clamped) == expected[:20]

    # default (no limit) -> 20.
    default = run({})
    assert texts(default) == expected[:20]


def test_list_images_empty(naba_bin, output_dir):
    """SPEC-MCP-011: an output dir with no naba-* images -> ``No images found``."""
    async def body(session, init):
        return await session.call_tool("list_images", {})

    result = run_session(naba_bin, make_env(output_dir=output_dir), body)
    assert not result.isError
    assert texts(result) == ["No images found"], texts(result)


def test_list_images_dir_missing(naba_bin, tmp_path):
    """SPEC-MCP-011: a nonexistent output dir -> ``No images found (directory ...)``."""
    missing = tmp_path / "does-not-exist"
    async def body(session, init):
        return await session.call_tool("list_images", {})

    result = run_session(naba_bin, make_env(output_dir=missing), body)
    assert not result.isError
    assert texts(result) == ["No images found (directory does not exist)"], texts(result)


def test_list_images_no_output_dir(naba_bin, tmp_path):
    """SPEC-MCP-011: no resolvable output dir -> ``no output directory configured``.

    Reached only when both NABA_OUTPUT_DIR and the XDG default resolve empty; the
    latter needs ``$HOME`` unset (``os.UserHomeDir`` fails), with an empty config
    so ``default_output_dir`` is unset too.
    """
    cfg = tmp_path / "config"
    cfg.mkdir()
    env = make_env(output_dir=None, config_dir=cfg, drop_home=True)

    async def body(session, init):
        return await session.call_tool("list_images", {})

    result = run_session(naba_bin, env, body)
    assert result.isError
    assert "no output directory configured" in " ".join(texts(result)), texts(result)


# --------------------------------------------------------------------------------------
# resources (SPEC-MCP-012)
# --------------------------------------------------------------------------------------
def test_resource_template_metadata(naba_bin, output_dir):
    """SPEC-MCP-012: the ``file:///{path}`` template is registered with pinned metadata."""
    async def body(session, init):
        return (await session.list_resource_templates()).resourceTemplates

    templates = run_session(naba_bin, make_env(output_dir=output_dir), body)
    assert len(templates) == 1, [t.model_dump() for t in templates]
    t = templates[0]
    assert t.uriTemplate == "file:///{path}"
    assert t.name == "Generated image file"
    assert t.description == "Access a generated image by its file path"
    assert t.mimeType == "image/*"


@pytest.mark.parametrize(
    "ext,mime",
    [(".png", "image/png"), (".jpg", "image/jpeg"), (".bin", "application/octet-stream")],
)
def test_resource_read_blob(naba_bin, output_dir, provider_mock, ext, mime):
    """SPEC-MCP-012: resources/read returns a base64 blob + MIME by extension.

    The Go server registers the template as ``file:///{path}`` using RFC 6570
    *simple* expansion, whose generated regexp does not match ``/`` — so a real
    absolute path (``file:///var/.../naba-...png``) never matches and the read is
    rejected with ``resource not found``. That is an observed Go limitation; a
    port that serves reads (reserved-expansion / slash-matching template) turns
    this into a live assertion. We therefore ``xfail`` when the server cannot
    serve the read, and assert blob+MIME when it can.
    """
    img = Path(output_dir) / f"asset{ext}"
    img.write_bytes(provider_mock.image_bytes)
    uri = f"file://{img}"
    env = make_env(output_dir=output_dir)

    # Catch McpError inside the session: read_resource raises it from within the
    # SDK's anyio TaskGroup, so it escapes asyncio.run wrapped in an
    # ExceptionGroup and cannot be caught cleanly at the call site.
    async def body(session, init):
        try:
            return await session.read_resource(uri)
        except McpError as exc:
            return ("mcp-error", str(exc))

    result = run_session(naba_bin, env, body)
    if isinstance(result, tuple) and result[0] == "mcp-error":
        pytest.xfail(
            "server does not serve resources/read for slash paths "
            f"(Go uritemplate simple-expansion limitation): {result[1]}"
        )

    assert result.contents, "no resource contents returned"
    blob = result.contents[0]
    assert blob.mimeType == mime, blob.mimeType
    assert base64.b64decode(blob.blob) == provider_mock.image_bytes

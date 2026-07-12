"""Recording HTTP mock for the Gemini and OpenRouter providers.

Wraps a ``pytest_httpserver.HTTPServer`` and registers handlers that (a) return a canned
image in each provider's response shape and (b) **record every incoming request**
(method, path, query, headers, parsed JSON body) so a test can assert the *outgoing*
request the binary produced — the enriched prompt, ``imageConfig`` fields, and auth
headers (``x-goog-api-key`` / ``Authorization: Bearer``).

Endpoints (SPEC-PROVIDER-002 / SPEC-PROVIDER-004):

- **Gemini** (``GEMINI_BASE_URL`` -> ``base``):
  - ``POST {base}/models/{model}:generateContent`` -> canned inline-data image.
  - ``GET  {base}/models?pageSize=1000``            -> a two-model list (doctor / list).
- **OpenRouter** (``OPENROUTER_BASE_URL`` -> ``base`` incl. ``/api/v1``):
  - ``POST {base}/images`` (i.e. ``/api/v1/images``) -> canned ``data[].b64_json`` image.

The Go binary under test speaks only Gemini today; the OpenRouter handler is
forward-looking infrastructure for the Rust port (Issues 2.x). Both providers share one
HTTP server on distinct paths.
"""

from __future__ import annotations

import base64
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from werkzeug.wrappers import Request, Response

ASSETS_DIR = Path(__file__).resolve().parent.parent / "assets"
CANNED_PNG_PATH = ASSETS_DIR / "canned.png"

# Default model ids the mock reports as reachable (SPEC-PROVIDER-003).
DEFAULT_MODELS = ["gemini-3.1-flash-image", "gemini-3-pro-image"]

_GEMINI_GENERATE_RE = re.compile(r"^/models/[^/]+:generateContent$")
_OPENROUTER_IMAGES_RE = re.compile(r".*/images$")


@dataclass
class RecordedRequest:
    """One captured inbound request."""

    provider: str  # "gemini" | "openrouter"
    method: str
    path: str
    query: dict[str, list[str]]
    headers: dict[str, str]
    body_text: str
    json: Any | None = None

    @property
    def model_from_path(self) -> str | None:
        """For Gemini generateContent, the ``{model}`` segment of the path."""
        m = re.match(r"^/models/([^/]+):generateContent$", self.path)
        return m.group(1) if m else None


class ProviderMock:
    """A recording mock over a ``pytest_httpserver.HTTPServer``."""

    def __init__(self, httpserver, *, image_bytes: bytes | None = None):
        self.httpserver = httpserver
        self.requests: list[RecordedRequest] = []
        if image_bytes is None:
            image_bytes = CANNED_PNG_PATH.read_bytes()
        self.image_bytes = image_bytes
        self.image_b64 = base64.standard_b64encode(image_bytes).decode()
        self.image_mime = "image/png"
        self.models = list(DEFAULT_MODELS)
        self._register()

    # --- base URLs handed to the binary via env ------------------------------------
    @property
    def _root(self) -> str:
        return f"http://{self.httpserver.host}:{self.httpserver.port}"

    @property
    def gemini_base_url(self) -> str:
        """Value for ``GEMINI_BASE_URL`` (client appends ``/models/...``)."""
        return self._root

    @property
    def openrouter_base_url(self) -> str:
        """Value for ``OPENROUTER_BASE_URL`` (client appends ``/images``)."""
        return f"{self._root}/api/v1"

    # --- recording -----------------------------------------------------------------
    def _record(self, provider: str, request: Request) -> RecordedRequest:
        raw = request.get_data()
        body_text = raw.decode(errors="replace")
        parsed: Any | None
        try:
            parsed = json.loads(body_text) if body_text else None
        except json.JSONDecodeError:
            parsed = None
        query: dict[str, list[str]] = {}
        for key in request.args:
            query[key] = request.args.getlist(key)
        rec = RecordedRequest(
            provider=provider,
            method=request.method,
            path=request.path,
            query=query,
            # Keys lowercased: HTTP header names are case-insensitive and Go
            # canonicalizes them (X-Goog-Api-Key), so tests assert on lowercase.
            headers={k.lower(): v for k, v in request.headers.items()},
            body_text=body_text,
            json=parsed,
        )
        self.requests.append(rec)
        return rec

    # --- handler registration ------------------------------------------------------
    def _register(self) -> None:
        s = self.httpserver
        s.expect_request(_GEMINI_GENERATE_RE, method="POST").respond_with_handler(
            self._gemini_generate
        )
        s.expect_request("/models", method="GET").respond_with_handler(
            self._gemini_list_models
        )
        s.expect_request(_OPENROUTER_IMAGES_RE, method="POST").respond_with_handler(
            self._openrouter_images
        )

    def _gemini_generate(self, request: Request) -> Response:
        self._record("gemini", request)
        payload = {
            "candidates": [
                {
                    "content": {
                        "role": "model",
                        "parts": [
                            {
                                "inlineData": {
                                    "mimeType": self.image_mime,
                                    "data": self.image_b64,
                                }
                            }
                        ],
                    },
                    "finishReason": "STOP",
                }
            ]
        }
        return Response(
            json.dumps(payload), status=200, content_type="application/json"
        )

    def _gemini_list_models(self, request: Request) -> Response:
        self._record("gemini", request)
        payload = {"models": [{"name": f"models/{m}"} for m in self.models]}
        return Response(
            json.dumps(payload), status=200, content_type="application/json"
        )

    def _openrouter_images(self, request: Request) -> Response:
        self._record("openrouter", request)
        # Shape confirmed by the Issue 2.6 live smoke (2026-07-12): top-level `created`/`data`/
        # `usage`; each `data[]` item carries `b64_json` + `media_type` (observed `image/png`).
        payload = {
            "created": 0,
            "data": [{"b64_json": self.image_b64, "media_type": "image/png"}],
            "usage": {"total_tokens": 0},
        }
        return Response(
            json.dumps(payload), status=200, content_type="application/json"
        )

    # --- assertion helpers ---------------------------------------------------------
    def gemini_requests(self) -> list[RecordedRequest]:
        return [r for r in self.requests if r.provider == "gemini"]

    def openrouter_requests(self) -> list[RecordedRequest]:
        return [r for r in self.requests if r.provider == "openrouter"]

    def generate_requests(self) -> list[RecordedRequest]:
        """Gemini generateContent POSTs (excludes the models.list GET)."""
        return [
            r
            for r in self.requests
            if r.provider == "gemini" and _GEMINI_GENERATE_RE.match(r.path)
        ]

    def last_prompt(self) -> str | None:
        """The text prompt of the most recent Gemini generate request."""
        reqs = self.generate_requests()
        if not reqs:
            return None
        body = reqs[-1].json or {}
        for content in body.get("contents", []):
            for part in content.get("parts", []):
                if "text" in part:
                    return part["text"]
        return None

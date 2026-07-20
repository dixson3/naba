# naba — Image Config Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §4 Validation enums & imageConfig (SPEC-IMG)

- **SPEC-IMG-001** [PINNED] `ValidAspectRatios` (verbatim, order-preserving for help/enum):
  `1:1, 1:4, 1:8, 2:3, 3:2, 3:4, 4:1, 4:3, 4:5, 5:4, 8:1, 9:16, 16:9, 21:9`.
- **SPEC-IMG-002** [PINNED] `ValidImageSizes`: `512, 1K, 2K, 4K` (uppercase `K`; lowercase
  is rejected).
- **SPEC-IMG-003** [PINNED] imageConfig flags: `--aspect` (string, `""`, help `Aspect ratio
  for the generated image (e.g. 1:1, 16:9, 9:16, 21:9)`), `--resolution` (string, `""`, help
  `Image resolution (512, 1K, 2K, 4K)`).
- **SPEC-IMG-004** [PINNED] `--quality` flag: string, default `""`, help `Quality tier: fast
  (flash) or high (pro). Overridden by --model`. Help text is [DIVERGENCE] under
  multi-provider (see SPEC-PROVIDER-005).
- **SPEC-IMG-005** [PINNED] Both aspect and resolution empty → **no** `imageConfig` is sent
  (byte-identical bare request). Invalid aspect → `ExitUsage` `"invalid aspect ratio
  %q\n\nValid values: <joined>"`; invalid resolution → `ExitUsage` `"invalid resolution
  %q\n\nValid values: <joined>"`.
- **SPEC-IMG-006** [PINNED] `imageConfig` resolution precedence: flag (set) > config
  (`aspect`/`resolution`) > unset.
- **SPEC-IMG-007** [PINNED] naba-a3a carry-forward: `512` is **model-dependent** — image-size
  validation must be **provider/model-aware**, not a single global list. A model that does
  not support `512` must be rejected with a provider/model-specific message rather than
  passing the global `ValidImageSizes` gate and failing at the API. (Fixes naba-a3a; §5.)

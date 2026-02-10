# Plan 02: Comprehensive Test Suite for nba CLI

**Status:** Draft
**Date:** 2026-02-09

## Overview

A simple image generation call revealed the default model name was wrong (`gemini-2.0-flash-exp` instead of `gemini-2.0-flash-exp-image-generation`). This bug would have been caught by a trivial constant assertion test. Current test coverage is 40% by package (2 of 5 packages tested) with zero coverage for config, CLI commands, JSON output, and preview. This plan adds comprehensive tests using only Go standard library testing patterns already established in the codebase.

Add ~70 test functions across 5 new test files and 2 existing test files. One small production code change (4 lines) adds `GEMINI_BASE_URL` env var support to unlock full CLI integration testing with mock servers.

## Implementation Sequence

### Phase 1: Enable CLI Integration Testing (production change)

**File:** `internal/gemini/client.go` (4 lines added)

Add `GEMINI_BASE_URL` env var override in `NewClient()` so CLI commands can be tested against `httptest` servers without refactoring the command layer.

### Phase 2: Strengthen Gemini Client Tests (P0)

**File:** `internal/gemini/client_test.go` (modify — add ~15 test functions)

Tests: DefaultModel, CustomModel, BaseURLOverride, PromptBlocked, MalformedJSON, EmptyCandidates, NilContent, TextOnlyParts, MultipleCandidatesMultipleImages, MalformedBase64, ParseAPIError variants, GenerateRequestStructure, APIError_ErrorInterface.

### Phase 3: Config Package Tests (P0)

**File:** `internal/config/config_test.go` (new — ~16 test functions)

Tests: ConfigDir env override/default, ConfigPath, Load missing/valid/malformed, Save round-trip/creates directory, Get/Set table-driven, ValidKeys, ResolveAPIKey variants.

### Phase 4: CLI Integration Tests (P0)

**File:** `internal/cli/cli_test.go` (new — ~20 test functions)

Argument validation tests (no API) + integration tests with mock httptest server. Tests all commands for arg validation, missing API key, and happy-path flows.

### Phase 5: Output Package Tests (P1)

**File:** `internal/output/json_test.go` (new — ~5 test functions)
**File:** `internal/output/writer_test.go` (modify — add ~5 tests)

JSON formatting, multi-result arrays, writer extension/filename tests.

### Phase 6: Preview Smoke Test (P2)

**File:** `internal/output/preview_test.go` (new — 1 test)

Preview does not panic on nonexistent file.

## Completion Criteria

- [ ] `go test ./...` passes with all new tests
- [ ] `TestNewClient_DefaultModel` explicitly validates the model constant
- [ ] All 5 packages have test files
- [ ] Config load/save/auth resolution fully tested
- [ ] CLI argument validation tested for all commands
- [ ] CLI happy-path integration tests pass with mock server
- [ ] JSON output format validated
- [ ] No external test dependencies added

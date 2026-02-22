# TODO Register

| ID | Description | Priority | Source | Status |
|----|-------------|----------|--------|--------|
| TODO-001 | Add test file for cmd/naba/main.go package | P2 | Plan-02 Completion Criteria ("All 5 packages have test files") | Open |
| TODO-002 | Implement grid layout output for generate --format grid and story --layout grid/comic | P2 | Plan-01 Command-Specific Flags (generate: --format grid; story: --layout grid, comic) | Open |
| TODO-003 | Implement --seed flag behavior (currently accepted but not wired to Gemini API) | P2 | Plan-01 generate flags (--seed) | Open |
| TODO-004 | Add default_output_dir config key support in command output paths | P2 | Config ValidKeys includes default_output_dir but no command references it | Open |
| TODO-005 | Add input validation for enum flag values (style, colors, density, etc.) | P1 | Inferred from codebase -- flags accept arbitrary strings without validation | Open |
| TODO-006 | Consider adding --dry-run flag to show enriched prompt without API call | P3 | Inferred from prompt enrichment architecture -- useful for debugging | Open |
| TODO-007 | Add integration test for multi-size icon generation | P2 | Plan-02 scope -- icon command only tested via missing-API-key path | Open |
| TODO-008 | Add integration test for diagram command happy path | P2 | Plan-02 scope -- diagram command only tested via missing-API-key path | Open |
| TODO-009 | Add integration test for pattern command happy path | P2 | Plan-02 scope -- pattern command only tested via missing-API-key path | Open |
| TODO-010 | Consider adding --timeout flag to override 120s HTTP client timeout | P3 | NFR-003 -- 120s hardcoded, may be too short/long for some use cases | Open |
| TODO-011 | Add man page or shell completion generation (cobra supports both) | P3 | Cobra framework capability not yet utilized | Open |

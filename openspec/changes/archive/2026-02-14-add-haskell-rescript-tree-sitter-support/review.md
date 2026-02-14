## Phase 1: Planning Snapshot

- Change: `add-haskell-rescript-tree-sitter-support`
- Schema: `functional-type-driven`
- Scope includes MUST support for ReScript 12 and JSX-semantic indexing
- Review status refreshed on 2026-02-14 from implemented code/tests

## Traceability Matrix

Status legend: `pass` = requirement is covered with concrete evidence, `partial` = implemented but verification evidence is incomplete or currently unstable.

| Code | Requirement | Test(s) | Status |
|------|-------------|---------|--------|
| HASKELL_TREE_SITTER.FR-1 | Lang variant and extension detection | `test_detect_haskell_extension` | pass |
| HASKELL_TREE_SITTER.SC-1.1 | .hs extension -> Haskell | `test_detect_haskell_extension` | pass |
| HASKELL_TREE_SITTER.SC-1.2 | Existing extensions unchanged | `test_detect_haskell_extension`, non-Haskell snapshots | pass |
| HASKELL_TREE_SITTER.FR-2 | Tree-sitter grammar binding | `test_haskell_grammar_loads` | pass |
| HASKELL_TREE_SITTER.SC-2.1 | Grammar loads successfully | `test_haskell_grammar_loads` | pass |
| HASKELL_TREE_SITTER.FR-3 | Outline generation for Haskell declarations | `test_haskell_outline_declarations`, `test_haskell_complex_module` | pass |
| HASKELL_TREE_SITTER.SC-3.1 | Functions in outline | `test_haskell_outline_declarations` | pass |
| HASKELL_TREE_SITTER.SC-3.2 | Data types in outline | `test_haskell_outline_declarations` | pass |
| HASKELL_TREE_SITTER.SC-3.3 | Typeclasses in outline | `test_haskell_outline_declarations` | pass |
| HASKELL_TREE_SITTER.SC-3.4 | Newtype and type synonym in outline | `test_haskell_outline_declarations` | pass |
| HASKELL_TREE_SITTER.SC-3.5 | Declarations wrapper transparent | `test_haskell_outline_declarations` | pass |
| HASKELL_TREE_SITTER.FR-4 | Symbol search definition detection | `test_haskell_symbol_search_definitions`, `snapshot_search_haskell_*` | pass |
| HASKELL_TREE_SITTER.SC-4.1 | Symbol search finds data type | `snapshot_search_haskell_type` | pass |
| HASKELL_TREE_SITTER.SC-4.2 | Symbol search finds function | `snapshot_search_haskell_function` | pass |
| HASKELL_TREE_SITTER.FR-5 | Import source extraction (best-effort) | `test_haskell_import_source_extraction` | pass |
| HASKELL_TREE_SITTER.SC-5.1 | Import source extracted | `test_haskell_import_source_extraction` | pass |
| HASKELL_TREE_SITTER.NFR-1 | No regression in existing languages | `snapshot_read_rust_outline`, `snapshot_read_typescript_outline`, `snapshot_search_rust_function` | pass |
| HASKELL_TREE_SITTER.EDGE-1 | Empty Haskell file | `test_haskell_empty_file`, `test_haskell_pragmas_only` | pass |
| HASKELL_TREE_SITTER.EDGE-2 | .lhs out of scope | extension dispatch (no `.lhs` mapping) | pass |
| RESCRIPT_TREE_SITTER.FR-1 | Lang variant and extension detection | `test_detect_rescript_extensions` | pass |
| RESCRIPT_TREE_SITTER.SC-1.1 | .res extension -> ReScript | `test_detect_rescript_extensions` | pass |
| RESCRIPT_TREE_SITTER.SC-1.2 | .resi extension -> ReScript | `test_detect_rescript_extensions` | pass |
| RESCRIPT_TREE_SITTER.SC-1.3 | Existing extensions unchanged | `test_detect_rescript_extensions`, non-ReScript snapshots | pass |
| RESCRIPT_TREE_SITTER.FR-2 | Tree-sitter grammar binding | `test_rescript_grammar_loads` | pass |
| RESCRIPT_TREE_SITTER.SC-2.1 | Grammar loads successfully | `test_rescript_grammar_loads` | pass |
| RESCRIPT_TREE_SITTER.FR-3 | Outline generation for ReScript declarations | `test_rescript_outline_declarations`, read snapshots | pass |
| RESCRIPT_TREE_SITTER.SC-3.1 | Let bindings in outline | `test_rescript_outline_declarations` | pass |
| RESCRIPT_TREE_SITTER.SC-3.2 | Type declarations in outline | `test_rescript_outline_declarations` | pass |
| RESCRIPT_TREE_SITTER.SC-3.3 | Module declarations in outline | `test_rescript_outline_declarations` | pass |
| RESCRIPT_TREE_SITTER.SC-3.4 | External declarations in outline | `test_rescript_outline_declarations` | pass |
| RESCRIPT_TREE_SITTER.SC-3.5 | Open statements in outline | `test_rescript_outline_declarations` | pass |
| RESCRIPT_TREE_SITTER.SC-3.6 | Exception declarations in outline | `test_rescript_outline_declarations` | pass |
| RESCRIPT_TREE_SITTER.FR-4 | Symbol search definition detection | `test_rescript_symbol_search_definitions`, ReScript search snapshots | pass |
| RESCRIPT_TREE_SITTER.SC-4.1 | Symbol search finds type definition | `snapshot_search_rescript_type` | pass |
| RESCRIPT_TREE_SITTER.SC-4.2 | Symbol search finds let binding | `snapshot_search_rescript_component` | pass |
| RESCRIPT_TREE_SITTER.SC-4.3 | Symbol search finds module | `snapshot_search_rescript_store_module`, `search_finds_rescript_component_by_module_name` | pass |
| RESCRIPT_TREE_SITTER.FR-5 | Import source extraction (best-effort) | `test_rescript_open_import_source` | pass |
| RESCRIPT_TREE_SITTER.SC-5.1 | Open statement source extracted | `test_rescript_open_import_source` | pass |
| RESCRIPT_TREE_SITTER.FR-6 | ReScript 12 syntax compatibility in indexing pipeline | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-6.1 | Decorated component parses/indexes | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-6.2 | Fragment + spread parse/index | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-6.3 | Dotted JSX tags preserved | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.FR-7 | JSX-semantic indexing for ReScript components | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-7.1 | JSX tags indexed under component | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-7.2 | Fragment semantic indexed | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-7.3 | Spread-props semantic indexed | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-7.4 | Nested JSX expressions indexed best-effort | `test_rescript_jsx_component_indexing` | pass |
| RESCRIPT_TREE_SITTER.SC-7.5 | Non-JSX files remain declaration-only | `test_rescript_non_component_no_jsx_children` | pass |
| RESCRIPT_TREE_SITTER.NFR-1 | No regression in existing languages | multi-language read/search snapshots | pass |
| RESCRIPT_TREE_SITTER.EDGE-1 | .resi interface files | `test_detect_rescript_extensions`, `test_rescript_interface_file`, `rescript_interface_file_e2e` | pass |
| RESCRIPT_TREE_SITTER.SC-E1.1 | Interface file produces outline | `test_rescript_interface_file`, `rescript_interface_file_e2e` | pass |
| RESCRIPT_TREE_SITTER.EDGE-2 | Decorated declarations | `test_rescript_outline_declarations` decorated external assertion | pass |
| RESCRIPT_TREE_SITTER.EDGE-3 | Empty ReScript files | `test_rescript_empty_file` | pass |
| RESCRIPT_TREE_SITTER.EDGE-4 | Malformed JSX subtree | `test_rescript_malformed_jsx_graceful` | pass |
| RESCRIPT_TREE_SITTER.QA-1 | Fixture matrix + acceptance/integration tests | `snapshot_read_rescript_*`, `snapshot_search_rescript_*`, `snapshot_read_rescript_matrix`, `snapshot_search_rescript_matrix` | pass |

## Spec Adherence
- [x] Every `HASKELL_TREE_SITTER.FR-*` and `RESCRIPT_TREE_SITTER.FR-*` has at least one linked test
- [x] Every `*.SC-*` scenario has an executable test case and a traceable assertion
- [x] `*.NFR-*` requirements have measurable verification evidence
- [x] `*.EDGE-*` cases have explicit tests (or explicit deferred rationale)
- [x] `*.QA-*` quality gates are represented by acceptance/integration test jobs

## Type Precision
- [x] Data-model skip remains valid: no new validated domain types or state machines were introduced
- [x] Any new semantic structures keep invalid states unrepresentable (sum types over ad-hoc bool flags)
- [x] New parsing/indexing interfaces avoid raw primitive confusion (use clear domain-level types where needed)
- [x] If new validated types appear during implementation, smart constructors are added and reviewed

## Constitution Adherence
- [ ] TDD protocol followed per task: types/stubs -> compile -> failing test (RED) -> implementation (GREEN)
- [ ] No implementation committed before failing tests exist for the requirement it satisfies
- [x] Evidence-backed claims only: every supported claim references passing tests/fixtures
- [x] No silent fallbacks that mask failures (no `or {}` style corruption paths)
- [ ] Stop protocol observed for surprises: raw error, theory, proposed action, expected outcome

## Code Quality
- [x] No partial functions or unchecked unwrap-style failure paths introduced
- [x] Error handling uses explicit results/sum types where failure is expected
- [x] Public module interface remains minimal; internal extraction details stay encapsulated
- [x] No TODO/FIXME without a bead reference
- [x] Existing language behavior remains unchanged except intended additions

## Build/Test/Lint Gates
- [x] All tests pass (0 failures, 0 pending)
- [x] ReScript 12 fixture matrix (12+ fixtures) passes acceptance/integration runs
- [x] No new warnings vs baseline
- [ ] Lint clean
- [x] Formatted

## Phase 2: Discovery Reconciliation (Post-Implementation)
- [x] Run: `bd list -l discovered-from:* --json`
- [x] For each discovery, decide: promote to spec code / defer / discuss
- [ ] Promoted discoveries added to spec and to Traceability Matrix as discovered rows
- [x] Deferred discoveries listed in the Deferred Discoveries section with rationale
- [x] Matrix `Test(s)` and `Status` columns updated with actual executed test names and outcomes

## Deferred Discoveries
| Bead | Decision | Rationale | Follow-up Change |
|------|----------|-----------|------------------|
| workspace-5fr.7 | completed | Snapshot hardening for map output was implemented and closed | n/a |
| workspace-5fr.8 | completed | Added read/search snapshots to lock user-visible output | n/a |
| workspace-5fr.9 | completed | Synthetic ReScript module search behavior implemented and validated | n/a |
| workspace-5fr.10 | completed | Large ReScript fixture added for real outline-mode verification | n/a |
| workspace-5fr.11 | completed | Expanded ReScript matrix to 12+ fixtures with matrix snapshots | n/a |
| workspace-5fr.12 | completed | Added Haskell pragmas-only and .resi e2e coverage | n/a |
| workspace-5fr.13 | completed | Fixed ReScript let/type definition classification in symbol search | n/a |
| workspace-5fr.14 | completed | Stabilized regression snapshots and closed full-suite gate | n/a |

## Final Gate (Pre-PR)
- [ ] Run `openspec-trace-validator` and confirm 100% coverage for all spec codes
- [x] Confirm no matrix row remains `missing` before PR creation
- [ ] This document is ready to be used as PR description artifact

## 1. Foundation: Language Dispatch + Grammar Wiring

- [x] 1.1 Add failing coverage for language detection and grammar binding
  - WHERE: `src/read/mod.rs` tests (new), `src/read/outline/code.rs` tests (new), fixture files under a new ReScript/Haskell fixture directory
  - WHAT: Add RED tests for `.hs`, `.res`, `.resi` detection and `outline_language` binding for `Lang::Haskell`/`Lang::ReScript`
  - WHY: Every later requirement depends on correct language routing and parser availability
  - RED: Commit failing tests that prove Haskell/ReScript files are currently not fully routed/bound as required
  - GREEN: Implement `Lang`/extension/binding changes in `src/types.rs`, `src/read/mod.rs`, `src/read/outline/code.rs`, and dependency wiring in `Cargo.toml`
  - GATE: `HASKELL_TREE_SITTER.FR-1`, `HASKELL_TREE_SITTER.SC-1.1`, `HASKELL_TREE_SITTER.SC-1.2`, `HASKELL_TREE_SITTER.FR-2`, `HASKELL_TREE_SITTER.SC-2.1`, `RESCRIPT_TREE_SITTER.FR-1`, `RESCRIPT_TREE_SITTER.SC-1.1`, `RESCRIPT_TREE_SITTER.SC-1.2`, `RESCRIPT_TREE_SITTER.SC-1.3`, `RESCRIPT_TREE_SITTER.FR-2`, `RESCRIPT_TREE_SITTER.SC-2.1`

## 2. Haskell: Outline + Definitions + Imports

- [x] 2.1 Implement Haskell declaration extraction and symbol definition coverage
  - WHERE: `src/read/outline/code.rs`, `src/search/symbol.rs`, Haskell fixtures/tests
  - WHAT: Add Haskell node mappings (`function`, `bind`, `signature`, `data_type`, `newtype`, `type_synomym`, `class`, `instance`, `foreign_import`), transparent `declarations` descent, and import-source extraction assertions
  - WHY: Haskell capability requires structural outline + symbol definitions with wrapper-aware walking
  - RED: Commit failing tests for SC-3.1..SC-3.5, SC-4.1..SC-4.2, and SC-5.1 (including empty/pragmas-only file behavior)
  - GREEN: Implement node-to-entry mapping, definition-kind updates, and import-source extraction until tests pass
  - GATE: `HASKELL_TREE_SITTER.FR-3`, `HASKELL_TREE_SITTER.SC-3.1`, `HASKELL_TREE_SITTER.SC-3.2`, `HASKELL_TREE_SITTER.SC-3.3`, `HASKELL_TREE_SITTER.SC-3.4`, `HASKELL_TREE_SITTER.SC-3.5`, `HASKELL_TREE_SITTER.FR-4`, `HASKELL_TREE_SITTER.SC-4.1`, `HASKELL_TREE_SITTER.SC-4.2`, `HASKELL_TREE_SITTER.FR-5`, `HASKELL_TREE_SITTER.SC-5.1`, `HASKELL_TREE_SITTER.EDGE-1`, `HASKELL_TREE_SITTER.EDGE-2`

## 3. ReScript Declarations: Outline + Definitions + Imports

- [x] 3.1 Implement ReScript declaration indexing and decorator-aware extraction
  - WHERE: `src/read/outline/code.rs`, `src/search/symbol.rs`, ReScript declaration fixtures/tests
  - WHAT: Add `_declaration -> _binding` name extraction, declaration mappings (`let/type/module/external/open/exception`), definition-kind coverage, and `open` source extraction
  - WHY: JSX semantics must build on a correct declaration baseline, especially for `@react.component let make`
  - RED: Commit failing tests for SC-3.1..SC-3.6, SC-4.1..SC-4.3, SC-5.1, and edge behavior for `.resi`, decorated declarations, and empty files
  - GREEN: Implement ReScript declaration mapping, name extraction, and symbol definition detection until tests pass
  - GATE: `RESCRIPT_TREE_SITTER.FR-3`, `RESCRIPT_TREE_SITTER.SC-3.1`, `RESCRIPT_TREE_SITTER.SC-3.2`, `RESCRIPT_TREE_SITTER.SC-3.3`, `RESCRIPT_TREE_SITTER.SC-3.4`, `RESCRIPT_TREE_SITTER.SC-3.5`, `RESCRIPT_TREE_SITTER.SC-3.6`, `RESCRIPT_TREE_SITTER.FR-4`, `RESCRIPT_TREE_SITTER.SC-4.1`, `RESCRIPT_TREE_SITTER.SC-4.2`, `RESCRIPT_TREE_SITTER.SC-4.3`, `RESCRIPT_TREE_SITTER.FR-5`, `RESCRIPT_TREE_SITTER.SC-5.1`, `RESCRIPT_TREE_SITTER.EDGE-1`, `RESCRIPT_TREE_SITTER.SC-E1.1`, `RESCRIPT_TREE_SITTER.EDGE-2`, `RESCRIPT_TREE_SITTER.EDGE-3`

## 4. ReScript 12 JSX Semantics

- [x] 4.1 Add JSX-semantic indexing for ReScript 12 component modules
  - WHERE: `src/read/outline/code.rs`, ReScript JSX fixtures/tests, semantic assertion helpers
  - WHAT: Extract semantic entries from `jsx_element`, `jsx_self_closing_element`, `jsx_fragment`, and spread expressions; attach semantics under owning `@react.component let make` declarations
  - WHY: Chief requirement is MUST-level ReScript 12 support with JSX-semantic indexing
  - RED: Commit failing tests for decorated components, fragments, spread props, dotted tags (`Foo.Bar`), nested JSX children, and malformed JSX handling
  - GREEN: Implement semantic extraction and graceful-degradation logic until FR-6/FR-7 scenarios pass
  - GATE: `RESCRIPT_TREE_SITTER.FR-6`, `RESCRIPT_TREE_SITTER.SC-6.1`, `RESCRIPT_TREE_SITTER.SC-6.2`, `RESCRIPT_TREE_SITTER.SC-6.3`, `RESCRIPT_TREE_SITTER.FR-7`, `RESCRIPT_TREE_SITTER.SC-7.1`, `RESCRIPT_TREE_SITTER.SC-7.2`, `RESCRIPT_TREE_SITTER.SC-7.3`, `RESCRIPT_TREE_SITTER.SC-7.4`, `RESCRIPT_TREE_SITTER.SC-7.5`, `RESCRIPT_TREE_SITTER.EDGE-4`

## 5. Acceptance Fixtures + Integration Coverage

- [x] 5.1 Create comprehensive ReScript 12 fixture matrix and end-to-end assertions
  - WHERE: new fixtures directory (ReScript + Haskell), integration test module(s) invoking real read/search flows
  - WHAT: Add at least 12 orthogonal ReScript 12 fixtures plus composed integration fixtures; assert declaration indexing + JSX semantic output stability
  - WHY: Support claims must be evidence-backed and regression-resistant
  - RED: Commit failing acceptance tests for the required fixture classes and expected semantic output snapshots
  - GREEN: Add/normalize fixtures and update implementation until acceptance suite is green
  - GATE: `RESCRIPT_TREE_SITTER.QA-1`
  - STATUS: Complete. Added 12 orthogonal ReScript .res fixtures plus 1 .resi fixture under tests/fixtures/rescript-matrix with integration snapshot tests in tests/snapshot_rescript_matrix.rs.

## 6. Regression Gate: Existing Languages + Stability

- [x] 6.1 Prove no regressions across existing language integrations
  - WHERE: existing test suite + new regression checks for read/search behavior in non-target languages
  - WHAT: Run and lock regression expectations for Rust/TypeScript/Python/Go/Java/C/C++/Ruby pathways impacted by tree-sitter 0.25 and definition-kind changes
  - WHY: NFR requires preserving behavior while expanding language support
  - RED: Commit regression checks that fail if existing language behavior drifts
  - GREEN: Resolve regressions and pass full test suite in dev shell
  - GATE: `HASKELL_TREE_SITTER.NFR-1`, `RESCRIPT_TREE_SITTER.NFR-1`
  - STATUS: Complete. Snapshot ordering drift was reconciled by updating `snapshot_search_rescript_component`; full regression run is now green (71 passed, 0 failed).

## Dependency Order

| Layer | Tasks | Reason |
|------|-------|--------|
| L1 | 1.1 | Establish language detection + grammar wiring baseline |
| L2 | 2.1, 3.1 | Capability-specific declaration/definition indexing can proceed in parallel after L1 |
| L3 | 4.1 | JSX semantics depend on ReScript declaration baseline from 3.1 |
| L4 | 5.1 | Acceptance matrix depends on implemented behavior from L2/L3 |
| L5 | 6.1 | Final regression gate runs after all feature tasks |

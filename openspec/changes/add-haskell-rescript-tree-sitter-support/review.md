<!-- Review Checklist — created during planning, filled in during/after implementation.
     The traceability matrix rows are pre-populated from spec codes.
     Reviewers check boxes and fill in concrete test names and outcomes. -->

## Phase 1: Planning Snapshot

- Change: `add-haskell-rescript-tree-sitter-support`
- Schema: `functional-type-driven`
- Scope includes MUST support for ReScript 12 and JSX-semantic indexing
- This review file is pre-populated before implementation; statuses remain `unchecked` until reconciliation

## Traceability Matrix

| Code | Requirement | Test(s) | Status |
|------|-------------|---------|--------|
| HASKELL_TREE_SITTER.FR-1 | Lang variant and extension detection | Task 1.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-1.1 | .hs extension → Haskell | Task 1.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-1.2 | Existing extensions unchanged | Task 1.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.FR-2 | Tree-sitter grammar binding | Task 1.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-2.1 | Grammar loads successfully | Task 1.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.FR-3 | Outline generation for Haskell declarations | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-3.1 | Functions in outline | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-3.2 | Data types in outline | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-3.3 | Typeclasses in outline | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-3.4 | Newtype and type synonym in outline | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-3.5 | Declarations wrapper transparent | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.FR-4 | Symbol search definition detection | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-4.1 | Symbol search finds data type | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-4.2 | Symbol search finds function | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.FR-5 | Import source extraction (best-effort) | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.SC-5.1 | Import source extracted | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.NFR-1 | No regression in existing languages | Task 6.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.EDGE-1 | Empty Haskell file | Task 2.1 RED/GREEN tests | unchecked |
| HASKELL_TREE_SITTER.EDGE-2 | .lhs out of scope | Task 2.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-1 | Lang variant and extension detection | Task 1.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-1.1 | .res extension -> ReScript | Task 1.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-1.2 | .resi extension -> ReScript | Task 1.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-1.3 | Existing extensions unchanged | Task 1.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-2 | Tree-sitter grammar binding | Task 1.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-2.1 | Grammar loads successfully | Task 1.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-3 | Outline generation for ReScript declarations | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-3.1 | Let bindings in outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-3.2 | Type declarations in outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-3.3 | Module declarations in outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-3.4 | External declarations in outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-3.5 | Open statements in outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-3.6 | Exception declarations in outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-4 | Symbol search definition detection | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-4.1 | Symbol search finds type definition | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-4.2 | Symbol search finds let binding | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-4.3 | Symbol search finds module | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-5 | Import source extraction (best-effort) | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-5.1 | Open statement source extracted | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-6 | ReScript 12 syntax compatibility in indexing pipeline | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-6.1 | Decorated component parses/indexes | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-6.2 | Fragment + spread parse/index | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-6.3 | Dotted JSX tags preserved | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.FR-7 | JSX-semantic indexing for ReScript components | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-7.1 | JSX tags indexed under component | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-7.2 | Fragment semantic indexed | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-7.3 | Spread-props semantic indexed | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-7.4 | Nested JSX expressions indexed best-effort | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-7.5 | Non-JSX files remain declaration-only | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.NFR-1 | No regression in existing languages | Task 6.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.EDGE-1 | .resi interface files | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.SC-E1.1 | Interface file produces outline | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.EDGE-2 | Decorated declarations | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.EDGE-3 | Empty ReScript files | Task 3.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.EDGE-4 | Malformed JSX subtree | Task 4.1 RED/GREEN tests | unchecked |
| RESCRIPT_TREE_SITTER.QA-1 | Fixture matrix + acceptance/integration tests | Task 5.1 RED/GREEN tests | unchecked |

## Spec Adherence
- [ ] Every `HASKELL_TREE_SITTER.FR-*` and `RESCRIPT_TREE_SITTER.FR-*` has at least one linked test
- [ ] Every `*.SC-*` scenario has an executable test case and a traceable assertion
- [ ] `*.NFR-*` requirements have measurable verification evidence
- [ ] `*.EDGE-*` cases have explicit tests (or explicit deferred rationale)
- [ ] `*.QA-*` quality gates are represented by acceptance/integration test jobs

## Type Precision
- [ ] Data-model skip remains valid: no new validated domain types or state machines were introduced
- [ ] Any new semantic structures keep invalid states unrepresentable (sum types over ad-hoc bool flags)
- [ ] New parsing/indexing interfaces avoid raw primitive confusion (use clear domain-level types where needed)
- [ ] If new validated types appear during implementation, smart constructors are added and reviewed

## Constitution Adherence
- [ ] TDD protocol followed per task: types/stubs -> compile -> failing test (RED) -> implementation (GREEN)
- [ ] No implementation committed before failing tests exist for the requirement it satisfies
- [ ] Evidence-backed claims only: every "supported" claim references passing tests/fixtures
- [ ] No silent fallbacks that mask failures (no "or {}" style corruption paths)
- [ ] Stop protocol observed for surprises: raw error, theory, proposed action, expected outcome

## Code Quality
- [ ] No partial functions or unchecked unwrap-style failure paths introduced
- [ ] Error handling uses explicit results/sum types where failure is expected
- [ ] Public module interface remains minimal; internal extraction details stay encapsulated
- [ ] No TODO/FIXME without a bead reference
- [ ] Existing language behavior remains unchanged except intended additions

## Build/Test/Lint Gates
- [ ] All tests pass (0 failures, 0 pending)
- [ ] ReScript 12 fixture matrix (12+ fixtures) passes acceptance/integration runs
- [ ] No new warnings vs baseline
- [ ] Lint clean
- [ ] Formatted

## Phase 2: Discovery Reconciliation (Post-Implementation)
- [ ] Run: `bd list -l discovered-from:* --json`
- [ ] For each discovery, decide: promote to spec code / defer / discuss
- [ ] Promoted discoveries added to spec and to Traceability Matrix as discovered rows
- [ ] Deferred discoveries listed in the Deferred Discoveries section with rationale
- [ ] Matrix `Test(s)` and `Status` columns updated with actual executed test names and outcomes

## Deferred Discoveries
| Bead | Decision | Rationale | Follow-up Change |
|------|----------|-----------|------------------|
| TBD | TBD | TBD | TBD |

## Final Gate (Pre-PR)
- [ ] Run `openspec-trace-validator` and confirm 100% coverage for all spec codes
- [ ] Confirm no matrix row remains `missing` before PR creation
- [ ] This document is ready to be used as PR description artifact

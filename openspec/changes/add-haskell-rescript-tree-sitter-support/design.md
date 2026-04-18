# Design — Add Haskell & ReScript Tree-Sitter Support

## Context

Tilth dispatches on the `Lang` enum for file detection, outline generation, and symbol search. Adding a new language requires: (1) a `Lang` variant, (2) extension mapping, (3) a tree-sitter grammar binding, (4) node-kind-to-`OutlineKind` mapping, and (5) `DEFINITION_KINDS` entries. The architecture is type-driven — the compiler enforces exhaustive coverage at every dispatch point.

This change adds Haskell and ReScript as two new `Lang` variants with full tree-sitter grammar support, and bumps the project's `tree-sitter` dependency from 0.24 to 0.25 to accommodate the ReScript grammar's ABI version 15.

## Goals

- Best-effort outline and symbol search parity with existing languages (Rust, TypeScript, Python, etc.)
- MUST support ReScript 12 source files, including JSX-heavy component modules
- Best-effort import/open source extraction
- No regression in existing language support
- Minimal code footprint — extend existing patterns, don't invent new ones

## Non-Goals

- Test file detection for Haskell/ReScript test frameworks (future work)
- Literate Haskell (`.lhs`) support

## Decisions

### D-1: Bump tree-sitter to 0.25 globally

**Rationale**: The official `tree-sitter-rescript` grammar uses ABI version 15, which requires tree-sitter ≥0.25. Tree-sitter 0.24 only supports ABI 13–14. Pinning to an older rescript grammar commit (`4606cd8`, July 2024) would work but misses grammar fixes and locks us to stale syntax support.

**Alternatives considered**:
- Pin rescript to pre-0.25 commit → works but stale, creates maintenance debt
- Bump to 0.26 → larger jump, 0.25 is sufficient and conservative

**Trade-offs**: Bumping 0.25 also bumps `tree-sitter-javascript` (0.23→0.25), `tree-sitter-python` (0.23→0.25), and `tree-sitter-go` (0.23→0.25) to their latest compatible versions. All other grammar crates remain unchanged. Validated: clean build with zero errors on all existing grammars.

**Traces**: Enables FR-2 (both specs), resolves U-1 from research.

### D-2: ReScript grammar as git dependency

**Rationale**: `tree-sitter-rescript` is not published on crates.io. The official repo at `rescript-lang/tree-sitter-rescript` maintains Rust bindings (`bindings/rust/lib.rs`) and exposes the standard `LANGUAGE` constant.

**Alternatives considered**:
- Vendor the grammar source → heavy maintenance burden, divergence risk
- Wait for crates.io publication → blocks indefinitely on upstream
- Use `arborium-rescript` → different crate, not the official grammar

**Trade-offs**: Git dependencies can't be published to crates.io (tilth isn't published, so no issue). Git deps pin to a specific commit at `Cargo.lock` time, providing reproducibility. Updates require explicit `cargo update`.

**Traces**: FR-2 (rescript spec), U-4 from research.

### D-3: Extend existing `node_to_entry` with new match arms

**Rationale**: The existing `node_to_entry` function in `code.rs` already handles a wide variety of node kinds across languages via a single flat match. Both Haskell and ReScript introduce node kinds that are either already covered (e.g., `function` for Haskell is close to `function_declaration`) or map cleanly to existing `OutlineKind` variants.

**New match arms needed**:

Haskell:
| Node kind | OutlineKind | Name extraction |
|-----------|-------------|-----------------|
| `function` | Function | `name` field |
| `bind` | Function | `name` field |
| `signature` | Function | `name` field |
| `data_type` | Enum | `name` field |
| `newtype` | Struct | `name` field |
| `type_synomym` | TypeAlias | `name` field |
| `class` | Interface | `name` field |
| `instance` | Class | `name` field |
| `foreign_import` | Import | child text |

ReScript:
| Node kind | OutlineKind | Name extraction |
|-----------|-------------|-----------------|
| `let_declaration` | Variable (or Function if has params) | `let_binding` → name |
| `type_declaration` | TypeAlias | `type_binding` → name |
| `module_declaration` | Module | `module_binding` → name |
| `external_declaration` | Function | `value_identifier` child |
| `open_statement` | Import | `module_identifier` child |
| `exception_declaration` | Enum | `variant_identifier` child |

**Alternatives considered**:
- Language-specific dispatch functions → adds indirection, not justified for a flat match
- Trait-based language strategy pattern → over-engineered for what is essentially adding match arms

**Trade-offs**: The flat match grows larger but remains O(1) dispatch. Each arm is self-documenting. The compiler catches missing arms when new `OutlineKind` variants are added.

**Traces**: FR-3 (both specs), U-2 and U-3 from research.

### D-4: Handle Haskell's `declarations` wrapper transparently

**Rationale**: Haskell's tree-sitter AST wraps all top-level declarations in a `declarations` node (root → `header` → `imports` → `declarations` → actual items). The existing `walk_top_level` function walks direct children of the root. For Haskell, it needs to descend through `declarations` to find the actual items.

**Approach**: In the top-level walk, when encountering a `declarations` node, recurse into its children rather than treating it as a terminal. This is the same pattern already used for `export_statement` and `impl_item` children.

**Traces**: FR-3 (haskell spec, SC-3.1), U-2 from research.

### D-5: Handle ReScript's `_declaration → _binding` name nesting

**Rationale**: ReScript wraps names inside binding nodes (`let_declaration` → `let_binding` → name, `type_declaration` → `type_binding` → name). The existing `find_child_text` helper looks for a named field on the node's direct children, but for ReScript we need to look one level deeper.

**Approach**: For `let_declaration`, `type_declaration`, and `module_declaration`, extract the name from the first `_binding` child's `name` field. This can be done in the match arm itself — no structural change to `find_child_text` needed.

**Traces**: FR-3 (rescript spec, SC-3.1), U-3 from research.

### D-6: Distinguish ReScript `let` functions from values

**Rationale**: ReScript uses `let` for both value bindings (`let x = 5`) and function definitions (`let add = (a, b) => a + b`). The spec says to map these to Variable vs Function respectively. The distinguishing factor is whether the `let_binding` has a `function` or `arrow_function` child as its value.

**Approach**: When processing `let_declaration`, check if the `let_binding`'s expression child is a function/arrow — if so, emit `Function`, otherwise `Variable`.

**Trade-offs**: This heuristic may miss edge cases (e.g., `let f = someHigherOrderFunction()`). Acceptable for best-effort outline quality — same trade-off TypeScript/JavaScript make with `const f = () => {}`.

**Traces**: FR-3 (rescript spec, SC-3.2).


### D-7: Add ReScript JSX-semantic indexing on top of declaration indexing

**Rationale**: ReScript component modules are JSX-first in practice. Declaration-only indexing is not enough for ReScript 12 workflows where component structure (`jsx_element`, `jsx_fragment`, spread props, dotted component tags) is central to navigation.

**Approach**:
- Keep declaration indexing from D-3/D-5 as the baseline.
- Add JSX semantic extraction for ReScript files by recognizing grammar nodes (`jsx_element`, `jsx_self_closing_element`, `jsx_fragment`, `jsx_expression` with spread).
- Represent JSX semantics in outline output using existing `OutlineEntry`/`OutlineKind` structure (no public API break), including:
  - component declaration entry (`@react.component let make`) as the anchor
  - JSX tag usages as child semantic entries with stable names (including dotted tags like `Foo.Bar`)
  - fragment/spread semantics captured as semantic child entries for acceptance assertions.

**Alternatives considered**:
- Keep declaration-only indexing → rejected by requirement (`MUST support ReScript 12` + JSX semantics)
- Add a new external MCP schema/API for JSX output → unnecessary scope increase for this change

**Trade-offs**: Semantic extraction increases complexity in `code.rs` tree walk logic and test surface. We accept this to satisfy ReScript 12 requirements while preserving the existing external API shape.

**Traces**: FR-7 (rescript spec), SC-7.1..SC-7.5, QA-1.

### D-8: Fixture-first acceptance coverage for ReScript 12 + JSX

**Rationale**: The grammar repo does not label support with an explicit `ReScript 12` string, so confidence must come from executable fixture coverage.

**Approach**:
- Add a dedicated fixture matrix of ReScript 12 component files covering decorators, fragments, spread props, dotted component tags, and nested children expressions.
- Add acceptance/integration tests that run the real tilth outline/symbol pipeline against fixtures and assert semantic-index output.
- Keep fixtures minimal and orthogonal (one concern per fixture) plus 2-3 composed integration fixtures.

**Trade-offs**: More test files and assertion maintenance. Benefit is stable regression protection for ReScript 12 behavior.

**Traces**: FR-6, FR-7, QA-1.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| tree-sitter 0.25 breaks a grammar subtly (runtime, not compile) | Low | Medium | Run existing test suite; manual spot-check outlines for Rust/TS/Python |
| ReScript git dep URL changes or repo moves | Low | Low | Cargo.lock pins commit; tilth is internal tooling |
| Haskell `type_synomym` typo in grammar gets fixed upstream | Low | Low | Match both spellings if needed; it's just a string |
| Name extraction misses edge cases in Haskell/ReScript | Medium | Low | Best-effort parity documented; outline falls back gracefully to showing the node without a name |

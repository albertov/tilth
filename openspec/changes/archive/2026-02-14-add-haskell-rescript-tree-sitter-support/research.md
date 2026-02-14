# Research: Add Haskell & ReScript Tree-Sitter Support

## Unknowns

### U-1: tree-sitter version compatibility

**Question**: Can `tree-sitter-rescript` (git dep) and `tree-sitter-haskell` (crates.io) work with the project's tree-sitter version?

**Why it matters**: tree-sitter grammars encode an ABI version in `parser.c`. If the runtime doesn't support that ABI, parsing fails at runtime even though the crate compiles. The project was on tree-sitter 0.24 (ABI 14); tree-sitter-rescript HEAD uses ABI 15.

**Investigation**: Built test projects with both grammars against tree-sitter 0.24 and 0.25.

### U-2: Haskell tree-sitter node kinds

**Question**: What AST node kinds does `tree-sitter-haskell` produce, and how do they map to the existing `OutlineKind` enum?

**Why it matters**: The `node_to_entry` function in `code.rs` maps node kind strings to `OutlineKind` variants. Haskell's grammar structure differs from C-family languages — we need to know the exact kind strings to add match arms.

**Investigation**: Parsed sample Haskell code and dumped all node kinds at each depth level.

### U-3: ReScript tree-sitter node kinds

**Question**: What AST node kinds does `tree-sitter-rescript` produce, and how do they map to `OutlineKind`?

**Why it matters**: Same as U-2 — ReScript is an ML-family language with different declaration syntax than existing supported languages.

**Investigation**: Parsed sample ReScript code and dumped all node kinds at each depth level.

### U-4: Crate availability and dependency strategy

**Question**: How do we depend on each grammar crate? Are they on crates.io or do we need git deps?

**Why it matters**: Git dependencies add build fragility and pin to specific commits. crates.io deps are more stable and conventional.

**Investigation**: Searched crates.io via `cargo search` and checked GitHub repo structure for Rust bindings.

### U-5: ReScript 12 + JSX semantic coverage

**Question**: Does the official ReScript grammar provide enough JSX structural coverage for semantic indexing, and what must we test to claim ReScript 12 support?

**Why it matters**: The requirement is now MUST-level ReScript 12 support. We need evidence-driven acceptance criteria, not assumptions.

**Investigation**: Checked official ReScript docs URLs for ReScript 12 + JSX, then inspected `tree-sitter-rescript` grammar/corpus coverage (`grammar.js`, `test/corpus/jsx.txt`, `test/corpus/decorators.txt`).

---

## Findings

### U-1: tree-sitter version — bump to 0.25 required and validated

**Answer**: `tree-sitter-haskell` 0.23.1 (ABI 14) works with both tree-sitter 0.24 and 0.25. `tree-sitter-rescript` at HEAD (ABI 15) **fails at runtime** with tree-sitter 0.24 (`LanguageError { version: 15 }`) but **works with tree-sitter 0.25**.

**Evidence**:
- tree-sitter 0.24 defines `TREE_SITTER_LANGUAGE_VERSION=14`, `MIN_COMPATIBLE=13` → rejects ABI 15
- tree-sitter-rescript's `src/parser.c` at HEAD: `LANGUAGE_VERSION 15` (introduced in commit `cc6950a`, their tree-sitter 0.25.2 update)
- tree-sitter-rescript's `src/parser.c` before `cc6950a`: `LANGUAGE_VERSION 14`
- Bumped project to tree-sitter 0.25: **clean build, all 9 existing grammars pass** (tested in nix develop shell)
- Also bumped `tree-sitter-javascript` 0.23→0.25, `tree-sitter-python` 0.23→0.25, `tree-sitter-go` 0.23→0.25 (newer versions available)

**Implication**: The tree-sitter 0.25 bump is safe and solves the ReScript compatibility issue. No regressions in existing language support.

### U-2: Haskell node kinds — good coverage with existing patterns

**Answer**: Haskell's grammar wraps all declarations in a `declarations` node. Top-level walk must descend into it. Key node kinds:

| Node kind | OutlineKind mapping | Name extraction |
|-----------|-------------------|-----------------|
| `function` | Function | `name` field |
| `bind` | Function (value binding) | `name` field |
| `signature` | Function (type sig) | `name` field |
| `data_type` | Enum (sum type) | `name` field |
| `newtype` | Struct (wrapper) | `name` field |
| `type_synomym` | TypeAlias | `name` field |
| `class` | Interface (typeclass) | `name` field |
| `instance` | Class (typeclass instance) | `name` field |
| `foreign_import` | Import (FFI) | child text |
| `haddock` | (doc comment, attach to next decl) | — |

**Evidence**: Parsed sample code containing all Haskell declaration forms. The `declarations` wrapper is the only structural surprise — all other node kinds have clean `name` fields.

**Implication**: Most Haskell kinds map naturally to existing `OutlineKind` variants. The `declarations` wrapper node needs handling in the tree walk (descend into it rather than treating it as a declaration).

### U-3: ReScript node kinds — ML-family patterns need new match arms

**Answer**: ReScript uses `_declaration` → `_binding` nesting. Top-level declarations are direct children of the root `source_file` node. Key node kinds:

| Node kind | OutlineKind mapping | Name extraction |
|-----------|-------------------|-----------------|
| `let_declaration` | Variable/Function | child `let_binding` → `name` field |
| `type_declaration` | TypeAlias/Enum | child `type_binding` → `name` field |
| `module_declaration` | Module | child `module_binding` → `name` field |
| `external_declaration` | Function (FFI) | `value_identifier` child |
| `open_statement` | Import | `module_identifier` child |
| `exception_declaration` | Enum (variant) | `variant_identifier` child |
| `decorator` | (annotation, attach to next decl) | `decorator_identifier` child |

**Evidence**: Parsed sample code with let bindings, type declarations, modules, externals, open statements, and exception declarations. Name extraction follows a consistent pattern: declaration → inner binding → name field.

**Implication**: ReScript's declaration→binding nesting is different from C-family `function_declaration` patterns. The `node_to_entry` function needs new arms for these kinds. Name extraction requires descending one level into the binding child node rather than reading a direct `name` field.

### U-4: Dependency strategy confirmed

**Answer**:
- `tree-sitter-haskell` 0.23.1: **crates.io** — official tree-sitter org crate, MIT licensed, straightforward `Cargo.toml` dependency
- `tree-sitter-rescript`: **git dependency** on `https://github.com/rescript-lang/tree-sitter-rescript` — not published to crates.io but has full Rust bindings (`bindings/rust/lib.rs` with `LANGUAGE` export, `Cargo.toml`, `build.rs`)

**Evidence**:
- `cargo search tree-sitter-haskell` → v0.23.1 found
- `cargo search tree-sitter-rescript` → no results
- GitHub repo inspection: `Cargo.toml` exists at root, `bindings/rust/lib.rs` exports `LANGUAGE: LanguageFn`

**Implication**: Haskell is a standard crates.io dep. ReScript requires a git dep which is slightly less stable but the official repo is actively maintained by the ReScript lang org.


### U-5: ReScript 12 + JSX semantic coverage — grammar supports structural extraction

**Answer**: The official grammar provides structural JSX nodes required for semantic indexing (`jsx_element`, `jsx_self_closing_element`, `jsx_fragment`, `jsx_expression`), plus decorator parsing (`@react.component`). This is sufficient to build best-effort JSX semantic indexing in tilth.

**Evidence**:
- ReScript docs URLs are reachable in this environment (HTTP 200):
  - `https://rescript-lang.org/blog/rescript-12-release`
  - `https://rescript-lang.org/blog/rescript-12-is-out`
  - `https://rescript-lang.org/docs/manual/latest/jsx/jsx4`
- `tree-sitter-rescript` corpus includes dedicated JSX coverage (`test/corpus/jsx.txt`) with nested/dotted component tags, fragments, spread expressions, and nested JSX children.
- `tree-sitter-rescript` decorator corpus includes `@react.component` forms (`test/corpus/decorators.txt`).
- No explicit `ReScript 12` marker string exists in the grammar repo; support confidence must be asserted by fixture-backed acceptance tests.

**Implication**: We should define ReScript 12 support operationally as passing a comprehensive JSX/decorator fixture matrix through tilth's real indexing pipeline.

---

## Summary

| Unknown | Resolved? | Key decision |
|---------|-----------|-------------|
| U-1: tree-sitter version | ✅ Yes | Bump tree-sitter to 0.25 globally — validated, no regressions |
| U-2: Haskell node kinds | ✅ Yes | Good mapping to existing OutlineKind; handle `declarations` wrapper |
| U-3: ReScript node kinds | ✅ Yes | New match arms needed for `_declaration`→`_binding` pattern |
| U-4: Dependency strategy | ✅ Yes | Haskell from crates.io, ReScript as git dep |
| U-5: ReScript 12 + JSX coverage | ✅ Yes | Use fixture-backed acceptance to claim support; grammar nodes are sufficient |

All unknowns resolved. No blockers for design.

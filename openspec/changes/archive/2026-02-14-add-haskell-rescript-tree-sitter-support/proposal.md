## Why

Tilth's tree-sitter integration currently supports 9 languages (Rust, TypeScript/TSX, JavaScript, Python, Go, Java, C, C++, Ruby). Haskell and ReScript are unrepresented — users working in these languages get no structural outlines and no AST-based symbol search, falling back to heuristic head/tail summaries. Both languages have mature tree-sitter grammars available: `tree-sitter-haskell` (v0.23.1 on crates.io, maintained by the tree-sitter org) and `tree-sitter-rescript` (official grammar from `rescript-lang`, with Rust bindings, not yet on crates.io).

## What Changes

- Add `Haskell` and `ReScript` variants to the `Lang` enum
- Add file extension detection for `.hs`, `.res`, `.resi`
- Integrate `tree-sitter-haskell` (crates.io v0.23.1) for Haskell parsing
- Integrate `tree-sitter-rescript` (git dependency from `rescript-lang/tree-sitter-rescript`) for ReScript parsing
- Bump core `tree-sitter` runtime from 0.24 to 0.25 globally so ReScript grammar ABI 15 is supported
- Map Haskell-specific tree-sitter node kinds (e.g., `function`, `data_type`, `type_alias`, `class`, `instance`, `signature`, `module`, `import`) to `OutlineKind`
- Map ReScript-specific tree-sitter node kinds (e.g., `let_declaration`, `type_declaration`, `module_declaration`, `external_declaration`, `open_statement`) to `OutlineKind`
- Extend `DEFINITION_KINDS` in symbol search to include language-specific node kinds for both languages
- Require ReScript 12 syntax compatibility (including JSX-heavy component files)
- Add JSX-semantic indexing for ReScript (`@react.component`, JSX elements/fragments, spread props)
- Add a comprehensive ReScript 12 fixture corpus plus acceptance/integration tests

## Capabilities

### New Capabilities
| Capability | CAP Code | Description |
|------------|----------|-------------|
| `haskell-tree-sitter` | `HASKELL_TREE_SITTER` | Tree-sitter integration for Haskell: structural outlines showing top-level declarations (functions, data types, type aliases, classes, instances, imports) and AST-based symbol search |
| `rescript-tree-sitter` | `RESCRIPT_TREE_SITTER` | Tree-sitter integration for ReScript 12: declaration outlines, JSX-semantic indexing, and AST-based symbol search validated by acceptance fixtures |

### Modified Capabilities
| Capability | CAP Code | What's Changing |
|------------|----------|-----------------|
| | | |

### Removed Capabilities
None.

## Impact

- **Code**: `src/types.rs` (Lang enum), `src/read/mod.rs` (extension detection), `src/read/outline/code.rs` (outline_language + node_to_entry + JSX semantic extraction), `src/search/symbol.rs` (DEFINITION_KINDS), plus new ReScript fixture/integration tests
- **Dependencies**: `tree-sitter-haskell = "0.23"` added to Cargo.toml from crates.io; `tree-sitter-rescript` added as git dependency from `https://github.com/rescript-lang/tree-sitter-rescript`; project-wide `tree-sitter` bumped to `0.25`
- **Risk**: ReScript is a git dependency (not crates.io), so upstream repo churn is a supply risk; mitigated by lockfile commit pinning
- **No breaking changes**: Existing language support is unchanged; new variants are additive

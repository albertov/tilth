# Capability: haskell-tree-sitter

Tree-sitter integration for Haskell: structural outlines, AST-based symbol search, and definition detection.

## Requirements

### FR-1: Haskell language variant

Tilth recognises Haskell source files and routes them through the tree-sitter pipeline.

- `Lang::Haskell` variant exists in the `Lang` enum.
- `.hs` extension maps to `FileType::Code(Lang::Haskell)` in `detect_file_type`.

#### SC-1.1: Haskell file detected by extension
WHEN a file has extension `.hs`
THEN `detect_file_type` returns `FileType::Code(Lang::Haskell)`

#### SC-1.2: Non-Haskell extensions unchanged
WHEN a file has extension `.rs`, `.ts`, `.py`, or any other already-mapped extension
THEN `detect_file_type` returns the same result as before this change

### FR-2: Tree-sitter grammar binding

`outline_language(Lang::Haskell)` returns the tree-sitter-haskell grammar.

- Dependency: `tree-sitter-haskell` from crates.io (0.23.x series).
- Returns `Some(tree_sitter_haskell::LANGUAGE.into())`.

#### SC-2.1: Grammar loads successfully
WHEN `outline_language(Lang::Haskell)` is called
THEN it returns `Some(language)` where `language` can initialise a tree-sitter `Parser`

### FR-3: Outline generation — top-level declarations

Haskell outlines show the structural declarations in a file. The tree-sitter-haskell grammar wraps all declarations in a `declarations` node — the outline walker must descend into it.

| Node kind | OutlineKind | Notes |
|-----------|-------------|-------|
| `function` | Function | Named function equation |
| `bind` | Function | Value binding (e.g. `x = 42`) |
| `signature` | Function | Type signature (e.g. `foo :: Int -> Int`) |
| `data_type` | Enum | Algebraic data type |
| `newtype` | Struct | Newtype wrapper |
| `type_synomym` | TypeAlias | Type synonym (note: grammar spells it `synomym`) |
| `class` | Interface | Typeclass definition |
| `instance` | Class | Typeclass instance |
| `foreign_import` | Import | FFI import |

Name extraction: declaration kinds use the `name` field where available; `foreign_import` extracts module text from a child node.

#### SC-3.1: Function declarations appear in outline
WHEN a Haskell file contains `area :: Shape -> Double` and `area (Circle r) = pi * r * r`
THEN the outline includes a Function entry named `area` (signature and equation)

#### SC-3.2: Data types appear in outline
WHEN a Haskell file contains `data Shape = Circle Double | Rectangle Double Double`
THEN the outline includes an Enum entry named `Shape`

#### SC-3.3: Typeclasses appear in outline
WHEN a Haskell file contains `class Printable a where`
THEN the outline includes an Interface entry named `Printable`

#### SC-3.4: Newtype and type synonym appear in outline
WHEN a Haskell file contains `newtype Wrapper = Wrapper Int` and `type Name = String`
THEN the outline includes a Struct entry named `Wrapper` and a TypeAlias entry named `Name`

#### SC-3.5: Declarations wrapper is transparent
WHEN a Haskell file has declarations wrapped in a `declarations` node (standard grammar structure)
THEN the outline walker descends into `declarations` and surfaces its children as top-level entries

### FR-4: Symbol search — definition detection

Haskell-specific node kinds are included in `DEFINITION_KINDS` for structural symbol search.

New definition kinds: `function`, `bind`, `data_type`, `newtype`, `type_synomym`, `class`, `instance`, `signature`.

#### SC-4.1: Symbol search finds Haskell definitions
WHEN searching for symbol `Shape` in a project containing `data Shape = Circle Double | Rectangle Double Double`
THEN the definition result includes the `data_type` node for `Shape`

#### SC-4.2: Symbol search finds Haskell functions
WHEN searching for symbol `area` in a project containing `area :: Shape -> Double`
THEN the definition result includes the `signature` and/or `function` node for `area`

### FR-5: Best-effort parity — import source extraction

Haskell `import` statements should have their source module extracted where feasible.

#### SC-5.1: Import source extracted
WHEN a Haskell file contains `import Data.Map.Strict`
THEN the import entry's source is `Data.Map.Strict`

### NFR-1: No regression in existing languages

Adding Haskell support must not change the behaviour of outline generation, symbol search, or file detection for any existing supported language.

### EDGE-1: Haskell files without top-level declarations

WHEN a Haskell file contains only comments or pragmas and no declarations
THEN the outline is empty (no entries) rather than erroring

### EDGE-2: Literate Haskell (.lhs) not in scope

`.lhs` (literate Haskell) files are out of scope for this change. They are not detected as Haskell.

## Code Index

| Code | Type | Summary |
|------|------|---------|
| FR-1 | Functional | Lang variant and extension detection |
| SC-1.1 | Scenario | .hs extension → Haskell |
| SC-1.2 | Scenario | Existing extensions unchanged |
| FR-2 | Functional | Tree-sitter grammar binding |
| SC-2.1 | Scenario | Grammar loads successfully |
| FR-3 | Functional | Outline generation for Haskell declarations |
| SC-3.1 | Scenario | Functions in outline |
| SC-3.2 | Scenario | Data types in outline |
| SC-3.3 | Scenario | Typeclasses in outline |
| SC-3.4 | Scenario | Newtype and type synonym in outline |
| SC-3.5 | Scenario | Declarations wrapper transparent |
| FR-4 | Functional | Symbol search definition detection |
| SC-4.1 | Scenario | Symbol search finds data type |
| SC-4.2 | Scenario | Symbol search finds function |
| FR-5 | Functional | Import source extraction (best-effort) |
| SC-5.1 | Scenario | Import source extracted |
| NFR-1 | Non-functional | No regression in existing languages |
| EDGE-1 | Edge case | Empty Haskell file |
| EDGE-2 | Edge case | .lhs out of scope |

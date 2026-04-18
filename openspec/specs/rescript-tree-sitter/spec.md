# Capability: rescript-tree-sitter

Tree-sitter integration for ReScript 12: declaration indexing, JSX-semantic indexing, and AST-based symbol search.

## Requirements

### FR-1: ReScript language variant

Tilth recognises ReScript source files and routes them through the tree-sitter pipeline.

- `Lang::ReScript` variant exists in the `Lang` enum.
- `.res` extension maps to `FileType::Code(Lang::ReScript)`.
- `.resi` (ReScript interface files) maps to `FileType::Code(Lang::ReScript)`.

#### SC-1.1: ReScript file detected by .res extension
WHEN a file has extension `.res`
THEN `detect_file_type` returns `FileType::Code(Lang::ReScript)`

#### SC-1.2: ReScript interface file detected by .resi extension
WHEN a file has extension `.resi`
THEN `detect_file_type` returns `FileType::Code(Lang::ReScript)`

#### SC-1.3: Non-ReScript extensions unchanged
WHEN a file has extension `.rs`, `.ts`, `.py`, or any other already-mapped extension
THEN `detect_file_type` returns the same result as before this change

### FR-2: Tree-sitter grammar binding

`outline_language(Lang::ReScript)` returns the tree-sitter-rescript grammar.

- Dependency: `tree-sitter-rescript` as git dependency from `https://github.com/rescript-lang/tree-sitter-rescript`.
- Returns `Some(tree_sitter_rescript::LANGUAGE.into())`.
- The crate is not on crates.io; the git dep must be pinned to a commit or branch.

#### SC-2.1: Grammar loads successfully
WHEN `outline_language(Lang::ReScript)` is called
THEN it returns `Some(language)` where `language` can initialise a tree-sitter `Parser`

### FR-3: Outline generation — top-level declarations

ReScript outlines show the structural declarations in a file. ReScript uses a `_declaration` -> `_binding` nesting pattern — name extraction requires descending into the inner binding node.

| Node kind | OutlineKind | Name extraction |
|-----------|-------------|-----------------|
| `let_declaration` | Variable (or Function) | child `let_binding` -> `name` field |
| `type_declaration` | TypeAlias | child `type_binding` -> `name` field |
| `module_declaration` | Module | child `module_binding` -> `name` field |
| `external_declaration` | Function | `value_identifier` child |
| `open_statement` | Import | `module_identifier` child |
| `exception_declaration` | Enum | `variant_identifier` child |

Decorators (`@module`, `@genType`, etc.) are not separate top-level declaration entries — they annotate the following declaration.

#### SC-3.1: Let bindings appear in outline
WHEN a ReScript file contains `let greeting = "hello"` and `let add = (a, b) => a + b`
THEN the outline includes entries named `greeting` and `add`

#### SC-3.2: Type declarations appear in outline
WHEN a ReScript file contains `type color = Red | Green | Blue`
THEN the outline includes a TypeAlias entry named `color`

#### SC-3.3: Module declarations appear in outline
WHEN a ReScript file contains `module StringMap = Belt.Map.String`
THEN the outline includes a Module entry named `StringMap`

#### SC-3.4: External declarations appear in outline
WHEN a ReScript file contains `external log: string => unit = "console.log"`
THEN the outline includes a Function entry named `log`

#### SC-3.5: Open statements appear in outline
WHEN a ReScript file contains `open Belt`
THEN the outline includes an Import entry with source `Belt`

#### SC-3.6: Exception declarations appear in outline
WHEN a ReScript file contains `exception Invalid(string)`
THEN the outline includes an Enum entry named `Invalid`

### FR-4: Symbol search — definition detection

ReScript-specific node kinds are included in `DEFINITION_KINDS` for structural symbol search.

New definition kinds: `let_declaration`, `type_declaration`, `module_declaration`, `external_declaration`, `exception_declaration`.

#### SC-4.1: Symbol search finds ReScript type definitions
WHEN searching for symbol `color` in a project containing `type color = Red | Green | Blue`
THEN the definition result includes the `type_declaration` node for `color`

#### SC-4.2: Symbol search finds ReScript let bindings
WHEN searching for symbol `add` in a project containing `let add = (a, b) => a + b`
THEN the definition result includes the `let_declaration` node for `add`

#### SC-4.3: Symbol search finds ReScript modules
WHEN searching for symbol `StringMap` in a project containing `module StringMap = Belt.Map.String`
THEN the definition result includes the `module_declaration` node for `StringMap`

### FR-5: Best-effort parity — import source extraction

ReScript `open` statements should have their source module extracted where feasible.

#### SC-5.1: Open statement source extracted
WHEN a ReScript file contains `open Belt`
THEN the import entry's source is `Belt`

### FR-6: MUST support ReScript 12 syntax in indexing pipeline

ReScript 12 source files, including JSX-heavy component modules, must parse and produce index output without falling back to generic summaries.

- Parser compatibility is validated through fixture-backed acceptance tests.
- Support claims are based on executed fixtures, not version labels in upstream grammar metadata.

#### SC-6.1: Decorated component module parses and indexes
WHEN a file contains `@react.component let make = (...) => <div />`
THEN outline and symbol indexing include `make` and no parser-compatibility error is produced

#### SC-6.2: JSX fragment + spread parse and index
WHEN a file contains fragments (`<>...</>`) and spread props (`<Comp {...props} />`)
THEN the indexing pipeline completes successfully and returns semantic output for the component module

#### SC-6.3: Dotted component tags parse and index
WHEN a file contains JSX tags like `<Foo.Bar />`
THEN indexing preserves the dotted tag path in semantic output

### FR-7: JSX-semantic indexing for ReScript components

Tilth extracts JSX semantics from ReScript AST nodes and exposes them as structured outline semantics under component declarations.

- Target nodes: `jsx_element`, `jsx_self_closing_element`, `jsx_fragment`, `jsx_expression` with spread.
- Semantic extraction includes tag names, fragment markers, and spread-props markers.
- JSX semantic entries are associated with the owning component declaration (`@react.component let make`).

#### SC-7.1: JSX element tags indexed under component
WHEN `make` renders `<Button />` and `<Layout.Header />`
THEN `make` includes semantic child entries for `Button` and `Layout.Header`

#### SC-7.2: JSX fragment indexed
WHEN `make` renders `<> <A /> <B /> </>`
THEN semantic output includes a fragment marker with child tag semantics

#### SC-7.3: Spread props indexed
WHEN JSX includes `<Comp {...props} />`
THEN semantic output includes a spread-props marker linked to `Comp`

#### SC-7.4: Nested JSX child expressions indexed best-effort
WHEN JSX includes nested expression children
THEN semantic output records nested JSX tags best-effort without crashing or dropping the component entry

#### SC-7.5: Non-JSX ReScript files remain declaration-only
WHEN a ReScript file has no JSX nodes
THEN output includes declaration indexing only, unchanged from FR-3 behavior

### NFR-1: No regression in existing languages

Adding ReScript support must not change the behaviour of outline generation, symbol search, or file detection for any existing supported language.

### EDGE-1: ReScript interface files (.resi)

`.resi` files contain only type signatures and module signatures. They are parsed with the same grammar and produce the same outline kinds.

#### SC-E1.1: Interface file produces outline
WHEN a `.resi` file contains `let add: (int, int) => int` and `type color`
THEN the outline includes entries for `add` and `color`

### EDGE-2: Decorated declarations

WHEN a ReScript file contains `@module("fs") external readFile: string => string = "readFileSync"`
THEN the outline includes a Function entry named `readFile` (the decorator is not a separate declaration entry)

### EDGE-3: Empty ReScript files

WHEN a ReScript file contains no declarations
THEN the outline is empty (no entries) rather than erroring

### EDGE-4: Malformed JSX in otherwise valid file

WHEN a ReScript file contains a malformed JSX subtree
THEN indexing degrades gracefully (best-effort output) and does not crash the process

### QA-1: ReScript 12 JSX fixture matrix and acceptance tests

The change includes a comprehensive fixture matrix (minimum 12 fixtures) plus integration tests that execute tilth's real read/search pipeline.

- Fixture classes include: decorated components, fragments, spread props, dotted tags, nested children expressions, mixed declaration + JSX files, and `.resi` files.
- Acceptance assertions validate both declaration indexing and JSX semantic output.

## Code Index

| Code | Type | Summary |
|------|------|---------|
| FR-1 | Functional | Lang variant and extension detection |
| SC-1.1 | Scenario | .res extension -> ReScript |
| SC-1.2 | Scenario | .resi extension -> ReScript |
| SC-1.3 | Scenario | Existing extensions unchanged |
| FR-2 | Functional | Tree-sitter grammar binding |
| SC-2.1 | Scenario | Grammar loads successfully |
| FR-3 | Functional | Outline generation for ReScript declarations |
| SC-3.1 | Scenario | Let bindings in outline |
| SC-3.2 | Scenario | Type declarations in outline |
| SC-3.3 | Scenario | Module declarations in outline |
| SC-3.4 | Scenario | External declarations in outline |
| SC-3.5 | Scenario | Open statements in outline |
| SC-3.6 | Scenario | Exception declarations in outline |
| FR-4 | Functional | Symbol search definition detection |
| SC-4.1 | Scenario | Symbol search finds type definition |
| SC-4.2 | Scenario | Symbol search finds let binding |
| SC-4.3 | Scenario | Symbol search finds module |
| FR-5 | Functional | Import source extraction (best-effort) |
| SC-5.1 | Scenario | Open statement source extracted |
| FR-6 | Functional | ReScript 12 syntax compatibility in indexing pipeline |
| SC-6.1 | Scenario | Decorated component parses/indexes |
| SC-6.2 | Scenario | Fragment + spread parse/index |
| SC-6.3 | Scenario | Dotted JSX tags preserved |
| FR-7 | Functional | JSX-semantic indexing for ReScript components |
| SC-7.1 | Scenario | JSX tags indexed under component |
| SC-7.2 | Scenario | Fragment semantic indexed |
| SC-7.3 | Scenario | Spread-props semantic indexed |
| SC-7.4 | Scenario | Nested JSX expressions indexed best-effort |
| SC-7.5 | Scenario | Non-JSX files remain declaration-only |
| NFR-1 | Non-functional | No regression in existing languages |
| EDGE-1 | Edge case | .resi interface files |
| SC-E1.1 | Scenario | Interface file produces outline |
| EDGE-2 | Edge case | Decorated declarations |
| EDGE-3 | Edge case | Empty ReScript files |
| EDGE-4 | Edge case | Malformed JSX subtree |
| QA-1 | Quality gate | Fixture matrix + acceptance/integration tests |

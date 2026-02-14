## Overview

tilth now provides structural code intelligence for Haskell (`.hs`) and ReScript (`.res`, `.resi`) files, including ReScript 12 component modules with JSX semantics. This includes AST-based outlines, symbol definitions, and best-effort JSX semantic indexing.

No configuration needed. If you point tilth at a codebase containing Haskell or ReScript files, it just works.

## Prerequisites

- tilth built from the branch containing this change (tree-sitter bumped to 0.25)
- No additional tooling or grammars to install — everything is compiled in

## Setup

1. Build tilth as usual:
   ```
   $ cargo build --release
   ```
2. No configuration changes needed. Haskell and ReScript support is automatic.

## Usage Examples

### Reading a Haskell file (structural outline)

When tilth reads a large Haskell file, it produces an AST outline instead of dumping raw content:

```
$ # Via MCP tool: tilth_read with path "src/Types.hs"
-- module Types
data Shape = Circle Double | Rectangle Double Double
type Name = String
newtype Wrapper a = Wrapper a
class Printable a where
  prettyPrint :: a -> String
area :: Shape -> Double
area (Circle r) = ...
area (Rectangle w h) = ...
```

### Reading a ReScript 12 file (declarations + JSX semantics)

```
$ # Via MCP tool: tilth_read with path "src/App.res"
@react.component
let make = (~title, ~props) =>
  <>
    <Layout.Header title />
    <Button {...props} />
  </>

# Outline includes declaration-level entries:
# - make
# - helper declarations in file
# and JSX semantic children under make:
# - Layout.Header
# - Button
# - <Fragment>
# - {...props}
```

### Symbol search in Haskell

```
$ # Via MCP tool: tilth_search with query "Printable"
# Definitions:
  src/Types.hs:10  class Printable a where
  src/Types.hs:15  instance Printable Shape where

# Usages:
  src/Main.hs:5    import Types (Printable(..))
  src/Render.hs:8  prettyPrint shape
```

### Symbol search in ReScript

```
$ # Via MCP tool: tilth_search with query "make"
# Definitions:
  src/App.res:2  let make = (~title, ~props) => ...

# Usages:
  src/Router.res:8  <App makeTitle="Home" />
```

### Codebase map includes Haskell and ReScript

```
$ # Via MCP tool: tilth_map
src/
  Types.hs          -- Shape, Name, Wrapper, Printable, area
  Main.hs           -- main
  App.res           -- make, helperTypes, helperModules
  Components.resi   -- make (interface)
```

## Verification

- [ ] `cargo build` succeeds with no tree-sitter-related errors
- [ ] `tilth_read` on a `.hs` file shows structural outline with functions, types, classes
- [ ] `tilth_read` on a ReScript 12 JSX file (`@react.component`, fragment, spread props, dotted tags) shows declaration + JSX semantics
- [ ] `tilth_search` finds definitions in Haskell files by symbol name
- [ ] `tilth_search` finds definitions in ReScript files by symbol name
- [ ] ReScript fixture matrix (12+ fixtures) passes acceptance/integration tests
- [ ] Existing language support (Rust, TypeScript, etc.) is unaffected

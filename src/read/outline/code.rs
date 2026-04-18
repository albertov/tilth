use crate::lang::outline::{extract_import_source, outline_language, walk_top_level};
use crate::types::{Lang, OutlineEntry, OutlineKind};
use std::path::Path;

/// Generate a code outline using tree-sitter. Walks top-level AST nodes,
/// emitting signatures without bodies.
pub fn outline(content: &str, lang: Lang, max_lines: usize, path: Option<&Path>) -> String {
    let Some(language) = outline_language(lang) else {
        return fallback_outline(content, max_lines);
    };

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return fallback_outline(content, max_lines);
    }

    let Some(tree) = parser.parse(content, None) else {
        return fallback_outline(content, max_lines);
    };

    let root = tree.root_node();
    let lines: Vec<&str> = content.lines().collect();
    let mut entries = walk_top_level(root, &lines, lang);

    // ReScript: every .res file is implicitly a module named after the file.
    // Wrap top-level entries in a synthetic module so symbol search finds
    // the component by its module name (e.g., searching "Button" finds Button.res).
    if lang == Lang::ReScript {
        if let Some(stem) = path.and_then(|p| p.file_stem()).and_then(|s| s.to_str()) {
            let end_line = entries.last().map_or(0, |e| e.end_line);
            entries = vec![OutlineEntry {
                kind: OutlineKind::Module,
                name: stem.to_string(),
                start_line: 1,
                end_line,
                signature: None,
                children: entries,
                doc: None,
            }];
        }
    }

    format_entries(&entries, &lines, max_lines, lang)
}

/// Format outline entries into the spec'd output format.
fn format_entries(
    entries: &[OutlineEntry],
    _lines: &[&str],
    max_lines: usize,
    lang: Lang,
) -> String {
    let mut out = Vec::new();
    let mut import_groups: Vec<&str> = Vec::new();
    // Track the start line of the first import in the current group.
    let mut import_group_start: u32 = 1;

    for entry in entries {
        if out.len() >= max_lines {
            break;
        }

        match entry.kind {
            OutlineKind::Import => {
                if import_groups.is_empty() {
                    import_group_start = entry.start_line;
                }
                import_groups.push(&entry.name);
                continue;
            }
            _ => {
                // Flush any accumulated imports
                if !import_groups.is_empty() {
                    out.push(format_imports(&import_groups, import_group_start));
                    import_groups.clear();
                }
            }
        }

        // Flatten namespace modules — hoist their children to top level
        // so classes inside namespaces show their methods at indent 1.
        if entry.kind == OutlineKind::Module && !entry.children.is_empty() {
            out.push(format_entry(entry, 0, lang));
            for child in &entry.children {
                if out.len() >= max_lines {
                    break;
                }
                out.push(format_entry(child, 1, lang));
                for grandchild in &child.children {
                    if out.len() >= max_lines {
                        break;
                    }
                    out.push(format_entry(grandchild, 2, lang));
                }
            }
        } else {
            out.push(format_entry(entry, 0, lang));
            for child in &entry.children {
                if out.len() >= max_lines {
                    break;
                }
                out.push(format_entry(child, 1, lang));
            }
        }
    }

    // Flush trailing imports
    if !import_groups.is_empty() {
        out.push(format_imports(&import_groups, import_group_start));
    }

    out.join("\n")
}

/// Format a collapsed import summary grouped by source with counts.
/// Spec format: `imports: react(4), express(2), @/lib(3)`
fn format_imports(imports: &[&str], start: u32) -> String {
    let count = imports.len();

    // Extract source modules and count occurrences
    let mut sources: Vec<String> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for imp in imports {
        let source = extract_import_source(imp);
        *seen.entry(source.clone()).or_insert(0) += 1;
        if !sources.contains(&source) {
            sources.push(source);
        }
    }

    // Format as "source(count)" or just "source" if count is 1
    let mut parts: Vec<String> = Vec::new();
    for src in sources.iter().take(5) {
        let c = seen[src];
        if c > 1 {
            parts.push(format!("{src}({c})"));
        } else {
            parts.push(src.clone());
        }
    }

    let suffix = if count > 5 {
        format!(", ... ({count} total)")
    } else {
        String::new()
    };
    let condensed = parts.join(", ");
    format!("[{start}-]   imports: {condensed}{suffix}")
}

/// Format a single outline entry with optional indentation.
fn format_entry(entry: &OutlineEntry, indent: usize, lang: Lang) -> String {
    let prefix = "  ".repeat(indent);
    let range = if entry.start_line == entry.end_line {
        format!("[{}]", entry.start_line)
    } else {
        format!("[{}-{}]", entry.start_line, entry.end_line)
    };

    let kind_label = match entry.kind {
        OutlineKind::Function => {
            if lang == Lang::Scala {
                "def"
            } else if lang == Lang::Kotlin {
                "fun"
            } else {
                "fn"
            }
        }
        OutlineKind::Class => "class",
        OutlineKind::Struct => "struct",
        OutlineKind::Interface => {
            if lang == Lang::Scala {
                "trait"
            } else {
                "interface"
            }
        }
        OutlineKind::TypeAlias => "type",
        OutlineKind::Enum => "enum",
        OutlineKind::Constant => "const",
        OutlineKind::ImmutableVariable => "val",
        OutlineKind::Variable => {
            if lang == Lang::Scala {
                "var"
            } else {
                "let"
            }
        }
        OutlineKind::Export => "export",
        OutlineKind::Property => "prop",
        OutlineKind::Module => {
            if lang == Lang::Scala || lang == Lang::Kotlin {
                "object"
            } else {
                "mod"
            }
        }
        OutlineKind::Import => "import",
        OutlineKind::TestSuite => "suite",
        OutlineKind::TestCase => "test",
    };

    let sig = match &entry.signature {
        Some(s) => format!("\n{prefix}           {s}"),
        None => String::new(),
    };

    let doc = match &entry.doc {
        Some(d) => {
            let truncated = if d.len() > 60 {
                format!("{}...", crate::types::truncate_str(d, 57))
            } else {
                d.clone()
            };
            format!("  // {truncated}")
        }
        None => String::new(),
    };

    format!("{prefix}{range:<12} {kind_label} {}{sig}{doc}", entry.name)
}

/// Fallback when tree-sitter grammar isn't available.
fn fallback_outline(content: &str, _max_lines: usize) -> String {
    super::fallback::head_tail(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scala_outline_constructs() {
        let scala_code = r#"
package example

import scala.util.Try

trait DataSource {
  def load(): String
}

class Database {
  val connectionString = "jdbc:..."
  var connected = false
  
  def connect(): Unit = {}
}

object Database {
  def create(): Database = new Database()
}

enum Color {
  case Red, Green, Blue
}

type UserId = String
"#;

        let outline = outline(scala_code, Lang::Scala, 1000, None);

        assert!(outline.contains("trait DataSource"));
        assert!(outline.contains("class Database"));
        assert!(outline.contains("object Database"));
        assert!(outline.contains("enum Color"));
        assert!(outline.contains("type UserId"));
        assert!(outline.contains("val connectionString"));
        assert!(outline.contains("var connected"));
        assert!(outline.contains("def load"));
        assert!(outline.contains("def connect"));
        assert!(outline.contains("def create"));
    }

    #[test]
    fn php_outline_constructs() {
        let php_code = r#"<?php
namespace App\Services;

use App\Support\Client;

trait LogsQueries {
    public function log(string $query): void {}
}

class UserService {
    use LogsQueries;

    public function __construct(private Client $client) {}

    public function findUser(int $id): array {
        return $this->client->loadUser($id);
    }
}
"#;

        let outline = outline(php_code, Lang::Php, 1000, None);

        assert!(outline.contains("mod App\\Services"));
        assert!(outline.contains("imports: App\\Support\\Client"));
        assert!(outline.contains("interface LogsQueries"));
        assert!(outline.contains("class UserService"));
        assert!(outline.contains("fn findUser"));
    }

    #[test]
    fn kotlin_outline_constructs() {
        let kotlin_code = r#"
package com.example

import kotlin.collections.List
import kotlin.io.println

interface Drawable {
    fun draw()
}

data class Point(val x: Int, val y: Int)

class Canvas : Drawable {
    val width = 800
    var height = 600

    override fun draw() {
        println("Drawing")
    }

    fun resize(w: Int, h: Int) {}

    companion object {
        fun create(): Canvas = Canvas()
    }
}

object Registry {
    fun register(item: Drawable) {}
}

enum class Color {
    RED, GREEN, BLUE
}

fun String.isPalindrome(): Boolean = this == this.reversed()

fun main() {
    val canvas = Canvas()
    canvas.draw()
}
"#;

        let outline = outline(kotlin_code, Lang::Kotlin, 1000, None);

        // Imports
        assert!(
            outline.contains("imports:"),
            "should have collapsed imports"
        );
        // Interface (shown as class since Kotlin grammar uses class_declaration)
        assert!(outline.contains("class Drawable"), "should have Drawable");
        // Data class
        assert!(outline.contains("class Point"), "should have Point");
        // Regular class with methods
        assert!(outline.contains("class Canvas"), "should have Canvas");
        assert!(outline.contains("fun draw"), "should have draw method");
        assert!(outline.contains("fun resize"), "should have resize method");
        // Properties inside classes
        assert!(outline.contains("prop width"), "should have width property");
        assert!(
            outline.contains("prop height"),
            "should have height property"
        );
        // Object declaration
        assert!(
            outline.contains("object Registry"),
            "should have Registry object"
        );
        assert!(
            outline.contains("fun register"),
            "should have register method"
        );
        // Enum class
        assert!(outline.contains("class Color"), "should have Color enum");
        // Top-level functions
        assert!(
            outline.contains("fun isPalindrome"),
            "should have extension fun"
        );
        assert!(outline.contains("fun main"), "should have main");
        // Kotlin-specific labels
        assert!(outline.contains("fun "), "should use 'fun' not 'fn'");
        assert!(!outline.contains("fn "), "should not use 'fn' for Kotlin");
    }

    #[test]
    fn test_haskell_grammar_loads() {
        let lang = outline_language(Lang::Haskell);
        assert!(lang.is_some(), "Haskell grammar should be available");
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang.unwrap())
            .expect("Haskell grammar should initialize parser");
    }

    #[test]
    fn test_rescript_grammar_loads() {
        let lang = outline_language(Lang::ReScript);
        assert!(lang.is_some(), "ReScript grammar should be available");
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang.unwrap())
            .expect("ReScript grammar should initialize parser");
    }

    #[test]
    fn test_haskell_outline_declarations() {
        let source = r#"module Main where

import Data.Map.Strict
import qualified Data.Text as T

data Color = Red | Green | Blue

newtype Name = Name String

type Alias = String

class Printable a where
  display :: a -> String

instance Printable Color where
  display Red = "red"

add :: Int -> Int -> Int
add x y = x + y
"#;

        let out = outline(source, Lang::Haskell, 200, None);

        assert!(out.contains("imports:"), "should collapse imports");
        assert!(out.contains("Data.Map.Strict"), "should include import source");
        assert!(out.contains("Data.Text"), "should handle qualified import source");
        assert!(out.contains("enum Color"), "data type should map to enum");
        assert!(out.contains("struct Name"), "newtype should map to struct");
        assert!(out.contains("type Alias"), "type synonym should map to type");
        assert!(
            out.contains("interface Printable"),
            "class should map to interface"
        );
        assert!(
            out.contains("class Printable"),
            "instance should map to class"
        );
        assert!(out.contains("fn add"), "function should map to fn");
    }

    #[test]
    fn test_haskell_empty_file() {
        let result = outline("", Lang::Haskell, 100, None);
        assert!(
            result.is_empty(),
            "empty Haskell file should produce empty outline"
        );
    }

    #[test]
    fn test_haskell_import_source_extraction() {
        assert_eq!(
            extract_import_source("import Data.Map.Strict"),
            "Data.Map.Strict"
        );
        assert_eq!(
            extract_import_source("import qualified Data.Text as T"),
            "Data.Text"
        );
        assert_eq!(
            extract_import_source("import React from \"react\""),
            "react"
        );
        assert_eq!(
            extract_import_source("import { useState } from \"react\""),
            "react"
        );
    }

    #[test]
    fn test_rescript_outline_declarations() {
        let source = r#"let name = \"hello\"

let add = (x, y) => x + y

type color = Red | Green | Blue

module Utils = {
  let helper = () => \"help\"
}

external alert: string => unit = \"alert\"

open Belt

exception NotFound(string)
"#;

        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        let items: Vec<(OutlineKind, &str)> = entries
            .iter()
            .map(|e| (e.kind, e.name.as_str()))
            .collect();

        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Variable && *n == "name"),
            "let value should map to Variable"
        );
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Function && *n == "add"),
            "let function should map to Function"
        );
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::TypeAlias && *n == "color"),
            "type should map to TypeAlias"
        );
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Module && *n == "Utils"),
            "module should map to Module"
        );
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Function && *n == "alert"),
            "external should map to Function"
        );
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Import && *n == "Belt"),
            "open should map to Import"
        );
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Enum && *n == "NotFound"),
            "exception should map to Enum"
        );
    }

    #[test]
    fn test_rescript_empty_file() {
        let result = outline("", Lang::ReScript, 100, None);
        assert!(
            result.is_empty(),
            "empty ReScript file should produce empty outline"
        );
    }

    #[test]
    fn test_rescript_open_import_source() {
        assert_eq!(extract_import_source("open Belt"), "Belt");
        assert_eq!(extract_import_source("open RescriptCore"), "RescriptCore");
    }

    #[test]
    fn test_rescript_interface_file() {
        let source = r#"type color = Red | Green | Blue
let add: (int, int) => int
external alert: string => unit = \"alert\"
"#;

        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::TypeAlias && e.name == "color"),
            "should find type declaration in interface source"
        );
    }

    #[test]
    fn test_rescript_jsx_component_indexing() {
        let source = r#"@react.component
let make = (~name: string) => {
  <div className=\"container\">
    <h1> {React.string(name)} </h1>
    <Counter count={1} />
    <> <span> {React.string(\"fragment\")} </span> </>
    <Header.Nav items={[\"a\", \"b\"]} />
    <Button {...props} />
  </div>
}
"#;

        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        assert_eq!(
            entries.len(),
            1,
            "should have one top-level component function"
        );
        let component = &entries[0];
        assert_eq!(component.kind, OutlineKind::Function);
        assert_eq!(component.name, "make");
        assert!(
            !component.children.is_empty(),
            "component should include JSX child entries"
        );

        let div = component
            .children
            .iter()
            .find(|c| c.name.contains("div"))
            .expect("should find <div>");

        assert!(
            div.children.iter().any(|c| c.name.contains("h1")),
            "should find nested <h1>"
        );
        assert!(
            div.children.iter().any(|c| c.name.contains("Counter")),
            "should find self-closing <Counter />"
        );
        assert!(
            div.children.iter().any(|c| c.name.contains("<>...")),
            "should find fragment"
        );
        assert!(
            div.children.iter().any(|c| c.name.contains("Header.Nav")),
            "should find dotted JSX tag"
        );
        assert!(
            div.children
                .iter()
                .any(|c| c.name.contains("Button") && c.name.contains("...")),
            "should mark spread props"
        );
    }

    #[test]
    fn test_rescript_non_component_no_jsx_children() {
        let source = r#"let add = (x, y) => x + y

let render = () => {
  <div> {React.string(\"not a component\")} </div>
}
"#;

        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        for entry in &entries {
            assert!(
                entry.children.is_empty(),
                "non-component '{}' should not include JSX semantic children",
                entry.name
            );
        }
    }

    #[test]
    fn test_rescript_malformed_jsx_graceful() {
        let source = r#"@react.component
let make = (~name: string) => {
  <div>
    <span>
  </div>
}
"#;

        let _ = outline(source, Lang::ReScript, 100, None);
    }

    // HASKELL_TREE_SITTER.FR-4, SC-4.1, SC-4.2: "Symbol search finds Haskell definitions"
    #[test]
    fn test_haskell_symbol_search_definitions() {
        use crate::search::symbol::DEFINITION_KINDS;

        // SC-4.1: data_type in DEFINITION_KINDS
        assert!(
            DEFINITION_KINDS.contains(&"data_type"),
            "data_type should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"newtype"),
            "newtype should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"type_synomym"),
            "type_synomym should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"class"),
            "class should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"instance"),
            "instance should be in DEFINITION_KINDS"
        );

        // SC-4.2: function/signature in DEFINITION_KINDS
        assert!(
            DEFINITION_KINDS.contains(&"function"),
            "function should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"bind"),
            "bind should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"signature"),
            "signature should be in DEFINITION_KINDS"
        );
    }

    // RESCRIPT_TREE_SITTER.FR-4, SC-4.1-4.3: "Symbol search finds ReScript definitions"
    #[test]
    fn test_rescript_symbol_search_definitions() {
        use crate::search::symbol::DEFINITION_KINDS;

        // SC-4.1: type_declaration already in DEFINITION_KINDS (for Go)
        assert!(
            DEFINITION_KINDS.contains(&"type_declaration"),
            "type_declaration should be in DEFINITION_KINDS"
        );

        // SC-4.2: let_declaration in DEFINITION_KINDS
        assert!(
            DEFINITION_KINDS.contains(&"let_declaration"),
            "let_declaration should be in DEFINITION_KINDS"
        );

        // SC-4.3: module_declaration in DEFINITION_KINDS
        assert!(
            DEFINITION_KINDS.contains(&"module_declaration"),
            "module_declaration should be in DEFINITION_KINDS"
        );

        // Also check external and exception
        assert!(
            DEFINITION_KINDS.contains(&"external_declaration"),
            "external_declaration should be in DEFINITION_KINDS"
        );
        assert!(
            DEFINITION_KINDS.contains(&"exception_declaration"),
            "exception_declaration should be in DEFINITION_KINDS"
        );
    }

    // RESCRIPT_TREE_SITTER.FR-3: Record type declarations
    #[test]
    fn test_rescript_record_type() {
        let source = r#"type user = {name: string, age: int, email: string}

type config = {
  debug: bool,
  port: int,
  host: string,
}
"#;
        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        assert_eq!(entries.len(), 2, "Should find 2 type declarations");
        assert!(entries.iter().all(|e| e.kind == OutlineKind::TypeAlias));
        assert!(entries.iter().any(|e| e.name == "user"));
        assert!(entries.iter().any(|e| e.name == "config"));
    }

    // RESCRIPT_TREE_SITTER.FR-3: Nested module declarations
    #[test]
    fn test_rescript_nested_modules() {
        let source = r#"module Outer = {
  let x = 1

  module Inner = {
    let y = 2
  }
}
"#;
        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        assert_eq!(entries.len(), 1, "Should find 1 top-level module");
        assert_eq!(entries[0].name, "Outer");
        assert_eq!(entries[0].kind, OutlineKind::Module);
    }

    // EDGE-2: "Decorated declarations without @react.component parse correctly"
    #[test]
    fn test_rescript_decorated_non_component() {
        let source = r#"@module("fs")
external readFile: string => string = "readFileSync"

@deprecated("Use newHelper instead")
let oldHelper = () => "old"

@genType
type status = Active | Inactive
"#;
        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        // All declarations should be present regardless of decorators
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::Function && e.name == "readFile"),
            "external with @module should be found"
        );
        assert!(
            entries.iter().any(|e| e.name == "oldHelper"),
            "@deprecated function should be found"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::TypeAlias && e.name == "status"),
            "@genType type should be found"
        );

        // No JSX children should be collected for non-@react.component functions
        for entry in &entries {
            assert!(
                entry.children.is_empty(),
                "Non-@react.component '{}' should have no JSX children",
                entry.name
            );
        }
    }

    // RESCRIPT_TREE_SITTER.FR-7: Multiple @react.component in one file
    #[test]
    fn test_rescript_multiple_components() {
        let source = r#"@react.component
let header = (~title: string) => {
  <nav> <h1> {React.string(title)} </h1> </nav>
}

let helper = () => "not a component"

@react.component
let footer = (~copyright: string) => {
  <footer> {React.string(copyright)} </footer>
}
"#;
        let entries = {
            let lang_ts = outline_language(Lang::ReScript).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::ReScript)
        };

        assert_eq!(entries.len(), 3, "Should find 3 declarations");

        let header = entries.iter().find(|e| e.name == "header").unwrap();
        assert!(
            !header.children.is_empty(),
            "header component should have JSX children"
        );

        let helper = entries.iter().find(|e| e.name == "helper").unwrap();
        assert!(
            helper.children.is_empty(),
            "helper should have no JSX children"
        );

        let footer = entries.iter().find(|e| e.name == "footer").unwrap();
        assert!(
            !footer.children.is_empty(),
            "footer component should have JSX children"
        );
    }

    // HASKELL_TREE_SITTER.FR-3: Complex Haskell module with GADTs-like syntax
    #[test]
    fn test_haskell_complex_module() {
        let source = r#"module Data.Config where

import Control.Monad.IO.Class
import qualified Data.Text as T
import Data.Map.Strict (Map, fromList, lookup)

data AppConfig = AppConfig
  { configPort :: Int
  , configHost :: String
  , configDebug :: Bool
  }

newtype AppM a = AppM { runAppM :: IO a }

type Handler = AppConfig -> IO ()

class HasConfig a where
  getConfig :: a -> AppConfig

instance HasConfig AppConfig where
  getConfig = id

startApp :: AppConfig -> IO ()
startApp config = putStrLn "Starting..."
"#;
        let entries = {
            let lang_ts = outline_language(Lang::Haskell).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::Haskell)
        };

        // Check we get a good outline of this complex module
        assert!(
            entries.len() >= 7,
            "Should find at least 7 declarations (3 imports + data + newtype + type + class + instance + function)"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::Enum && e.name == "AppConfig"),
            "data AppConfig should map to Enum"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::Struct && e.name == "AppM"),
            "newtype AppM should map to Struct"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::TypeAlias && e.name == "Handler"),
            "type Handler should map to TypeAlias"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::Interface && e.name == "HasConfig"),
            "class HasConfig should map to Interface"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::Function && e.name == "startApp"),
            "function startApp should map to Function"
        );
    }
}

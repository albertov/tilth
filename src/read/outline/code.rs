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

    format_entries(&entries, &lines, max_lines)
}

/// Get the tree-sitter Language for a given Lang variant.
pub fn outline_language(lang: Lang) -> Option<tree_sitter::Language> {
    let lang = match lang {
        Lang::Rust => tree_sitter_rust::LANGUAGE,
        Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX,
        Lang::JavaScript => tree_sitter_javascript::LANGUAGE,
        Lang::Python => tree_sitter_python::LANGUAGE,
        Lang::Go => tree_sitter_go::LANGUAGE,
        Lang::Java => tree_sitter_java::LANGUAGE,
        Lang::C => tree_sitter_c::LANGUAGE,
        Lang::Cpp => tree_sitter_cpp::LANGUAGE,
        Lang::Ruby => tree_sitter_ruby::LANGUAGE,
        Lang::Haskell => tree_sitter_haskell::LANGUAGE,
        Lang::ReScript => tree_sitter_rescript::LANGUAGE,
        // Languages without shipped grammars — fall back
        Lang::Swift | Lang::Kotlin | Lang::CSharp | Lang::Dockerfile | Lang::Make => {
            return None;
        }
    };
    Some(lang.into())
}

/// Find the first child with a specific node kind and return its text.
fn first_child_by_kind(node: tree_sitter::Node, kind: &str, lines: &[&str]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            return Some(node_text(child, lines));
        }
    }
    None
}

/// Extract name from ReScript declaration nodes.
/// Handles the _declaration → _binding → name/pattern nesting pattern.
fn rescript_binding_name(node: tree_sitter::Node, lines: &[&str]) -> Option<String> {
    match node.kind() {
        "let_declaration" => {
            // let_declaration → let_binding → pattern(value_identifier)
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "let_binding" {
                    return find_child_text(child, "pattern", lines)
                        .or_else(|| find_child_text(child, "name", lines));
                }
            }
            None
        }
        "type_declaration" => {
            // type_declaration → type_binding → name(type_identifier)
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_binding" {
                    return find_child_text(child, "name", lines);
                }
            }
            None
        }
        "module_declaration" => {
            // module_declaration → module_binding → name(module_identifier)
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "module_binding" {
                    return find_child_text(child, "name", lines);
                }
            }
            None
        }
        "external_declaration" => {
            // FLAT: direct child value_identifier
            first_child_by_kind(node, "value_identifier", lines)
        }
        "exception_declaration" => {
            // FLAT: direct child variant_identifier
            first_child_by_kind(node, "variant_identifier", lines)
        }
        "open_statement" => {
            // direct child module_identifier
            first_child_by_kind(node, "module_identifier", lines)
        }
        _ => None,
    }
}

/// Check if a ReScript let_binding has a function body (arrow or function expression).
fn rescript_let_is_function(node: tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "let_binding" {
            if let Some(body) = child.child_by_field_name("body") {
                return matches!(body.kind(), "function" | "arrow_function");
            }
        }
    }
    false
}

/// Walk top-level children of the root node, extracting outline entries.
pub(crate) fn walk_top_level(
    root: tree_sitter::Node,
    lines: &[&str],
    lang: Lang,
) -> Vec<OutlineEntry> {
    let mut entries = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        // Haskell wraps declarations in `declarations` and `imports` wrapper nodes
        if matches!(lang, Lang::Haskell) && matches!(child.kind(), "declarations" | "imports") {
            let mut inner_cursor = child.walk();
            for inner in child.children(&mut inner_cursor) {
                if let Some(entry) = node_to_entry(inner, lines, lang, 0) {
                    entries.push(entry);
                }
            }
        } else if let Some(entry) = node_to_entry(child, lines, lang, 0) {
            entries.push(entry);
        }
    }

    entries
}

/// Convert a tree-sitter node to an `OutlineEntry` based on its kind.
fn node_to_entry(
    node: tree_sitter::Node,
    lines: &[&str],
    lang: Lang,
    depth: usize,
) -> Option<OutlineEntry> {
    let kind_str = node.kind();
    let start_line = node.start_position().row as u32 + 1;
    let end_line = node.end_position().row as u32 + 1;

    let (kind, name, signature) = match kind_str {
        // Functions
        "function_declaration"
        | "function_definition"
        | "function_item"
        | "method_definition"
        | "method_declaration" => {
            let name = find_child_text(node, "name", lines)
                .or_else(|| find_child_text(node, "identifier", lines))
                .unwrap_or_else(|| "<anonymous>".into());
            let sig = extract_signature(node, lines);
            (OutlineKind::Function, name, Some(sig))
        }

        // Classes & structs
        "class_declaration" | "class_definition" => {
            let name = find_child_text(node, "name", lines)
                .or_else(|| find_child_text(node, "identifier", lines))
                .unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Class, name, None)
        }
        "struct_item" | "struct_declaration" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Struct, name, None)
        }

        // Interfaces & types
        "interface_declaration" | "type_alias_declaration" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Interface, name, None)
        }
        "type_item" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::TypeAlias, name, None)
        }

        // Enums
        "enum_item" | "enum_declaration" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Enum, name, None)
        }

        // Impl blocks (Rust)
        "impl_item" => {
            let name = find_child_text(node, "type", lines).unwrap_or_else(|| "<impl>".into());
            (OutlineKind::Module, format!("impl {name}"), None)
        }

        // Constants and variables
        "const_item" | "static_item" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<const>".into());
            (OutlineKind::Constant, name, None)
        }
        "lexical_declaration" | "variable_declaration" => {
            let name = first_identifier_text(node, lines).unwrap_or_else(|| "<var>".into());
            (OutlineKind::Variable, name, None)
        }

        // Imports — collect as a group
        "import_statement" | "import_declaration" | "use_declaration" | "use_item" => {
            let text = node_text(node, lines);
            (OutlineKind::Import, text, None)
        }

        // Exports
        "export_statement" => {
            let name = node_text(node, lines);
            (OutlineKind::Export, name, None)
        }

        // Module declarations
        "mod_item" | "module" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<module>".into());
            (OutlineKind::Module, name, None)
        }

        // Haskell: functions and type signatures
        "function" | "bind" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            let sig = extract_signature(node, lines);
            (OutlineKind::Function, name, Some(sig))
        }
        "signature" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            let sig = extract_signature(node, lines);
            (OutlineKind::Function, name, Some(sig))
        }

        // Haskell: data types
        "data_type" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Enum, name, None)
        }

        // Haskell: newtype
        "newtype" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Struct, name, None)
        }

        // Haskell: type alias (NOTE: misspelled in grammar as "type_synomym")
        "type_synomym" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::TypeAlias, name, None)
        }

        // Haskell: type class
        "class" => {
            let name = find_child_text(node, "name", lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::Interface, name, None)
        }

        // Haskell: type class instance
        "instance" => {
            let class_name =
                find_child_text(node, "name", lines).unwrap_or_else(|| "<instance>".into());
            let type_name = find_child_text(node, "patterns", lines).unwrap_or_default();
            let display_name = if type_name.is_empty() {
                class_name
            } else {
                format!("{class_name} {type_name}")
            };
            (OutlineKind::Class, display_name, None)
        }

        // Haskell: foreign import (nested: foreign_import → signature → name)
        "foreign_import" => {
            let name = node
                .child_by_field_name("signature")
                .and_then(|sig| find_child_text(sig, "name", lines))
                .unwrap_or_else(|| node_text(node, lines));
            (OutlineKind::Import, name, None)
        }

        // Haskell: import declaration
        "import" => {
            let text = node_text(node, lines);
            (OutlineKind::Import, text, None)
        }

        // ReScript: let declarations (function or variable)
        "let_declaration" => {
            let name = rescript_binding_name(node, lines).unwrap_or_else(|| "<anonymous>".into());
            if rescript_let_is_function(node) {
                let sig = extract_signature(node, lines);
                (OutlineKind::Function, name, Some(sig))
            } else {
                (OutlineKind::Variable, name, None)
            }
        }

        // ReScript: type declarations (guarded to avoid Go conflicts)
        "type_declaration" if matches!(lang, Lang::ReScript) => {
            let name = rescript_binding_name(node, lines).unwrap_or_else(|| "<anonymous>".into());
            (OutlineKind::TypeAlias, name, None)
        }

        // ReScript: module declarations
        "module_declaration" => {
            let name = rescript_binding_name(node, lines).unwrap_or_else(|| "<module>".into());
            (OutlineKind::Module, name, None)
        }

        // ReScript: external declarations
        "external_declaration" => {
            let name = rescript_binding_name(node, lines).unwrap_or_else(|| "<external>".into());
            (OutlineKind::Function, name, None)
        }

        // ReScript: open statements → imports
        "open_statement" => {
            let name = rescript_binding_name(node, lines).unwrap_or_else(|| node_text(node, lines));
            (OutlineKind::Import, name, None)
        }

        // ReScript: exception declarations
        "exception_declaration" => {
            let name = rescript_binding_name(node, lines).unwrap_or_else(|| "<exception>".into());
            (OutlineKind::Enum, name, None)
        }

        _ => return None,
    };

    // Collect children for classes, impls, modules
    let children = if matches!(
        kind,
        OutlineKind::Class | OutlineKind::Struct | OutlineKind::Module
    ) && depth < 1
    {
        collect_children(node, lines, lang, depth + 1)
    } else {
        Vec::new()
    };

    // Collect JSX children for @react.component functions
    let children = if matches!(lang, Lang::ReScript)
        && node.kind() == "let_declaration"
        && matches!(kind, OutlineKind::Function)
    {
        // Check for @react.component decorator (sibling-based)
        if node.prev_sibling().map_or(false, |prev| {
            prev.kind() == "decorator" && node_text(prev, lines).contains("@react.component")
        }) {
            collect_jsx_children(node, lines)
        } else {
            children
        }
    } else {
        children
    };

    // Extract doc comment if present
    let doc = extract_doc(node, lines);

    Some(OutlineEntry {
        kind,
        name,
        start_line,
        end_line,
        signature,
        children,
        doc,
    })
}

/// Collect child entries from a class/struct/impl body.
fn collect_children(
    node: tree_sitter::Node,
    lines: &[&str],
    lang: Lang,
    depth: usize,
) -> Vec<OutlineEntry> {
    let mut children = Vec::new();
    let mut cursor = node.walk();

    // Look for a body node first
    let body = node
        .children(&mut cursor)
        .find(|c| c.kind().contains("body") || c.kind().contains("block"));

    let parent = body.unwrap_or(node);
    let mut cursor2 = parent.walk();

    for child in parent.children(&mut cursor2) {
        if let Some(entry) = node_to_entry(child, lines, lang, depth) {
            children.push(entry);
        }
    }

    children
}

/// Extract the first line as a function signature (name + params + return type).
fn extract_signature(node: tree_sitter::Node, lines: &[&str]) -> String {
    let start_row = node.start_position().row;
    if start_row < lines.len() {
        let line = lines[start_row].trim();
        // Truncate at opening brace
        if let Some(pos) = line.find('{') {
            return line[..pos].trim().to_string();
        }
        if line.ends_with(':') {
            // Python — truncate at trailing colon (for `def foo(x: int):` etc.)
            if let Some(pos) = line.rfind(':') {
                return line[..pos].trim().to_string();
            }
        }
        // Full first line, truncated
        if line.len() > 120 {
            format!("{}...", crate::types::truncate_str(line, 117))
        } else {
            line.to_string()
        }
    } else {
        String::new()
    }
}

/// Find a named child and return its text.
fn find_child_text(node: tree_sitter::Node, field: &str, lines: &[&str]) -> Option<String> {
    node.child_by_field_name(field).map(|n| node_text(n, lines))
}

/// Get the text of a node, truncated to the first line.
fn node_text(node: tree_sitter::Node, lines: &[&str]) -> String {
    let row = node.start_position().row;
    let col_start = node.start_position().column;
    let end_row = node.end_position().row;

    if row < lines.len() {
        if row == end_row {
            let col_end = node.end_position().column.min(lines[row].len());
            lines[row][col_start..col_end].to_string()
        } else {
            // Multi-line — take first line only, truncated
            let text = &lines[row][col_start..];
            if text.len() > 80 {
                format!("{}...", crate::types::truncate_str(text, 77))
            } else {
                text.to_string()
            }
        }
    } else {
        String::new()
    }
}

/// Find the first identifier-like child.
fn first_identifier_text(node: tree_sitter::Node, lines: &[&str]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind.contains("identifier") || kind.contains("name") || kind.contains("declarator") {
            let text = node_text(child, lines);
            if !text.is_empty() {
                return Some(text);
            }
            // Recurse one level for variable_declarator → identifier
            let mut inner = child.walk();
            for grandchild in child.children(&mut inner) {
                if grandchild.kind().contains("identifier") {
                    let text = node_text(grandchild, lines);
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }
    }
    None
}

/// Extract a doc comment from the previous sibling.
fn extract_doc(node: tree_sitter::Node, lines: &[&str]) -> Option<String> {
    let prev = node.prev_sibling()?;
    let kind = prev.kind();
    if kind.contains("comment") || kind.contains("doc") {
        let text = node_text(prev, lines);
        let trimmed = text
            .trim_start_matches("///")
            .trim_start_matches("//!")
            .trim_start_matches("/**")
            .trim_start_matches('#')
            .trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    }
}

/// Format outline entries into the spec'd output format.
fn format_entries(entries: &[OutlineEntry], _lines: &[&str], max_lines: usize) -> String {
    let mut out = Vec::new();
    let mut import_groups: Vec<&str> = Vec::new();

    for entry in entries {
        if out.len() >= max_lines {
            break;
        }

        match entry.kind {
            OutlineKind::Import => {
                import_groups.push(&entry.name);
                continue;
            }
            _ => {
                // Flush any accumulated imports
                if !import_groups.is_empty() {
                    out.push(format_imports(&import_groups, entries.first()));
                    import_groups.clear();
                }
            }
        }

        out.push(format_entry(entry, 0));

        for child in &entry.children {
            if out.len() >= max_lines {
                break;
            }
            out.push(format_entry(child, 1));
        }
    }

    // Flush trailing imports
    if !import_groups.is_empty() {
        out.push(format_imports(&import_groups, entries.first()));
    }

    out.join("\n")
}

/// Format a collapsed import summary grouped by source with counts.
/// Spec format: `imports: react(4), express(2), @/lib(3)`
fn format_imports(imports: &[&str], first_entry: Option<&OutlineEntry>) -> String {
    let start = first_entry.map_or(1, |e| e.start_line);
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

/// Extract the source module name from an import statement text.
/// Handles: `use std::fs;` → `std::fs`, `import X from "react"` → `react`,
/// `from collections import X` → `collections`
pub(crate) fn extract_import_source(text: &str) -> String {
    let trimmed = text.trim().trim_end_matches(';');

    // Rust: `use foo::bar` → `foo::bar`
    if let Some(rest) = trimmed.strip_prefix("use ") {
        return rest
            .split('{')
            .next()
            .unwrap_or(rest)
            .trim()
            .trim_end_matches("::")
            .to_string();
    }

    // ReScript: `open Module` → `Module`
    if let Some(rest) = trimmed.strip_prefix("open ") {
        return rest.split_whitespace().next().unwrap_or("").to_string();
    }

    // Python: `from module import ...` or `import module`
    if let Some(rest) = trimmed.strip_prefix("from ") {
        return rest.split_whitespace().next().unwrap_or("").to_string();
    }
    // Haskell: `import [qualified] Module [as Alias] [(items)]`
    // Haskell imports never contain `from` — JS/TS does (`import X from "source"`)
    if let Some(rest) = trimmed.strip_prefix("import ") {
        if !trimmed.contains(" from ") && !trimmed.contains(" from\"") {
            let rest = rest.strip_prefix("qualified ").unwrap_or(rest);
            if let Some(module) = rest.split_whitespace().next() {
                if module.chars().next().map_or(false, |c| c.is_uppercase()) {
                    return module.to_string();
                }
            }
        }
    }
    // JS/TS: `import ... from "source"` or `import "source"`
    if trimmed.starts_with("import") {
        if let Some(from_pos) = trimmed.find("from ") {
            let source = &trimmed[from_pos + 5..];
            return source
                .trim()
                .trim_matches(|c| c == '"' || c == '\'' || c == ';')
                .to_string();
        }
        // Direct import: `import "source"`
        let after = trimmed.strip_prefix("import ").unwrap_or("");
        return after
            .trim()
            .trim_matches(|c| c == '"' || c == '\'' || c == ';')
            .to_string();
    }
    // Generic Python-style `import module` (after Haskell handler)
    if let Some(rest) = trimmed.strip_prefix("import ") {
        return rest.split_whitespace().next().unwrap_or("").to_string();
    }

    // C/C++: #include "file.h" or #include <header>
    if let Some(rest) = trimmed.strip_prefix("#include") {
        return rest.trim().to_string(); // preserves quotes/angles for external detection
    }

    // Go: `import "source"` — already handled above via "import"
    // Fallback: first meaningful token
    trimmed
        .split_whitespace()
        .last()
        .unwrap_or(trimmed)
        .to_string()
}

/// Format a single outline entry with optional indentation.
fn format_entry(entry: &OutlineEntry, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    let range = if entry.start_line == entry.end_line {
        format!("[{}]", entry.start_line)
    } else {
        format!("[{}-{}]", entry.start_line, entry.end_line)
    };

    let kind_label = match entry.kind {
        OutlineKind::Function => "fn",
        OutlineKind::Method => "method",
        OutlineKind::Class => "class",
        OutlineKind::Struct => "struct",
        OutlineKind::Interface => "interface",
        OutlineKind::TypeAlias => "type",
        OutlineKind::Enum => "enum",
        OutlineKind::Constant => "const",
        OutlineKind::Variable => "let",
        OutlineKind::Export => "export",
        OutlineKind::Property => "prop",
        OutlineKind::Module => "mod",
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

/// Collect JSX element children from a ReScript component function.
/// Walks the entire function body tree looking for JSX elements.
fn collect_jsx_children(node: tree_sitter::Node, lines: &[&str]) -> Vec<OutlineEntry> {
    let mut entries = Vec::new();
    collect_jsx_recursive(node, lines, &mut entries);
    entries
}

fn collect_jsx_recursive(node: tree_sitter::Node, lines: &[&str], entries: &mut Vec<OutlineEntry>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "jsx_element" => {
                // Extract tag name from jsx_opening_element
                let tag = jsx_tag_name(child, lines);
                let mut entry = OutlineEntry {
                    kind: OutlineKind::Property, // Use Property for JSX tags
                    name: format!("<{tag}>"),
                    start_line: (child.start_position().row + 1) as u32,
                    end_line: (child.end_position().row + 1) as u32,
                    signature: None,
                    children: Vec::new(),
                    doc: None,
                };
                // Recurse into jsx_element children for nested JSX
                collect_jsx_recursive(child, lines, &mut entry.children);
                entries.push(entry);
            }
            "jsx_self_closing_element" => {
                let tag = jsx_self_closing_tag(child, lines);
                // Check for spread props
                let has_spread = has_jsx_spread(child);
                let name = if has_spread {
                    format!("<{tag} .../>")
                } else {
                    format!("<{tag} />")
                };
                entries.push(OutlineEntry {
                    kind: OutlineKind::Property,
                    name,
                    start_line: (child.start_position().row + 1) as u32,
                    end_line: (child.end_position().row + 1) as u32,
                    signature: None,
                    children: Vec::new(),
                    doc: None,
                });
            }
            "jsx_fragment" => {
                let mut entry = OutlineEntry {
                    kind: OutlineKind::Property,
                    name: "<>...</>".to_string(),
                    start_line: (child.start_position().row + 1) as u32,
                    end_line: (child.end_position().row + 1) as u32,
                    signature: None,
                    children: Vec::new(),
                    doc: None,
                };
                collect_jsx_recursive(child, lines, &mut entry.children);
                entries.push(entry);
            }
            _ => {
                // Recurse into non-JSX nodes looking for JSX
                collect_jsx_recursive(child, lines, entries);
            }
        }
    }
}

/// Extract tag name from a jsx_element (from its jsx_opening_element child).
fn jsx_tag_name(node: tree_sitter::Node, lines: &[&str]) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "jsx_opening_element" {
            return jsx_identifier_text(child, lines);
        }
    }
    "unknown".to_string()
}

/// Extract tag name from a jsx_self_closing_element.
fn jsx_self_closing_tag(node: tree_sitter::Node, lines: &[&str]) -> String {
    jsx_identifier_text(node, lines)
}

/// Extract the identifier text from a JSX element node (handles dotted tags).
fn jsx_identifier_text(node: tree_sitter::Node, lines: &[&str]) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "nested_jsx_identifier" => {
                // Dotted tag: Header.Nav -> collect all jsx_identifier children, join with "."
                let mut parts = Vec::new();
                let mut inner_cursor = child.walk();
                for inner in child.children(&mut inner_cursor) {
                    if inner.kind() == "jsx_identifier" {
                        parts.push(node_text(inner, lines));
                    }
                }
                return parts.join(".");
            }
            "jsx_identifier" => {
                return node_text(child, lines);
            }
            _ => {}
        }
    }
    "unknown".to_string()
}

/// Check if a JSX element has spread props ({...expr}).
fn has_jsx_spread(node: tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "jsx_expression" {
            let mut inner = child.walk();
            for inner_child in child.children(&mut inner) {
                if inner_child.kind() == "spread_element" {
                    return true;
                }
            }
        }
    }
    false
}

/// Fallback when tree-sitter grammar isn't available.
fn fallback_outline(content: &str, _max_lines: usize) -> String {
    super::fallback::head_tail(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    // HASKELL_TREE_SITTER.FR-2, SC-2.1: "outline_language(Haskell) → grammar loads + parser init"
    #[test]
    fn test_haskell_grammar_loads() {
        let lang = outline_language(Lang::Haskell);
        assert!(lang.is_some(), "Haskell grammar should be available");
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang.unwrap())
            .expect("Haskell grammar should initialize parser");
    }

    // RESCRIPT_TREE_SITTER.FR-2, SC-2.1: "outline_language(ReScript) → grammar loads + parser init"
    #[test]
    fn test_rescript_grammar_loads() {
        let lang = outline_language(Lang::ReScript);
        assert!(lang.is_some(), "ReScript grammar should be available");
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang.unwrap())
            .expect("ReScript grammar should initialize parser");
    }

    // HASKELL_TREE_SITTER.FR-3, SC-3.1-SC-3.5: "Outline extracts all Haskell declaration types"
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
        let entries = {
            let lang_ts = outline_language(Lang::Haskell).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::Haskell)
        };

        // Collect (kind, name) pairs
        let items: Vec<(OutlineKind, &str)> = entries
            .iter()
            .map(|e| (e.kind.clone(), e.name.as_str()))
            .collect();

        // SC-3.5: declarations wrapper is transparent — we get actual decl nodes
        assert!(
            !entries.is_empty(),
            "Should extract declarations through wrapper"
        );

        // Imports
        assert!(
            items.iter().any(|(k, _)| *k == OutlineKind::Import),
            "Should have imports"
        );

        // SC-3.2: data_type → Enum
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Enum && *n == "Color"),
            "data_type 'Color' should map to Enum"
        );

        // SC-3.4: newtype → Struct
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Struct && *n == "Name"),
            "newtype 'Name' should map to Struct"
        );

        // SC-3.4: type_synomym → TypeAlias
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::TypeAlias && *n == "Alias"),
            "type synonym 'Alias' should map to TypeAlias"
        );

        // SC-3.3: class → Interface
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Interface && *n == "Printable"),
            "class 'Printable' should map to Interface"
        );

        // SC-3.3: instance → Class
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Class && n.contains("Printable")),
            "instance should map to Class with class name"
        );

        // SC-3.1: function → Function
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Function && *n == "add"),
            "function 'add' should map to Function"
        );
    }

    // EDGE-1: "Empty Haskell file → empty outline"
    #[test]
    fn test_haskell_empty_file() {
        let source = "";
        let result = outline(source, Lang::Haskell, 100, None);
        assert!(
            result.is_empty(),
            "Empty Haskell file should produce empty outline"
        );
    }

    // HASKELL_TREE_SITTER.FR-5, SC-5.1: "import source extraction for Haskell"
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
    }

    // Regression: Haskell import handler must not steal JS/TS imports
    #[test]
    fn test_import_source_js_not_stolen_by_haskell() {
        // JS/TS: import React from "react" → source should be "react", not "React"
        assert_eq!(
            extract_import_source(r#"import React from "react""#),
            "react"
        );
        assert_eq!(
            extract_import_source(r#"import { useState } from "react""#),
            "react"
        );
    }

    // RESCRIPT_TREE_SITTER.FR-3, SC-3.1-SC-3.6: "Outline extracts all ReScript declaration types"
    #[test]
    fn test_rescript_outline_declarations() {
        let source = r#"let name = "hello"

let add = (x, y) => x + y

type color = Red | Green | Blue

module Utils = {
  let helper = () => "help"
}

external alert: string => unit = "alert"

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
            .map(|e| (e.kind.clone(), e.name.as_str()))
            .collect();

        assert!(!entries.is_empty(), "Should extract ReScript declarations");

        // SC-3.1: let value → Variable
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Variable && *n == "name"),
            "let value 'name' should map to Variable"
        );

        // SC-3.1: let function → Function
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Function && *n == "add"),
            "let function 'add' should map to Function"
        );

        // SC-3.2: type_declaration → TypeAlias
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::TypeAlias && *n == "color"),
            "type 'color' should map to TypeAlias"
        );

        // SC-3.3: module_declaration → Module
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Module && *n == "Utils"),
            "module 'Utils' should map to Module"
        );

        // SC-3.4: external_declaration → Function
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Function && *n == "alert"),
            "external 'alert' should map to Function"
        );

        // SC-3.5: open_statement → Import
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Import && *n == "Belt"),
            "open 'Belt' should map to Import"
        );

        // SC-3.6: exception_declaration → Enum
        assert!(
            items
                .iter()
                .any(|(k, n)| *k == OutlineKind::Enum && *n == "NotFound"),
            "exception 'NotFound' should map to Enum"
        );
    }

    // EDGE-3: "Empty ReScript file → empty outline"
    #[test]
    fn test_rescript_empty_file() {
        let source = "";
        let result = outline(source, Lang::ReScript, 100, None);
        assert!(
            result.is_empty(),
            "Empty ReScript file should produce empty outline"
        );
    }

    // RESCRIPT_TREE_SITTER.FR-5, SC-5.1: "import source extraction for ReScript open"
    #[test]
    fn test_rescript_open_import_source() {
        assert_eq!(extract_import_source("open Belt"), "Belt");
        assert_eq!(extract_import_source("open RescriptCore"), "RescriptCore");
    }

    // EDGE-1, SC-E1.1: ".resi interface files parse correctly"
    #[test]
    fn test_rescript_interface_file() {
        let source = r#"type color = Red | Green | Blue
let add: (int, int) => int
external alert: string => unit = "alert"
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
            !entries.is_empty(),
            "ReScript interface file should produce entries"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.kind == OutlineKind::TypeAlias && e.name == "color"),
            "Should find type declaration in .resi"
        );
    }

    // RESCRIPT_TREE_SITTER.FR-6, SC-6.1-6.3: "@react.component parses and produces outline with JSX"
    // RESCRIPT_TREE_SITTER.FR-7, SC-7.1-7.5: "JSX-semantic indexing"
    #[test]
    fn test_rescript_jsx_component_indexing() {
        let source = r#"@react.component
let make = (~name: string) => {
  <div className="container">
    <h1> {React.string(name)} </h1>
    <Counter count={1} />
    <> <span> {React.string("fragment")} </span> </>
    <Header.Nav items={["a", "b"]} />
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

        // SC-6.1: @react.component function produces outline entry
        assert_eq!(
            entries.len(),
            1,
            "Should have one top-level entry (the component)"
        );
        let component = &entries[0];
        assert_eq!(component.kind, OutlineKind::Function);
        assert_eq!(component.name, "make");

        // SC-7.1: JSX tags are indexed as children
        assert!(
            !component.children.is_empty(),
            "Component should have JSX children"
        );

        // Should have <div> as top-level JSX child
        let div = component.children.iter().find(|c| c.name.contains("div"));
        assert!(div.is_some(), "Should find <div> JSX element");
        let div = div.unwrap();

        // SC-7.4: Nested JSX children
        assert!(
            !div.children.is_empty(),
            "div should have nested JSX children"
        );

        // Check for h1
        assert!(
            div.children.iter().any(|c| c.name.contains("h1")),
            "Should find <h1>"
        );

        // Check for Counter (self-closing)
        assert!(
            div.children.iter().any(|c| c.name.contains("Counter")),
            "Should find <Counter>"
        );

        // SC-7.2: Fragment
        assert!(
            div.children.iter().any(|c| c.name.contains("<>...")),
            "Should find fragment"
        );

        // SC-6.3, SC-7.3: Dotted tag
        assert!(
            div.children.iter().any(|c| c.name.contains("Header.Nav")),
            "Should find dotted tag <Header.Nav>"
        );

        // SC-7.3: Spread props
        assert!(
            div.children
                .iter()
                .any(|c| c.name.contains("Button") && c.name.contains("...")),
            "Should find <Button .../> with spread indication"
        );
    }

    // SC-7.5: "Non-JSX functions are not affected"
    #[test]
    fn test_rescript_non_component_no_jsx_children() {
        let source = r#"let add = (x, y) => x + y

let render = () => {
  <div> {React.string("not a component")} </div>
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

        // Both functions should have NO JSX children since no @react.component
        for entry in &entries {
            assert!(
                entry.children.is_empty(),
                "Function '{}' without @react.component should have no JSX children",
                entry.name
            );
        }
    }

    // EDGE-4: "Malformed JSX degrades gracefully"
    #[test]
    fn test_rescript_malformed_jsx_graceful() {
        let source = r#"@react.component
let make = (~name: string) => {
  <div>
    <span>
  </div>
}
"#;
        // Should not panic — graceful degradation
        let result = outline(source, Lang::ReScript, 100, None);
        // May produce partial results or empty, but should not crash
        let _ = result;
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

    // HASKELL_PRAGMAS_ONLY.12: Pragmas-only Haskell file should not crash and return empty outline
    #[test]
    fn test_haskell_pragmas_only() {
        let source = r#"{-# LANGUAGE ScopedTypeVariables #-}
{-# LANGUAGE DeriveGeneric #-}
{-# OPTIONS_GHC -Wall #-}
"#;
        let result = outline(source, Lang::Haskell, 100, None);

        // Should not crash - graceful handling
        assert!(
            result.is_empty() || result.trim().is_empty() || !result.contains("ERROR"),
            "Pragmas-only Haskell file should produce empty output, not crash. Got: {}",
            result
        );

        // Verify no declarations were extracted (only pragmas present)
        let entries = {
            let lang_ts = outline_language(Lang::Haskell).unwrap();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang_ts).unwrap();
            let tree = parser.parse(source, None).unwrap();
            let root = tree.root_node();
            let lines: Vec<&str> = source.lines().collect();
            walk_top_level(root, &lines, Lang::Haskell)
        };

        assert!(
            entries.is_empty(),
            "Pragmas-only file should have no declarations extracted, found {}",
            entries.len()
        );
    }
}

use std::path::Path;
use tilth::cache::OutlineCache;

fn search_fixture(symbol: &str) -> String {
    let scope = Path::new("tests/fixtures/polyglot-project");
    let cache = OutlineCache::new();
    tilth::run(symbol, scope, None, None, &cache).expect("search should succeed")
}

#[test]
fn snapshot_search_haskell_function() {
    insta::assert_snapshot!(search_fixture("tokenize"));
}

#[test]
fn snapshot_search_haskell_type() {
    insta::assert_snapshot!(search_fixture("Token"));
}

#[test]
fn snapshot_search_rescript_component() {
    // "make" is the ReScript component function name
    insta::assert_snapshot!(search_fixture("make"));
}

#[test]
fn snapshot_search_rescript_react_component() {
    // "Button" is the React component name (derived from filename)
    insta::assert_snapshot!(search_fixture("Button"));
}

#[test]
fn snapshot_search_rescript_type() {
    insta::assert_snapshot!(search_fixture("color"));
}

#[test]
fn snapshot_search_rust_function() {
    insta::assert_snapshot!(search_fixture("initialize"));
}

#[test]
fn snapshot_search_python_class() {
    insta::assert_snapshot!(search_fixture("Pipeline"));
}

#[test]
fn snapshot_search_cross_language_function() {
    // "debounce" only exists in utils.js
    insta::assert_snapshot!(search_fixture("debounce"));
}

#[test]
fn search_finds_rescript_component_by_module_name() {
    let result = search_fixture("Button");
    assert!(
        result.contains("Button.res"),
        "Searching for 'Button' should find Button.res (ReScript module name derived from filename).\nActual result:\n{}",
        result
    );
}

#[test]
fn snapshot_search_rescript_store_module() {
    insta::assert_snapshot!(search_fixture("Store"));
}

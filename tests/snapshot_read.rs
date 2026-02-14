use std::path::Path;
use tilth::cache::OutlineCache;

fn read_fixture(filename: &str) -> String {
    let scope = Path::new(".");
    let cache = OutlineCache::new();
    let query = format!("tests/fixtures/polyglot-project/src/{}", filename);
    tilth::run(&query, scope, None, None, &cache).expect("read should succeed")
}

#[test]
fn snapshot_read_haskell_outline() {
    insta::assert_snapshot!(read_fixture("Parser.hs"));
}

#[test]
fn snapshot_read_rescript_outline() {
    insta::assert_snapshot!(read_fixture("Button.res"));
}

#[test]
fn snapshot_read_rust_outline() {
    insta::assert_snapshot!(read_fixture("main.rs"));
}

#[test]
fn snapshot_read_typescript_outline() {
    insta::assert_snapshot!(read_fixture("app.ts"));
}

#[test]
fn snapshot_read_python_outline() {
    insta::assert_snapshot!(read_fixture("core.py"));
}

#[test]
fn snapshot_read_go_outline() {
    insta::assert_snapshot!(read_fixture("server.go"));
}

#[test]
fn snapshot_read_java_outline() {
    insta::assert_snapshot!(read_fixture("App.java"));
}

#[test]
fn snapshot_read_ruby_outline() {
    insta::assert_snapshot!(read_fixture("helper.rb"));
}

#[test]
fn snapshot_read_javascript_outline() {
    insta::assert_snapshot!(read_fixture("utils.js"));
}

#[test]
fn snapshot_read_tsx_outline() {
    insta::assert_snapshot!(read_fixture("component.tsx"));
}

#[test]
fn snapshot_read_rescript_large_outline() {
    insta::assert_snapshot!(read_fixture("Store.res"));
}

use std::path::Path;

// Import from the tilth crate
use tilth::cache::OutlineCache;
use tilth::map;

#[test]
fn snapshot_polyglot_map() {
    let scope = Path::new("tests/fixtures/polyglot-project");
    let cache = OutlineCache::new();
    let result = map::generate(scope, 3, None, &cache);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_polyglot_map_depth_limited() {
    let scope = Path::new("tests/fixtures/polyglot-project");
    let cache = OutlineCache::new();
    let result = map::generate(scope, 1, None, &cache);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_polyglot_map_budget_limited() {
    let scope = Path::new("tests/fixtures/polyglot-project");
    let cache = OutlineCache::new();
    let result = map::generate(scope, 3, Some(500), &cache);
    insta::assert_snapshot!(result);
}

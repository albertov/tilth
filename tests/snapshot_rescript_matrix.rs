use std::path::Path;
use tilth::cache::OutlineCache;

/// Verify we have at least 12 .res fixtures for orthogonal coverage
#[test]
fn rescript_matrix_has_minimum_fixture_count() {
    let matrix_dir = Path::new("tests/fixtures/rescript-matrix");
    let res_files: Vec<_> = std::fs::read_dir(matrix_dir)
        .expect("rescript-matrix directory should exist")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "res"))
        .collect();

    assert!(
        res_files.len() >= 12,
        "Expected at least 12 .res fixtures in tests/fixtures/rescript-matrix, found {}",
        res_files.len()
    );
}

/// Snapshot aggregated read output for all ReScript matrix fixtures
#[test]
fn snapshot_read_rescript_matrix() {
    let scope = Path::new(".");
    let cache = OutlineCache::new();
    let matrix_dir = Path::new("tests/fixtures/rescript-matrix");

    let mut fixture_paths: Vec<_> = std::fs::read_dir(matrix_dir)
        .expect("rescript-matrix directory should exist")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "res"))
        .map(|entry| entry.path())
        .collect();

    fixture_paths.sort();

    let mut aggregated_output = String::new();
    for fixture_path in &fixture_paths {
        let relative_path = fixture_path
            .strip_prefix(".")
            .unwrap_or(fixture_path)
            .to_string_lossy()
            .to_string();

        match tilth::run(&relative_path, scope, None, None, &cache) {
            Ok(output) => {
                aggregated_output.push_str("=== ");
                aggregated_output.push_str(&fixture_path.file_name().unwrap().to_string_lossy());
                aggregated_output.push_str(" ===\n");
                aggregated_output.push_str(&output);
                aggregated_output.push('\n');
            }
            Err(e) => {
                aggregated_output.push_str("=== ");
                aggregated_output.push_str(&fixture_path.file_name().unwrap().to_string_lossy());
                aggregated_output.push_str(" ===\n");
                aggregated_output.push_str(&format!("ERROR: {e}\n"));
                aggregated_output.push('\n');
            }
        }
    }

    insta::assert_snapshot!(aggregated_output);
}

/// Snapshot symbol search over the ReScript matrix directory
#[test]
fn snapshot_search_rescript_matrix() {
    let scope = Path::new("tests/fixtures/rescript-matrix");
    let cache = OutlineCache::new();

    // Search for key ReScript forms: type, let, module, exception, component
    let symbols = vec!["type", "let", "module", "exception", "make"];

    let mut aggregated_output = String::new();
    for symbol in symbols {
        match tilth::run(symbol, scope, None, None, &cache) {
            Ok(output) => {
                aggregated_output.push_str("=== Search: ");
                aggregated_output.push_str(symbol);
                aggregated_output.push_str(" ===\n");
                aggregated_output.push_str(&output);
                aggregated_output.push('\n');
            }
            Err(e) => {
                aggregated_output.push_str("=== Search: ");
                aggregated_output.push_str(symbol);
                aggregated_output.push_str(" ===\n");
                aggregated_output.push_str(&format!("ERROR: {e}\n"));
                aggregated_output.push('\n');
            }
        }
    }

    insta::assert_snapshot!(aggregated_output);
}

/// End-to-end test for .resi interface file: verify outline contains expected declarations
#[test]
fn rescript_interface_file_e2e() {
    let scope = Path::new(".");
    let cache = OutlineCache::new();
    let query = "tests/fixtures/rescript-matrix/Types.resi";

    let output =
        tilth::run(query, scope, None, None, &cache).expect("Should successfully read .resi file");

    // Verify outline contains type declarations (expected in interface files)
    assert!(
        output.contains("type"),
        "Output should contain 'type' declarations from .resi interface file"
    );

    // Find at least one type identifier in the outline
    let type_keywords = ["type", "let", "external", "module"];
    let found_keyword = type_keywords.iter().any(|kw| output.contains(kw));

    assert!(
        found_keyword,
        "Output should contain at least one of {:?}",
        type_keywords
    );
}

/// Symbol search should find symbols declared in .resi files
#[test]
fn symbol_search_finds_resi_declarations() {
    let scope = Path::new("tests/fixtures/rescript-matrix");
    let cache = OutlineCache::new();

    // Search for a type that should be defined in Types.resi
    let output = tilth::run("Color", scope, None, None, &cache)
        .expect("Should successfully search for type symbol");

    assert!(
        output.contains("Types.resi"),
        "Search results should reference Types.resi interface file"
    );
}

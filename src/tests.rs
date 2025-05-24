use std::{env, path::Path};

fn extract_version(content: &str, match_: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with(match_) {
            return line.split('"').nth(1).map(|s| s.to_string());
        }
    }
    None
}

#[test]
fn lib_version_is_updated_in_readme() {
    let lib_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let lib_content = std::fs::read_to_string(&lib_path).expect("Failed to read src/lib.rs");

    let expected_version = format!(
        "{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR")
    );
    let version = extract_version(&lib_content, "//! cucumber-thirtyfour-worlder = ")
        .expect("cucumber-thirtyfour-worlder version not found in src/lib.rs");

    assert_eq!(
        version, expected_version,
        "Version in src/lib.rs does not match CARGO_PKG_VERSION"
    );
}

#[test]
fn lib_version_is_updated_in_docref() {
    let docref_cargotoml_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docref")
        .join("Cargo.toml");
    let docref_cargotoml_content =
        std::fs::read_to_string(&docref_cargotoml_path).expect("Failed to read docref/Cargo.toml");

    let version = extract_version(&docref_cargotoml_content, "version = ")
        .expect("Version not found in docref/Cargo.toml");
    let expected_version = env!("CARGO_PKG_VERSION");

    assert_eq!(
        version, expected_version,
        "Version in docref/Cargo.toml does not match version in Cargo.toml"
    );
}

#[test]
fn cucumber_version_in_readme_is_updated_with_docref() {
    let docref_cargotoml_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docref")
        .join("Cargo.toml");
    let docref_content =
        std::fs::read_to_string(&docref_cargotoml_path).expect("Failed to read docref/Cargo.toml");

    let expected_version = extract_version(&docref_content, "cucumber = ")
        .expect("cucumber version not found in docref/Cargo.toml");

    let lib_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let lib_content = std::fs::read_to_string(&lib_path).expect("Failed to read src/lib.rs");

    let version = extract_version(&lib_content, "//! cucumber = ")
        .expect("cucumber version not found in src/lib.rs");

    assert_eq!(
        version, expected_version,
        "Cucumber version in src/lib.rs does not match docref/Cargo.toml"
    );
}

#[test]
fn thirtyfour_version_in_readme_is_updated_with_docref() {
    let docref_cargotoml_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docref")
        .join("Cargo.toml");
    let docref_content =
        std::fs::read_to_string(&docref_cargotoml_path).expect("Failed to read docref/Cargo.toml");

    let expected_version = extract_version(&docref_content, "thirtyfour = ")
        .expect("thirtyfour version not found in docref/Cargo.toml");

    let lib_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let lib_content = std::fs::read_to_string(&lib_path).expect("Failed to read src/lib.rs");
    let version = extract_version(&lib_content, "//! thirtyfour = ")
        .expect("thirtyfour version not found in src/lib.rs");

    assert_eq!(
        version, expected_version,
        "Thirtyfour version in src/lib.rs does not match docref/Cargo.toml"
    );
}

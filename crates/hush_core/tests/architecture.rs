//! Enforces strict dependency direction rules.
//! This test ensures that lower-level crates (like core, crypto, envelope)
//! never accidentally depend on higher-level crates (like cli, tui).

use std::process::Command;

/// Runs `cargo tree` for a specific package and returns the output as a string.
fn get_dependency_tree(package: &str) -> String {
    let output = Command::new("cargo")
        .args(["tree", "-p", package, "--prefix", "none"])
        .output()
        .expect("Failed to execute cargo tree. Is it in your PATH?");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Ensures the core business logic never depends on the user interfaces.
#[test]
fn test_core_does_not_depend_on_ui() {
    let tree = get_dependency_tree("hush_core");

    assert!(
        !tree.contains("hush_cli"),
        "Architecture violation: hush_core cannot depend on hush_cli! Core must remain UI-agnostic."
    );
    assert!(
        !tree.contains("hush_tui"),
        "Architecture violation: hush_core cannot depend on hush_tui! Core must remain UI-agnostic."
    );
}

/// Ensures foundational primitives never depend on higher-level orchestration.
#[test]
fn test_primitives_do_not_depend_on_core() {
    let crypto_tree = get_dependency_tree("hush_crypto");
    let envelope_tree = get_dependency_tree("hush_envelope");

    assert!(
        !crypto_tree.contains("hush_core"),
        "Architecture violation: hush_crypto cannot depend on hush_core."
    );
    assert!(
        !envelope_tree.contains("hush_core"),
        "Architecture violation: hush_envelope cannot depend on hush_core."
    );
}

/// Ensures the configuration crate remains a pure data layer.
#[test]
fn test_config_is_leaf_node() {
    let tree = get_dependency_tree("hush_config");

    // hush_config should only depend on external crates (like serde), not internal ones.
    assert!(
        !tree.contains("hush_core")
            && !tree.contains("hush_crypto")
            && !tree.contains("hush_envelope")
            && !tree.contains("hush_cli")
            && !tree.contains("hush_tui"),
        "Architecture violation: hush_config must not depend on any other internal hush crates."
    );
}

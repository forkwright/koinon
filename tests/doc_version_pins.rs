//! Documentation dependency-pin examples must track the released version.

use std::fs;
use std::path::Path;

// WHY(koinon#27): README.md and ADOPTION.md each carry copy-paste
// `koinon = { git = ..., tag = "vX.Y.Z" }` examples. Nothing else keeps
// those in sync with Cargo.toml's `version` — both examples stayed pinned
// to v0.1.0 through the v0.1.3 release with nothing to catch the drift.
fn assert_pins_match_current_release(relative_path: &str, expected_tag: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = Path::new(manifest_dir).join(relative_path);
    let content = match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => panic!("failed to read {full_path:?}: {e}"),
    };

    let pins: Vec<&str> = content
        .lines()
        .filter(|line| line.contains("forkwright/koinon") && line.contains("tag ="))
        .collect();

    assert!(
        !pins.is_empty(),
        "{relative_path}: expected at least one `forkwright/koinon` dependency \
         example with a `tag = ` pin"
    );

    for line in pins {
        assert!(
            line.contains(expected_tag),
            "{relative_path}: dependency example pinned to a stale tag \
             (want {expected_tag}): {line}"
        );
    }
}

#[test]
fn readme_dependency_examples_pin_current_release() {
    let expected_tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert_pins_match_current_release("README.md", &expected_tag);
}

#[test]
fn adoption_guide_dependency_examples_pin_current_release() {
    let expected_tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert_pins_match_current_release("ADOPTION.md", &expected_tag);
}

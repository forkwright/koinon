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
        Err(e) => panic!("failed to read {}: {e}", full_path.display()),
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

// WHY(koinon#27): the two tests above assert the pins are current, but nothing makes them
// current — release-please does, and only for files its config lists. Without the markers
// the assertions above are unsatisfiable at exactly one moment: the release PR, which bumps
// CARGO_PKG_VERSION while leaving the docs behind. That failure is not a caught defect, it
// is the release path blocked on itself, and a permanently-red release PR is what recruits
// people into building a bypass around the gate.
//
// So this asserts the mechanism, not the state. It fails on the change that removes the
// markers rather than on the next release that needed them.
fn assert_release_please_markers_bracket_every_pin(relative_path: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = std::path::Path::new(manifest_dir).join(relative_path);
    let content = match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => panic!("failed to read {}: {e}", full_path.display()),
    };

    let mut inside = false;
    let mut unmarked = Vec::new();
    for line in content.lines() {
        match line.trim() {
            "<!-- x-release-please-start-version -->" => inside = true,
            "<!-- x-release-please-end-version -->" => inside = false,
            _ => {
                if !inside && line.contains("forkwright/koinon") && line.contains("tag =") {
                    unmarked.push(line);
                }
            }
        }
    }

    assert!(
        unmarked.is_empty(),
        "{relative_path}: dependency pin(s) outside an x-release-please-version block, so \
         release-please will not bump them and the release PR will fail on the stale tag: \
         {unmarked:#?}"
    );
}

#[test]
fn readme_pins_are_release_please_managed() {
    assert_release_please_markers_bracket_every_pin("README.md");
}

#[test]
fn adoption_guide_pins_are_release_please_managed() {
    assert_release_please_markers_bracket_every_pin("ADOPTION.md");
}

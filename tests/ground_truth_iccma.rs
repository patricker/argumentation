//! Real ICCMA 2019 benchmark integration tests.
//!
//! Loads `.apx` fixtures from `tests/iccma_fixtures/real_2019/` and compares
//! the crate's extension outputs against externally-published expected
//! outputs from the ICCMA 2019 reference-results archive. See
//! `tests/iccma_fixtures/real_2019/PROVENANCE.md` for source URLs,
//! licensing, and a description of the selected instances.
//!
//! These tests exist to guard against silent regressions by comparing to
//! third-party ground truth that we did not compute ourselves.

use argumentation::parsers::parse_apx;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

/// Parse an expected-extensions file into a set of extensions (each a
/// `BTreeSet<String>`). Extensions are separated by lines of exactly `---`;
/// within an extension, each non-blank non-comment line is one argument
/// name. Lines starting with `#` are comments.
///
/// A file with zero `---` separators and no argument lines parses to a
/// single empty extension — matching the ICCMA SE-GR / SE-ID convention
/// for "the grounded/ideal extension is ∅".
fn parse_expected(path: &Path) -> BTreeSet<BTreeSet<String>> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let mut result = BTreeSet::new();
    let mut current: BTreeSet<String> = BTreeSet::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "---" {
            result.insert(std::mem::take(&mut current));
            continue;
        }
        current.insert(trimmed.to_string());
    }
    result.insert(current);
    result
}

fn to_canonical(exts: Vec<HashSet<String>>) -> BTreeSet<BTreeSet<String>> {
    exts.into_iter().map(|s| s.into_iter().collect()).collect()
}

/// Run one fixture through one semantic and assert expected match.
fn check_instance(stem: &str, semantic: &str) {
    let apx_path = format!("tests/iccma_fixtures/real_2019/{stem}.apx");
    let expected_path = format!("tests/iccma_fixtures/real_2019/expected/{stem}.{semantic}.txt");
    assert!(Path::new(&apx_path).exists(), "fixture missing: {apx_path}");
    assert!(
        Path::new(&expected_path).exists(),
        "expected file missing: {expected_path}"
    );
    let apx = fs::read_to_string(&apx_path).unwrap();
    let af = parse_apx(&apx).unwrap_or_else(|e| panic!("parse {apx_path}: {e}"));
    let actual: BTreeSet<BTreeSet<String>> = match semantic {
        "grounded" => {
            let g = af.grounded_extension();
            let mut singleton = BTreeSet::new();
            singleton.insert(g.into_iter().collect::<BTreeSet<_>>());
            singleton
        }
        "complete" => to_canonical(af.complete_extensions().unwrap()),
        "preferred" => to_canonical(af.preferred_extensions().unwrap()),
        "stable" => to_canonical(af.stable_extensions().unwrap()),
        "ideal" => {
            let i = af.ideal_extension().unwrap();
            let mut singleton = BTreeSet::new();
            singleton.insert(i.into_iter().collect::<BTreeSet<_>>());
            singleton
        }
        "semi-stable" => to_canonical(af.semi_stable_extensions().unwrap()),
        _ => panic!("unknown semantic {semantic}"),
    };
    let expected = parse_expected(Path::new(&expected_path));
    assert_eq!(
        actual, expected,
        "instance {stem} semantic {semantic}: actual vs expected"
    );
}

// ---------------------------------------------------------------------------
// Small-result-b2: 5 args, mutual attacks + isolated unattacked defender.
// ---------------------------------------------------------------------------

#[test]
fn small_result_b2_grounded() {
    check_instance("Small-result-b2", "grounded");
}
#[test]
fn small_result_b2_ideal() {
    check_instance("Small-result-b2", "ideal");
}
#[test]
fn small_result_b2_complete() {
    check_instance("Small-result-b2", "complete");
}
#[test]
fn small_result_b2_preferred() {
    check_instance("Small-result-b2", "preferred");
}
#[test]
fn small_result_b2_stable() {
    check_instance("Small-result-b2", "stable");
}
#[test]
fn small_result_b2_semi_stable() {
    check_instance("Small-result-b2", "semi-stable");
}

// ---------------------------------------------------------------------------
// Small-result-b41: 6 args, three unattacked sources + cycle among disputes.
// ---------------------------------------------------------------------------

#[test]
fn small_result_b41_grounded() {
    check_instance("Small-result-b41", "grounded");
}
#[test]
fn small_result_b41_ideal() {
    check_instance("Small-result-b41", "ideal");
}
#[test]
fn small_result_b41_complete() {
    check_instance("Small-result-b41", "complete");
}
#[test]
fn small_result_b41_preferred() {
    check_instance("Small-result-b41", "preferred");
}
#[test]
fn small_result_b41_stable() {
    check_instance("Small-result-b41", "stable");
}
#[test]
fn small_result_b41_semi_stable() {
    check_instance("Small-result-b41", "semi-stable");
}

// ---------------------------------------------------------------------------
// Small-result-b8: 8 args, central argument vs many peripherals (nested).
// ---------------------------------------------------------------------------

#[test]
fn small_result_b8_grounded() {
    check_instance("Small-result-b8", "grounded");
}
#[test]
fn small_result_b8_ideal() {
    check_instance("Small-result-b8", "ideal");
}
#[test]
fn small_result_b8_complete() {
    check_instance("Small-result-b8", "complete");
}
#[test]
fn small_result_b8_preferred() {
    check_instance("Small-result-b8", "preferred");
}
#[test]
fn small_result_b8_stable() {
    check_instance("Small-result-b8", "stable");
}
#[test]
fn small_result_b8_semi_stable() {
    check_instance("Small-result-b8", "semi-stable");
}

// ---------------------------------------------------------------------------
// Small-result-b57: 8 args, a1_0 ↔ many peripherals, dense cross-attacks.
// ---------------------------------------------------------------------------

#[test]
fn small_result_b57_grounded() {
    check_instance("Small-result-b57", "grounded");
}
#[test]
fn small_result_b57_ideal() {
    check_instance("Small-result-b57", "ideal");
}
#[test]
fn small_result_b57_complete() {
    check_instance("Small-result-b57", "complete");
}
#[test]
fn small_result_b57_preferred() {
    check_instance("Small-result-b57", "preferred");
}
#[test]
fn small_result_b57_stable() {
    check_instance("Small-result-b57", "stable");
}
#[test]
fn small_result_b57_semi_stable() {
    check_instance("Small-result-b57", "semi-stable");
}

// ---------------------------------------------------------------------------
// Small-result-b35: 7 args, 22 attacks — 6 complete extensions.
// ---------------------------------------------------------------------------

#[test]
fn small_result_b35_grounded() {
    check_instance("Small-result-b35", "grounded");
}
#[test]
fn small_result_b35_ideal() {
    check_instance("Small-result-b35", "ideal");
}
#[test]
fn small_result_b35_complete() {
    check_instance("Small-result-b35", "complete");
}
#[test]
fn small_result_b35_preferred() {
    check_instance("Small-result-b35", "preferred");
}
#[test]
fn small_result_b35_stable() {
    check_instance("Small-result-b35", "stable");
}
#[test]
fn small_result_b35_semi_stable() {
    check_instance("Small-result-b35", "semi-stable");
}

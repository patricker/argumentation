//! Integration tests against fixture instances in ICCMA APX format.

use argumentation::parsers::parse_apx;
use std::collections::HashSet;
use std::fs;

fn read_expected_set(path: &str) -> HashSet<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#') && *l != "---")
        .map(|l| l.trim().to_string())
        .collect()
}

fn read_expected_extensions(path: &str) -> Vec<HashSet<String>> {
    let contents = fs::read_to_string(path).unwrap();
    let mut result = Vec::new();
    let mut current: HashSet<String> = HashSet::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.insert(trimmed.to_string());
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

#[test]
fn figure_1_grounded_matches_fixture() {
    let apx = fs::read_to_string("tests/iccma_fixtures/figure_1.apx").unwrap();
    let af = parse_apx(&apx).unwrap();
    let grounded = af.grounded_extension();
    let expected = read_expected_set("tests/iccma_fixtures/figure_1_grounded.txt");
    assert_eq!(grounded, expected);
}

#[test]
fn mutual_attack_preferred_matches_fixture() {
    let apx = fs::read_to_string("tests/iccma_fixtures/mutual_attack.apx").unwrap();
    let af = parse_apx(&apx).unwrap();
    let preferred = af.preferred_extensions().unwrap();
    let expected = read_expected_extensions("tests/iccma_fixtures/mutual_attack_preferred.txt");
    assert_eq!(preferred.len(), expected.len());
    for e in &expected {
        assert!(preferred.contains(e));
    }
}

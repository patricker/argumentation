use argumentation_schemes::catalog::default_catalog;
use encounter_argumentation::value_argument::scheme_value_argument;

#[test]
fn higher_conviction_wins() {
    let registry = default_catalog();
    let result = scheme_value_argument("alice", "bob", "honor", 0.8, 0.4, 0.6, &registry);
    assert_eq!(result.winner, "alice");
    assert!(result.loser_value_shift > 0.0);
}

#[test]
fn defender_wins_when_more_convinced() {
    let registry = default_catalog();
    let result = scheme_value_argument("alice", "bob", "pragmatism", 0.3, 0.9, 0.5, &registry);
    assert_eq!(result.winner, "bob");
}

#[test]
fn equal_conviction_favors_attacker() {
    let registry = default_catalog();
    let result = scheme_value_argument("alice", "bob", "tradition", 0.5, 0.5, 0.8, &registry);
    assert_eq!(result.winner, "alice");
}

#[test]
fn returns_compatible_result_type() {
    let registry = default_catalog();
    let result = scheme_value_argument("alice", "bob", "security", 0.7, 0.3, 0.6, &registry);
    assert!(!result.value_at_stake.is_empty());
    assert!(result.loser_value_shift >= 0.0 && result.loser_value_shift <= 1.0);
    assert!(result.winner_value_shift >= 0.0 && result.winner_value_shift <= 0.1);
}

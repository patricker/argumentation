//! Multi-value defeat semantics (Kaci & van der Torre 2008).
//!
//! Pareto rule: A defeats B iff for every value v_b in val(B), some
//! value v_a in val(A) is not strictly less-preferred than v_b under
//! the audience.

use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

fn binary_attack(
    a_values: &[&str],
    b_values: &[&str],
) -> ValueBasedFramework<&'static str> {
    let mut base = ArgumentationFramework::new();
    base.add_argument("a");
    base.add_argument("b");
    base.add_attack(&"a", &"b").unwrap();
    let mut values = ValueAssignment::new();
    for v in a_values {
        values.promote("a", Value::new(*v));
    }
    for v in b_values {
        values.promote("b", Value::new(*v));
    }
    ValueBasedFramework::new(base, values)
}

#[test]
fn pareto_defeat_when_attacker_dominates() {
    // a promotes {life, autonomy}, b promotes {property}
    // Under audience [life > property], life ≥ property (life higher).
    // For target value property: attacker has life (preferred over property).
    // → a defeats b.
    let vaf = binary_attack(&["life", "autonomy"], &["property"]);
    let aud = Audience::total([Value::new("life"), Value::new("property")]);
    assert!(vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn pareto_defeat_blocked_when_target_strictly_dominates_every_attacker_value() {
    // a promotes {property}, b promotes {life, autonomy}
    // Under audience [life > autonomy > property]:
    //   target value life: attacker has only property; life is preferred over property
    //                      → no attacker value satisfies the rule for target value life.
    // → a does NOT defeat b.
    let vaf = binary_attack(&["property"], &["life", "autonomy"]);
    let aud = Audience::total([
        Value::new("life"),
        Value::new("autonomy"),
        Value::new("property"),
    ]);
    assert!(!vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn pareto_defeat_one_target_value_can_save_the_target() {
    // a promotes {fairness}, b promotes {fairness, life}
    // Under audience [life > fairness]:
    //   target value fairness: attacker has fairness, equal rank → not strictly preferred,
    //                          attacker side wins → satisfies rule.
    //   target value life: attacker has fairness; life IS preferred over fairness
    //                      → attacker has no value not-less-preferred than life
    //                      → fails the rule.
    // → a does NOT defeat b (one target value B has that A can't match).
    let vaf = binary_attack(&["fairness"], &["fairness", "life"]);
    let aud = Audience::total([Value::new("life"), Value::new("fairness")]);
    assert!(!vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn pareto_defeat_reduces_to_benchcapon_for_single_values() {
    // Single-value sanity check: attacker promotes {life}, target promotes {property}.
    // Under [life > property]: a defeats b. Under [property > life]: a does NOT defeat b.
    let vaf = binary_attack(&["life"], &["property"]);
    let life_audience = Audience::total([Value::new("life"), Value::new("property")]);
    let property_audience = Audience::total([Value::new("property"), Value::new("life")]);
    assert!(vaf.defeats(&"a", &"b", &life_audience));
    assert!(!vaf.defeats(&"a", &"b", &property_audience));
}

#[test]
fn unranked_target_value_does_not_save_target() {
    // a promotes {life}, b promotes {fairness} (fairness not in audience).
    // Under [life]: fairness is unranked → audience.prefers(fairness, life) = false
    //               → for target value fairness, attacker value life satisfies rule
    //               → a defeats b.
    let vaf = binary_attack(&["life"], &["fairness"]);
    let aud = Audience::total([Value::new("life")]);
    assert!(vaf.defeats(&"a", &"b", &aud));
}

#[test]
fn unranked_attacker_value_can_still_defeat_unranked_target() {
    // Both unranked → audience.prefers returns false → defeats by null-tie rule.
    let vaf = binary_attack(&["honor"], &["tradition"]);
    let aud = Audience::total([Value::new("life")]);
    assert!(vaf.defeats(&"a", &"b", &aud));
}

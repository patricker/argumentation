//! APX text format I/O for VAFs (ASPARTIX-compatible).
//!
//! The APX format uses Prolog-style facts:
//!
//! ```text
//! arg(h1).
//! arg(c1).
//! att(h1, c1).
//! att(c1, h1).
//! val(h1, life).
//! val(c1, property).
//! valpref(life, property).
//! ```
//!
//! Comments start with `%` and run to end of line. Whitespace is ignored.
//!
//! `valpref(a, b)` means value `a` is strictly preferred over value `b`.
//! Multiple `valpref` facts together encode an audience.

use crate::error::Error;
use crate::framework::ValueBasedFramework;
use crate::types::{Audience, Value, ValueAssignment};
use argumentation::ArgumentationFramework;

/// Parse an APX document into (framework, audience) pair.
///
/// The resulting framework owns string-typed argument labels (matching
/// the `arg(name)` identifier in the input). The audience is derived
/// from the `valpref` facts; if no `valpref` facts are present, the
/// audience is empty.
pub fn from_apx(input: &str) -> Result<(ValueBasedFramework<String>, Audience), Error> {
    let mut base = ArgumentationFramework::new();
    let mut values = ValueAssignment::new();
    let mut prefs: Vec<(String, String)> = Vec::new();

    for (line_idx, raw_line) in input.lines().enumerate() {
        let line_no = line_idx + 1;
        // Strip comments
        let line = raw_line.split('%').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        // Each fact ends with `.`
        let fact = line.strip_suffix('.').ok_or_else(|| Error::ApxParse {
            line: line_no,
            reason: format!("expected fact ending with '.', got: {line}"),
        })?;
        // Predicate(args) form
        let (pred, args) = parse_pred(fact, line_no)?;
        match pred {
            "arg" => {
                let [name] = expect_n_args::<1>(&args, line_no, "arg")?;
                base.add_argument(name.to_string());
            }
            "att" => {
                let [attacker, target] = expect_n_args::<2>(&args, line_no, "att")?;
                let attacker_s = attacker.to_string();
                let target_s = target.to_string();
                base.add_attack(&attacker_s, &target_s)
                    .map_err(Error::from)?;
            }
            "val" => {
                let [arg, value] = expect_n_args::<2>(&args, line_no, "val")?;
                values.promote(arg.to_string(), Value::new(value.to_string()));
            }
            "valpref" => {
                let [a, b] = expect_n_args::<2>(&args, line_no, "valpref")?;
                prefs.push((a.to_string(), b.to_string()));
            }
            other => {
                return Err(Error::ApxParse {
                    line: line_no,
                    reason: format!("unknown predicate: {other}"),
                });
            }
        }
    }

    let audience = audience_from_prefs(&prefs);
    Ok((ValueBasedFramework::new(base, values), audience))
}

/// Serialise a VAF + audience to APX format.
pub fn to_apx(vaf: &ValueBasedFramework<String>, audience: &Audience) -> String {
    let mut out = String::new();
    let mut args: Vec<&String> = vaf.base().arguments().collect();
    args.sort();
    for arg in &args {
        out.push_str(&format!("arg({arg}).\n"));
    }
    for target in &args {
        let mut attackers: Vec<&String> = vaf.base().attackers(*target).into_iter().collect();
        attackers.sort();
        for atk in attackers {
            out.push_str(&format!("att({atk}, {target}).\n"));
        }
    }
    let mut entries: Vec<(&String, &[Value])> = vaf
        .value_assignment()
        .entries()
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    for (arg, vals) in entries {
        for v in vals {
            out.push_str(&format!("val({arg}, {v}).\n"));
        }
    }
    // valpref: emit one fact per strict preference. The pairwise scan is
    // O(n²) on |values| but n is bounded by ENUMERATION_LIMIT * a small
    // constant in practice. ASPARTIX accepts redundant valpref facts
    // (it computes the transitive closure on import), so emitting all
    // strict pairs (not just the transitive reduction) is fine.
    let all_values: Vec<&Value> = audience.values().collect();
    let mut emitted = std::collections::BTreeSet::new();
    for a in &all_values {
        for b in &all_values {
            if audience.prefers(a, b) && emitted.insert((a.as_str(), b.as_str())) {
                out.push_str(&format!("valpref({a}, {b}).\n"));
            }
        }
    }
    out
}

fn parse_pred(fact: &str, line: usize) -> Result<(&str, Vec<&str>), Error> {
    let open = fact.find('(').ok_or_else(|| Error::ApxParse {
        line,
        reason: format!("expected '(' in fact: {fact}"),
    })?;
    let close = fact.rfind(')').ok_or_else(|| Error::ApxParse {
        line,
        reason: format!("expected ')' in fact: {fact}"),
    })?;
    if close <= open {
        return Err(Error::ApxParse {
            line,
            reason: format!("malformed fact: {fact}"),
        });
    }
    let pred = &fact[..open];
    let args_str = &fact[open + 1..close];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    Ok((pred, args))
}

fn expect_n_args<const N: usize>(
    args: &[&str],
    line: usize,
    pred: &str,
) -> Result<[String; N], Error> {
    if args.len() != N {
        return Err(Error::ApxParse {
            line,
            reason: format!(
                "{pred} expects {N} arg(s), got {got}",
                got = args.len()
            ),
        });
    }
    let mut out: [String; N] = std::array::from_fn(|_| String::new());
    for (i, a) in args.iter().enumerate() {
        out[i] = (*a).to_string();
    }
    Ok(out)
}

/// Build an Audience from a set of pairwise (strict) preferences.
///
/// Strategy: topologically sort the directed preference graph (a → b if
/// `valpref(a, b)`). Each topological level becomes one tier.
fn audience_from_prefs(prefs: &[(String, String)]) -> Audience {
    use std::collections::{BTreeSet, HashMap, HashSet};

    if prefs.is_empty() {
        return Audience::new();
    }

    // Collect all distinct values.
    let mut all: BTreeSet<String> = BTreeSet::new();
    for (a, b) in prefs {
        all.insert(a.clone());
        all.insert(b.clone());
    }

    // Build successor map (a → set of b's that a outranks).
    let mut outranks: HashMap<String, HashSet<String>> = HashMap::new();
    let mut indegree: HashMap<String, usize> = HashMap::new();
    for v in &all {
        outranks.entry(v.clone()).or_default();
        indegree.entry(v.clone()).or_insert(0);
    }
    for (a, b) in prefs {
        if outranks.get_mut(a).unwrap().insert(b.clone()) {
            *indegree.get_mut(b).unwrap() += 1;
        }
    }

    let mut tiers: Vec<Vec<Value>> = Vec::new();
    let mut remaining: HashSet<String> = all.into_iter().collect();
    while !remaining.is_empty() {
        let mut current_tier: Vec<String> = remaining
            .iter()
            .filter(|v| *indegree.get(*v).unwrap() == 0)
            .cloned()
            .collect();
        if current_tier.is_empty() {
            // Cycle — emit remaining as one indistinguishable tier.
            current_tier = remaining.iter().cloned().collect();
        }
        current_tier.sort();
        for v in &current_tier {
            remaining.remove(v);
            // Decrement indegrees for successors.
            if let Some(succs) = outranks.get(v) {
                for s in succs {
                    if let Some(d) = indegree.get_mut(s)
                        && *d > 0
                    {
                        *d -= 1;
                    }
                }
            }
        }
        tiers.push(current_tier.into_iter().map(Value::new).collect());
    }

    Audience::from_tiers(tiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_small_vaf() {
        let input = r#"
% Hal & Carla in APX
arg(h1).
arg(c1).
att(h1, c1).
att(c1, h1).
val(h1, life).
val(c1, property).
valpref(life, property).
"#;
        let (vaf, audience) = from_apx(input).unwrap();
        assert_eq!(vaf.base().len(), 2);
        assert!(audience.prefers(&Value::new("life"), &Value::new("property")));
        // Round-trip: serialise then re-parse.
        let serialised = to_apx(&vaf, &audience);
        let (vaf2, audience2) = from_apx(&serialised).unwrap();
        assert_eq!(vaf2.base().len(), 2);
        assert!(audience2.prefers(&Value::new("life"), &Value::new("property")));
    }

    #[test]
    fn parse_error_reports_line_and_predicate() {
        let input = "arg(a).\nbogus(stuff).\n";
        let err = from_apx(input).unwrap_err();
        match err {
            Error::ApxParse { line, reason } => {
                assert_eq!(line, 2);
                assert!(reason.contains("unknown predicate"));
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    fn comments_and_whitespace_ignored() {
        let input = r#"
% header comment
arg(a).   % trailing comment
arg(b).

att(a, b).
"#;
        let (vaf, _) = from_apx(input).unwrap();
        assert_eq!(vaf.base().len(), 2);
    }

    #[test]
    fn empty_input_yields_empty_vaf() {
        let (vaf, audience) = from_apx("").unwrap();
        assert_eq!(vaf.base().len(), 0);
        assert_eq!(audience.value_count(), 0);
    }

    #[test]
    fn multi_tier_audience_emerges_from_chained_prefs() {
        let input = r#"
arg(a).
arg(b).
arg(c).
val(a, life).
val(b, fairness).
val(c, property).
valpref(life, fairness).
valpref(fairness, property).
"#;
        let (_vaf, audience) = from_apx(input).unwrap();
        assert_eq!(audience.tier_count(), 3);
        assert!(audience.prefers(&Value::new("life"), &Value::new("fairness")));
        assert!(audience.prefers(&Value::new("fairness"), &Value::new("property")));
        assert!(audience.prefers(&Value::new("life"), &Value::new("property")));
    }
}

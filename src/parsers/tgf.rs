//! Parser for Trivial Graph Format (TGF).
//!
//! TGF splits nodes and edges with a `#` line:
//!   a
//!   b
//!   c
//!   #
//!   a b
//!   b c

use crate::Error;
use crate::framework::ArgumentationFramework;

/// Parse a TGF document into an argumentation framework with `String` arguments.
pub fn parse_tgf(input: &str) -> Result<ArgumentationFramework<String>, Error> {
    let mut af = ArgumentationFramework::new();
    let mut in_edges = false;
    for (lineno, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "#" {
            in_edges = true;
            continue;
        }
        if !in_edges {
            let id = line.split_whitespace().next().unwrap().to_string();
            af.add_argument(id);
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(Error::Parse(format!(
                    "line {}: edge needs two ids",
                    lineno + 1
                )));
            }
            af.add_attack(&parts[0].to_string(), &parts[1].to_string())?;
        }
    }
    Ok(af)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_tgf() {
        let input = "a\nb\nc\n#\na b\nb c\n";
        let af = parse_tgf(input).unwrap();
        assert_eq!(af.arguments().count(), 3);
        assert_eq!(af.attackers(&"b".to_string()).len(), 1);
        assert_eq!(af.attackers(&"c".to_string()).len(), 1);
    }
}

//! Parser for ICCMA APX format.
//!
//! APX is a simple text format:
//!   arg(a).
//!   arg(b).
//!   att(a, b).
//!
//! Comments start with `%`. Whitespace is flexible.

use crate::Error;
use crate::framework::ArgumentationFramework;

/// Parse an APX document into an argumentation framework with `String` arguments.
pub fn parse_apx(input: &str) -> Result<ArgumentationFramework<String>, Error> {
    let mut af = ArgumentationFramework::new();
    for (lineno, raw_line) in input.lines().enumerate() {
        let line = raw_line.split('%').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("arg(") {
            let arg = rest
                .trim_end_matches(['.', ' '])
                .trim_end_matches(')')
                .trim()
                .to_string();
            if arg.is_empty() {
                return Err(Error::Parse(format!("line {}: empty arg", lineno + 1)));
            }
            af.add_argument(arg);
        } else if let Some(rest) = line.strip_prefix("att(") {
            let inner = rest
                .trim_end_matches(['.', ' '])
                .trim_end_matches(')')
                .trim();
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            if parts.len() != 2 {
                return Err(Error::Parse(format!(
                    "line {}: att expects 2 args, got {}",
                    lineno + 1,
                    parts.len()
                )));
            }
            af.add_attack(&parts[0].to_string(), &parts[1].to_string())?;
        } else {
            return Err(Error::Parse(format!(
                "line {}: unrecognised: {}",
                lineno + 1,
                line
            )));
        }
    }
    Ok(af)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_apx() {
        let input = "arg(a).\narg(b).\natt(a,b).\n";
        let af = parse_apx(input).unwrap();
        assert_eq!(af.arguments().count(), 2);
        assert_eq!(af.attackers(&"b".to_string()).len(), 1);
    }

    #[test]
    fn parse_apx_with_comments() {
        let input = "% test\narg(x).\narg(y).\n% comment\natt(x, y).\n";
        let af = parse_apx(input).unwrap();
        assert_eq!(af.arguments().count(), 2);
    }

    #[test]
    fn parse_apx_rejects_unknown_syntax() {
        let input = "foo(a).\n";
        assert!(parse_apx(input).is_err());
    }
}

//! Parser for ICCMA APX format.
//!
//! APX is a simple text format:
//!   arg(a).
//!   arg(b).
//!   att(a, b).
//!
//! Comments start with `%`. Whitespace is flexible but each statement must
//! match the shape `arg(NAME).` or `att(NAME, NAME).` exactly — mixed
//! whitespace or stray characters between the closing paren and the period
//! produce a parse error rather than silently mangling the argument name.

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
            let body = extract_paren_body(rest, lineno)?;
            if body.is_empty() {
                return Err(Error::Parse(format!("line {}: empty arg", lineno + 1)));
            }
            af.add_argument(body.to_string());
        } else if let Some(rest) = line.strip_prefix("att(") {
            let body = extract_paren_body(rest, lineno)?;
            let parts: Vec<&str> = body.split(',').map(str::trim).collect();
            if parts.len() != 2 {
                return Err(Error::Parse(format!(
                    "line {}: att expects 2 args, got {}",
                    lineno + 1,
                    parts.len()
                )));
            }
            af.add_attack(&parts[0].to_string(), &parts[1].to_string())
                .map_err(|e| Error::Parse(format!("line {}: {}", lineno + 1, e)))?;
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

/// Extract the body between `(` (already consumed) and the final `)`,
/// requiring only whitespace and an optional `.` terminator after the `)`.
/// Returns the trimmed body or `Error::Parse` on malformed input.
fn extract_paren_body(rest: &str, lineno: usize) -> Result<&str, Error> {
    let Some(close_idx) = rest.rfind(')') else {
        return Err(Error::Parse(format!(
            "line {}: missing closing `)`",
            lineno + 1
        )));
    };
    let (body, tail) = rest.split_at(close_idx);
    let tail = &tail[1..]; // skip the `)`
    if !tail.is_empty() && tail != "." {
        return Err(Error::Parse(format!(
            "line {}: unexpected text after `)`: {:?}",
            lineno + 1,
            tail
        )));
    }
    Ok(body.trim())
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

    #[test]
    fn parse_apx_rejects_tab_between_paren_and_period() {
        // Previously: silently accepted as argument name "a)".
        let input = "arg(a)\t.\n";
        let err = parse_apx(input).unwrap_err();
        assert!(
            matches!(err, Error::Parse(_)),
            "expected Parse error, got {:?}",
            err
        );
    }

    #[test]
    fn parse_apx_rejects_trailing_garbage_after_close() {
        let input = "arg(a)extra.\n";
        assert!(parse_apx(input).is_err());
    }

    #[test]
    fn parse_apx_rejects_missing_close_paren() {
        let input = "arg(a.\n";
        assert!(parse_apx(input).is_err());
    }

    #[test]
    fn parse_apx_accepts_extra_whitespace_inside() {
        // `arg( a ).` and `att( a , b ).` should still be accepted.
        let input = "arg( a ).\narg( b ).\natt( a , b ).\n";
        let af = parse_apx(input).unwrap();
        assert_eq!(af.arguments().count(), 2);
        assert_eq!(af.attackers(&"b".to_string()).len(), 1);
    }
}

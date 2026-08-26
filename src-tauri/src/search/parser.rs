#[derive(Debug, PartialEq, Eq)]
pub enum QueryMode {
    Empty,
    Calculator(String),
    Command(String),
    Web(String),
    General(String),
}

pub struct QueryParser;

impl QueryParser {
    pub fn parse(raw: &str) -> QueryMode {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return QueryMode::Empty;
        }

        // Explicit calculator prefix '='
        if let Some(expr) = trimmed.strip_prefix('=') {
            return QueryMode::Calculator(expr.trim().to_string());
        }

        // Explicit command prefix '>'
        if let Some(cmd) = trimmed.strip_prefix('>') {
            return QueryMode::Command(cmd.trim().to_string());
        }

        // URL pattern recognition
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || (trimmed.contains('.') && !trimmed.contains(' ') && trimmed.len() > 4)
        {
            return QueryMode::Web(trimmed.to_string());
        }

        // Math expression pattern (e.g. 2 + 2, sqrt(144))
        if Self::looks_like_math(trimmed) {
            return QueryMode::Calculator(trimmed.to_string());
        }

        QueryMode::General(trimmed.to_string())
    }

    fn looks_like_math(s: &str) -> bool {
        let has_operator = s.contains('+')
            || s.contains('-')
            || s.contains('*')
            || s.contains('/')
            || s.contains('^')
            || s.contains('%')
            || s.starts_with("sqrt(")
            || s.starts_with("abs(");

        let valid_chars = s.chars().all(|c| {
            c.is_ascii_digit() || c.is_whitespace() || "+-*/^%().,".contains(c) || c.is_alphabetic()
        });

        has_operator && valid_chars && !s.chars().all(|c| c.is_alphabetic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert_eq!(QueryParser::parse(""), QueryMode::Empty);
        assert_eq!(QueryParser::parse("   "), QueryMode::Empty);
    }

    #[test]
    fn test_parse_explicit_calculator() {
        assert_eq!(
            QueryParser::parse("= 25 + 5"),
            QueryMode::Calculator("25 + 5".into())
        );
    }

    #[test]
    fn test_parse_implicit_math() {
        assert_eq!(
            QueryParser::parse("100 * 4"),
            QueryMode::Calculator("100 * 4".into())
        );
    }

    #[test]
    fn test_parse_command() {
        assert_eq!(
            QueryParser::parse("> lock"),
            QueryMode::Command("lock".into())
        );
    }

    #[test]
    fn test_parse_general() {
        assert_eq!(
            QueryParser::parse("chrome"),
            QueryMode::General("chrome".into())
        );
    }
}

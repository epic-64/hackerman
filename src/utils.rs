pub trait ToDuration {
    /// Convert a number to a [`std::time::Duration`].
    fn milliseconds(&self) -> std::time::Duration;
}

impl ToDuration for u64 {
    /// Convert a number to a [`std::time::Duration`].
    fn milliseconds(&self) -> std::time::Duration {
        std::time::Duration::from_millis(*self)
    }
}

pub trait TrimMargin {
    #![allow(dead_code)]
    fn nice(&self) -> String;
}

impl TrimMargin for str {
    /// Remove the surrounding whitespace from a multi-line string.
    /// Opinionated: it removes the first and last line, and trims the leading whitespace based on minimum indentation.
    /// (empty lines are ignored)
    fn nice(&self) -> String {
        let lines: Vec<&str> = self.lines().collect();

        let content_lines = &lines[1..lines.len().saturating_sub(1)];

        let indent = content_lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|&c| c == ' ').count())
            .min()
            .unwrap_or(0);

        let trimmed = |line: &&str| {
            if line.len() >= indent {
                line.chars().skip(indent).collect::<String>()
            } else {
                (*line).to_string()
            }
        };

        content_lines.iter().map(trimmed).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annoying_default_behavior() {
        let input = "
            asdf
        ";

        let expected = "\n            asdf\n        ";

        assert_eq!(expected, input);
    }

    #[test]
    fn test_nice_basic() {
        let input = "
            This is a test string.
            It has multiple lines.
            Some lines are indented.
        ";

        let expected = vec![
            "This is a test string.",
            "It has multiple lines.",
            "Some lines are indented."
        ].join("\n");

        let result = input.nice();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_nice_indentation() {
        let input = "
            This is a test string.
              It has multiple lines.
                Some lines are indented.
        ";

        let expected = vec![
            "This is a test string.",
            "  It has multiple lines.",
            "    Some lines are indented."
        ].join("\n");

        let result = input.nice();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_preserve_empty_lines() {
        let input = "
            This is a test string.

            It has multiple lines.

            Some lines are indented.
        ";

        let expected = vec![
            "This is a test string.",
            "",
            "It has multiple lines.",
            "",
            "Some lines are indented."
        ].join("\n");

        let result = input.nice();

        assert_eq!(expected, result);
    }
}
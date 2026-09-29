#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    /// Preserve CRLF only when every ASCII newline in the input uses CRLF.
    /// Inputs without newlines or with mixed line endings retain formatter output.
    fn detect_consistent(content: &str) -> Self {
        let bytes = content.as_bytes();
        let mut saw_crlf = false;
        let mut index = 0;

        while index < bytes.len() {
            match bytes[index] {
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                    saw_crlf = true;
                    index += 2;
                }
                b'\r' | b'\n' => return Self::Lf,
                _ => index += 1,
            }
        }

        if saw_crlf { Self::CrLf } else { Self::Lf }
    }

    fn apply(self, content: String) -> String {
        match self {
            Self::Lf => content,
            Self::CrLf => {
                let mut converted = String::with_capacity(content.len());
                push_crlf(&mut converted, &content);
                converted
            }
        }
    }
}

pub(super) fn apply_crlf_preserve(original: &str, formatted: String) -> String {
    LineEnding::detect_consistent(original).apply(formatted)
}

fn push_crlf(output: &mut String, content: &str) {
    let mut previous_was_cr = false;
    for ch in content.chars() {
        if ch == '\n' && !previous_was_cr {
            output.push('\r');
        }
        output.push(ch);
        previous_was_cr = ch == '\r';
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_only_consistent_crlf() {
        assert_eq!(
            LineEnding::detect_consistent("a\r\nb\r\n"),
            LineEnding::CrLf
        );
        assert_eq!(LineEnding::detect_consistent("a\nb\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\rb\r"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\r\nb\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\nb\r\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\r\nb\r"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("no newline"), LineEnding::Lf);

        assert_eq!(
            LineEnding::detect_consistent("a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}\r\n"),
            LineEnding::CrLf
        );
    }

    #[test]
    fn crlf_conversion_only_rewrites_bare_lf() {
        let content = "a\nb\r\nc\rd\u{000b}e\u{000c}f\u{0085}g\u{2028}h\u{2029}i";
        let converted = apply_crlf_preserve("a\r\nb\r\n", content.to_owned());

        assert_eq!(
            converted,
            "a\r\nb\r\nc\rd\u{000b}e\u{000c}f\u{0085}g\u{2028}h\u{2029}i"
        );
        assert_eq!(apply_crlf_preserve("a\nb\n", content.to_owned()), content);
    }

    #[test]
    fn crlf_preserve_is_idempotent_across_detection_branches() {
        let cases = [
            ("a\r\nb\r\n", "a\nb\n"),
            ("a\nb\n", "a\nb\n"),
            ("a\r\nb\n", "a\nb\n"),
            ("a\r\nb\r", "a\nb\r"),
            ("no newline", "no newline\n"),
        ];

        for &(original, formatted) in &cases {
            let once = apply_crlf_preserve(original, formatted.to_owned());
            let twice = apply_crlf_preserve(&once, once.clone());

            assert_eq!(once, twice);
        }
    }
}

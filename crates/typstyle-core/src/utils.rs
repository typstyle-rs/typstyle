/// Strip trailing whitespace in each line of the input string.
pub fn strip_trailing_whitespace(s: &str) -> String {
    if s.is_empty() {
        return "\n".to_string();
    }
    let mut res = String::with_capacity(s.len());
    for line in s.lines() {
        res.push_str(line.trim_end());
        res.push('\n');
    }
    res
}

pub fn count_spaces_after_last_newline(s: &str, i: usize) -> usize {
    // Ensure the byte position `i` is a valid UTF-8 boundary
    debug_assert!(
        s.is_char_boundary(i),
        "Position i is not a valid UTF-8 boundary"
    );

    // Find the last newline (`\n`) before position `i`
    if let Some(pos) = s[..i].rfind('\n') {
        // Get the substring after the newline and up to position `i`
        let after_newline = &s[pos + 1..i];
        // Count the number of consecutive spaces in the substring
        after_newline.chars().take_while(|&c| c == ' ').count()
    } else {
        // If no newline is found, return 0
        0
    }
}

/// Changes the indentation of a formatted string from one size to another.
///
/// This function converts space-only indentation from one size to another while
/// preserving the relative indentation structure. Assumes input uses only spaces
/// and indentation is always a multiple of the given indent size.
///
/// # Arguments
/// - `text`: The input text with existing space indentation
/// - `from_indent`: The current indentation size (e.g., 4 for 4 spaces)
/// - `to_indent`: The desired indentation size (e.g., 2 for 2 spaces)
///
/// # Examples
/// ```
/// use typstyle_core::utils::change_indent;
///
/// let input = "    line1\n        line2\n    line3";
/// let output = change_indent(input, 4, 2);
/// assert_eq!(output, "  line1\n    line2\n  line3");
/// ```
pub fn change_indent(text: &str, from_indent: usize, to_indent: usize) -> String {
    if text.is_empty() || from_indent == 0 || from_indent == to_indent {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut first = true;

    for line in text.lines() {
        if !first {
            result.push('\n');
        }
        first = false;

        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            // Keep blank lines empty
            continue;
        } else {
            let leading_spaces = line.len() - trimmed.len();
            let indent_level = leading_spaces / from_indent;
            let new_indent_size = to_indent * indent_level;

            // Build new line with correct indentation
            for _ in 0..new_indent_size {
                result.push(' ');
            }
            result.push_str(trimmed);
        }
    }

    result
}

/// Convenience function to change space indentation from 4 to 2 spaces
pub fn indent_4_to_2(text: &str) -> String {
    change_indent(text, 4, 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_trailing_whitespace() {
        let s = strip_trailing_whitespace("");
        assert_eq!(s, "\n");
        let s = strip_trailing_whitespace(" ");
        assert_eq!(s, "\n");
        let s = strip_trailing_whitespace("\n");
        assert_eq!(s, "\n");
        let s = strip_trailing_whitespace(" \n - \n");
        assert_eq!(s, "\n -\n");
        let s = strip_trailing_whitespace(" \n - \n ");
        assert_eq!(s, "\n -\n\n");
    }

    #[test]
    fn test_change_indent_basic() {
        let input = "    line1\n        line2\n    line3";
        let output = change_indent(input, 4, 2);
        assert_eq!(output, "  line1\n    line2\n  line3");
    }

    #[test]
    fn test_change_indent_empty() {
        assert_eq!(change_indent("", 4, 2), "");
        assert_eq!(change_indent("   ", 4, 2), "");
    }
}

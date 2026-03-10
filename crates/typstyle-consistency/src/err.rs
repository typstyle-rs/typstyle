use anyhow::{Result, anyhow};

#[derive(Default)]
pub struct ErrorSink {
    description: String,
    errors: Vec<String>,
}

impl ErrorSink {
    pub fn new(description: String) -> Self {
        Self {
            description,
            errors: Default::default(),
        }
    }

    pub fn push(&mut self, err: impl Into<String>) {
        self.errors.push(err.into());
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn sink_to(&self, parent: &mut Self) {
        if !self.errors.is_empty() {
            parent.push(format!("{self}"));
        }
    }
}

impl From<ErrorSink> for Result<()> {
    fn from(value: ErrorSink) -> Self {
        if value.is_ok() {
            Ok(())
        } else {
            Err(anyhow!("{value}"))
        }
    }
}

impl From<&ErrorSink> for Result<()> {
    fn from(value: &ErrorSink) -> Self {
        if value.errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow!("{value}"))
        }
    }
}

impl std::fmt::Display for ErrorSink {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "errors occurred in {}:", self.description)?;
        for (i, e) in self.errors.iter().enumerate() {
            let err_str = e.replace('\n', "\n    ");
            writeln!(f, "{:4}: {err_str}", i + 1)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! sink_assert_eq {
    ($sink: ident, $left: expr, $right: expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $sink.push(::std::format!(
                        "assertion failed: `(left == right)`\
                        \n\
                        \n{}\
                        \n",
                        similar_asserts::SimpleDiff::from_str(
                            &format!("{:?}", left_val),
                            &format!("{:?}", right_val),
                            "left",
                            "right"
                        ),
                    ))
                }
            }
        }
    };
    ($sink: ident, $left: expr, $right: expr, $($arg:tt)+) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $sink.push(::std::format!(
                        "assertion failed: `(left == right)`: {}\
                        \n\
                        \n{}\
                        \n",
                        ::std::format_args!($($arg)*),
                        similar_asserts::SimpleDiff::from_str(
                            &format!("{:?}", left_val),
                            &format!("{:?}", right_val),
                            "left",
                            "right"
                        ),
                    ))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! sink_assert_str_eq {
    ($sink: ident, $left: expr, $right: expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $sink.push(::std::format!(
                        "assertion failed: `(left == right)`\
                        \n\
                        \n{}\
                        \n",
                        similar_asserts::SimpleDiff::from_str(left_val, right_val, "left", "right"),
                    ))
                }
            }
        }
    };
    ($sink: ident, $left: expr, $right: expr, $($arg:tt)+) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $sink.push(::std::format!(
                        "assertion failed: `(left == right)`: {}\
                        \n\
                        \n{}\
                        \n",
                        ::std::format_args!($($arg)*),
                        similar_asserts::SimpleDiff::from_str(left_val, right_val, "left", "right"),
                    ))
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_empty_sink() {
        let sink = ErrorSink::new("empty description".to_string());
        let output = format!("{}", sink);
        assert!(output.ends_with('\n'));
        assert!(!output.ends_with("\n\n"));
        insta::assert_snapshot!(output, @"errors occurred in empty description:");
    }

    #[test]
    fn test_nested_sinks() {
        let mut sub_sink = ErrorSink::new("sub task".to_string());
        sub_sink.push("sub error 1");
        sub_sink.push("sub error 2\nwith newline");

        let mut main_sink = ErrorSink::new("main task".to_string());
        main_sink.push("main error");
        sub_sink.sink_to(&mut main_sink);

        let output = format!("{}", main_sink);
        assert!(output.ends_with('\n'));
        assert!(!output.ends_with("\n\n"));
        insta::assert_snapshot!(output, @"
        errors occurred in main task:
           1: main error
           2: errors occurred in sub task:
               1: sub error 1
               2: sub error 2
                with newline
        ");
    }
}

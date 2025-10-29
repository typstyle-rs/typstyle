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

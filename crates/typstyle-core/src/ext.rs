//! Extension traits for primitive types used across the crate.

/// Extension methods for [`bool`].
pub trait BoolExt {
    /// Replaces the value with the given one, returning the previous value.
    fn replace(&mut self, value: Self) -> Self;
}

impl BoolExt for bool {
    /// Replaces the value in place, returning the old one.
    fn replace(&mut self, value: Self) -> Self {
        let old = *self;
        *self = value;
        old
    }
}

/// Extension methods for string slices, used for layout decisions.
pub(crate) trait StrExt {
    /// Returns whether the text contains a linebreak.
    fn has_linebreak(&self) -> bool;

    /// Counts the number of linebreaks in the text.
    fn count_linebreaks(&self) -> usize;
}

impl StrExt for str {
    /// Returns whether the string contains a linebreak.
    fn has_linebreak(&self) -> bool {
        self.contains('\n')
    }

    /// Counts the linebreaks in the string.
    fn count_linebreaks(&self) -> usize {
        self.chars().filter(|c| *c == '\n').count()
    }
}

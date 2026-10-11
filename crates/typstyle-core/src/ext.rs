pub trait BoolExt {
    fn replace(&mut self, value: Self) -> Self;
}

impl BoolExt for bool {
    fn replace(&mut self, value: Self) -> Self {
        let old = *self;
        *self = value;
        old
    }
}

pub(crate) trait StrExt {
    fn has_linebreak(&self) -> bool;

    fn count_linebreaks(&self) -> usize;
}

impl StrExt for str {
    fn has_linebreak(&self) -> bool {
        self.contains('\n')
    }

    fn count_linebreaks(&self) -> usize {
        self.chars().filter(|c| *c == '\n').count()
    }
}

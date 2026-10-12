//! Sentence-boundary policy for prose, independent of Typst syntax and layout.
//!
//! ICU supplies contextual candidate boundaries. The default abbreviation list is
//! an explicit suppression policy, not a claim of linguistic certainty. Keeping
//! filtering here leaves room for locale and caller-supplied exceptions without
//! changing the markup adapter or the pretty-printing machinery.

use icu_segmenter::SentenceSegmenterBorrowed;

/// Contextual sentence segmentation with the formatter's default suppressions.
pub(crate) struct SentenceSegmenter {
    inner: SentenceSegmenterBorrowed<'static>,
}

impl SentenceSegmenter {
    pub(crate) fn new() -> Self {
        Self {
            inner: icu_segmenter::SentenceSegmenter::new(Default::default()),
        }
    }

    /// Returns UTF-8 offsets inside a complete prose run. Input endpoints are
    /// excluded: ending an analysis buffer does not establish a sentence ending.
    pub(crate) fn boundaries<'a>(&'a self, text: &'a str) -> impl Iterator<Item = usize> + 'a {
        self.inner.segment_str(text).filter(move |&offset| {
            offset != 0 && offset != text.len() && !ends_with_abbreviation(&text[..offset])
        })
    }
}

/// Terminal evidence for the compatibility fallback at opaque syntax boundaries.
/// Known prose must instead use [`SentenceSegmenter::boundaries`] with both sides
/// present, so lowercase continuations and closing punctuation retain context.
pub(crate) fn has_terminal_suffix(text: &str) -> bool {
    let text = trim_closers(text);
    text.ends_with(['.', '!', '?', '。', '！', '？']) && !ends_with_abbreviation(text)
}

/// Returns whether an unspaced boundary follows CJK sentence punctuation.
pub(crate) fn ends_with_cjk_terminator(text: &str) -> bool {
    trim_closers(text).ends_with(['。', '！', '？', '｡'])
}

fn trim_closers(text: &str) -> &str {
    const CLOSERS: &str = ")]}）】』」〉》\"'”’»›";
    text.trim_end_matches(|c: char| c.is_whitespace() || CLOSERS.contains(c))
}

fn ends_with_abbreviation(text: &str) -> bool {
    const OPENERS: &str = "([{（【『「〈《\"'“‘«‹";
    const ABBREVIATIONS: &[&str] = &[
        "dr.", "mr.", "mrs.", "ms.", "prof.", "sr.", "jr.", "st.", "vs.", "etc.", "e.g.", "i.e.",
        "al.", "vol.", "ed.", "pp.", "fig.", "sec.", "approx.",
    ];
    // An unspaced sentence terminator can precede the abbreviation in mixed
    // prose. Keep ASCII dots inside the token for dotted abbreviations and
    // code-like words, rather than treating every dot as a token separator.
    let word = trim_closers(text)
        .rsplit(|c: char| c.is_whitespace() || matches!(c, '!' | '?' | '。' | '！' | '？'))
        .next()
        .unwrap_or_default();
    let word = word.trim_start_matches(|c| OPENERS.contains(c));
    ABBREVIATIONS
        .iter()
        .any(|abbreviation| word.eq_ignore_ascii_case(abbreviation))
}

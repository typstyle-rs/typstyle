//! Sentence-boundary policy for prose, independent of Typst syntax and layout.
//!
//! ICU supplies contextual candidate boundaries. The default abbreviation list is
//! an explicit suppression policy, not a claim of linguistic certainty. Keeping
//! filtering here leaves room for locale and caller-supplied exceptions without
//! changing the markup adapter or the pretty-printing machinery.

use icu_segmenter::SentenceSegmenterBorrowed;

/// CJK terminators that also permit a break without whitespace.
const CJK_TERMINATORS: [char; 4] = ['。', '！', '？', '｡'];

/// Closing delimiters ignored when looking for terminal punctuation.
const CLOSERS: &str = ")]}）】』」〉》\"'”’»›";

/// Opening delimiters ignored when testing an abbreviation token.
const OPENERS: &str = "([{（【『「〈《\"'“‘«‹";

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
            offset != 0
                && offset != text.len()
                && !ends_with_abbreviation(trim_closers(&text[..offset]))
        })
    }
}

/// Terminal evidence for the compatibility fallback at opaque syntax boundaries.
/// Known prose must instead use [`SentenceSegmenter::boundaries`] with both sides
/// present, so lowercase continuations and closing punctuation retain context.
pub(crate) fn has_terminal_suffix(text: &str) -> bool {
    let text = trim_closers(text);
    text.ends_with(is_terminal_punctuation) && !ends_with_abbreviation(text)
}

/// Returns whether this punctuation can end a sentence.
pub(crate) fn is_terminal_punctuation(c: char) -> bool {
    matches!(c, '.' | '!' | '?') || CJK_TERMINATORS.contains(&c)
}

/// Returns whether an unspaced boundary follows CJK sentence punctuation.
pub(crate) fn ends_with_cjk_terminator(text: &str) -> bool {
    trim_closers(text).ends_with(CJK_TERMINATORS)
}

fn trim_closers(text: &str) -> &str {
    text.trim_end_matches(|c: char| c.is_whitespace() || CLOSERS.contains(c))
}

/// Checks for an abbreviation after trailing whitespace and closers have been stripped.
fn ends_with_abbreviation(text: &str) -> bool {
    const ABBREVIATIONS: &[&str] = &[
        "al.", "approx.", "dr.", "e.g.", "ed.", "etc.", "fig.", "i.e.", "jr.", "mr.", "mrs.",
        "ms.", "pp.", "prof.", "sec.", "sr.", "st.", "vol.", "vs.",
    ];
    if !text.ends_with('.') {
        return false;
    }

    // Inspect candidate suffixes without scanning the rest of an unspaced run.
    // An ASCII dot can belong to the same token as the abbreviation.
    ABBREVIATIONS.iter().any(|abbreviation| {
        let Some((prefix, suffix)) =
            text.split_at_checked(text.len().saturating_sub(abbreviation.len()))
        else {
            return false;
        };
        suffix.eq_ignore_ascii_case(abbreviation)
            && prefix
                .trim_end_matches(|c| OPENERS.contains(c))
                .chars()
                .next_back()
                .is_none_or(|c| c.is_whitespace() || (c != '.' && is_terminal_punctuation(c)))
    })
}

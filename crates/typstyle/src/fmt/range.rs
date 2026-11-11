//! Formats selected line ranges and assembles the resulting source.
//!
//! Range formatting expands each selection to the smallest enclosing syntax
//! node. Overlapping selections are merged so their edits cannot conflict, and
//! the complete source is returned with only the selected ranges replaced.

use std::{ops::Range, str::FromStr};

use anyhow::{Result, bail};
use typst_syntax::Source;
use typstyle_core::{Typstyle, partial::RangeResult};

/// A 1-based, inclusive line range selected for partial formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    /// The first selected line, counted from 1.
    first: usize,
    /// The last selected line, counted from 1 and inclusive.
    last: usize,
}

impl FromStr for LineRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, last) = s.split_once(':').unwrap_or((s, s));
        let invalid =
            || format!("invalid line range `{s}`: expected FIRST:LAST with 1-based line numbers");
        let first = first.parse::<usize>().map_err(|_| invalid())?;
        let last = last.parse::<usize>().map_err(|_| invalid())?;
        if first == 0 || last < first {
            return Err(format!(
                "invalid line range `{s}`: expected 1 <= FIRST <= LAST"
            ));
        }
        Ok(Self { first, last })
    }
}

/// A formatted selection: the byte range requested on the CLI and the core
/// formatter's result for it.
struct RangeEdit {
    requested: Range<usize>,
    result: RangeResult,
}

/// Formats the selected line ranges and returns the complete source with only
/// those ranges replaced.
///
/// The whole source must parse, matching the behavior of document formatting:
/// a source with syntax errors is returned as `None` rather than partially
/// rewritten. When expanded selections overlap, a containing result is kept when
/// possible and otherwise their requested ranges are merged and reformatted;
/// unaffected selections are reused.
pub(super) fn format(
    source: &Source,
    typstyle: &Typstyle,
    lines: &[LineRange],
) -> Result<Option<String>> {
    let content = source.text();
    // Validate the selection first so invalid line numbers always report an
    // error, even when the source itself cannot be parsed.
    let ranges = resolve_line_ranges(content, lines)?;
    if source.root().diagnosis().errors {
        return Ok(None);
    }
    let Some(mut edits) = ranges
        .into_iter()
        .map(|requested| format_range(source, typstyle, requested))
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(None);
    };

    loop {
        edits.sort_unstable_by_key(|edit| {
            (edit.result.source_range.start, edit.result.source_range.end)
        });
        let (merged, overlapped) = merge_overlaps(source, typstyle, edits)?;
        edits = merged;
        if !overlapped {
            break;
        }
    }

    let mut out = String::with_capacity(content.len());
    let mut cursor = 0;
    for edit in &edits {
        out.push_str(&content[cursor..edit.result.source_range.start]);
        out.push_str(&edit.result.content);
        cursor = edit.result.source_range.end;
    }
    out.push_str(&content[cursor..]);
    Ok(Some(out))
}

/// Formats one byte range, or returns `None` if the selection cannot be parsed.
fn format_range(
    source: &Source,
    typstyle: &Typstyle,
    requested: Range<usize>,
) -> Option<RangeEdit> {
    typstyle
        .format_source_range(source.clone(), requested.clone())
        .ok()
        .map(|result| RangeEdit { requested, result })
}

/// Merges every overlapping group of selections in one pass while keeping
/// unaffected selections as they are.
///
/// A group is replaced by the result of its containing selection when one
/// exists, and otherwise by the result of formatting the union of its requested
/// ranges. Returns the merged selections and whether any overlap was found.
/// The selections must be sorted by source range.
fn merge_overlaps(
    source: &Source,
    typstyle: &Typstyle,
    edits: Vec<RangeEdit>,
) -> Result<(Vec<RangeEdit>, bool)> {
    let mut groups: Vec<Vec<RangeEdit>> = Vec::new();
    let mut group_end = 0;
    let mut overlapped = false;
    for edit in edits {
        let start = edit.result.source_range.start;
        let end = edit.result.source_range.end;
        if start < group_end {
            overlapped = true;
            groups.last_mut().expect("group exists").push(edit);
        } else {
            groups.push(vec![edit]);
        }
        group_end = group_end.max(end);
    }

    let mut merged = Vec::with_capacity(groups.len());
    for mut group in groups {
        if group.len() == 1 {
            merged.push(group.pop().expect("singleton group"));
            continue;
        }
        let start = group
            .iter()
            .map(|edit| edit.result.source_range.start)
            .min()
            .expect("non-empty group");
        let end = group
            .iter()
            .map(|edit| edit.result.source_range.end)
            .max()
            .expect("non-empty group");
        // If one selection already formatted the containing node, its output
        // covers the whole group and the others can be discarded.
        if let Some(index) = group
            .iter()
            .position(|edit| edit.result.source_range == (start..end))
        {
            merged.push(group.swap_remove(index));
            continue;
        }
        let requested = group
            .iter()
            .map(|edit| edit.requested.start)
            .min()
            .expect("non-empty group")
            ..group
                .iter()
                .map(|edit| edit.requested.end)
                .max()
                .expect("non-empty group");
        let Some(edit) = format_range(source, typstyle, requested) else {
            bail!("failed to merge overlapping line range selections");
        };
        merged.push(edit);
    }
    Ok((merged, overlapped))
}

/// Converts 1-based inclusive line ranges into byte ranges, merging overlapping
/// and adjacent selections.
///
/// Lines are delimited by LF to match Jujutsu's line numbering. Typst's
/// [`Source::lines`] cannot be used here: it also treats vertical tab, form
/// feed, carriage return, next line, line separator, and paragraph separator as
/// line breaks, so its line indices and counts would not match Jujutsu's. The LF
/// boundaries and the line count are computed from the text directly instead.
fn resolve_line_ranges(text: &str, lines: &[LineRange]) -> Result<Vec<Range<usize>>> {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(index, _)| index + 1))
        .collect();
    let line_count = text.lines().count().max(1);
    let mut ranges = Vec::with_capacity(lines.len());
    for line in lines {
        if line.last > line_count {
            let line_word = if line_count == 1 { "line" } else { "lines" };
            bail!(
                "line range {}:{} is out of bounds; the input has {} {}",
                line.first,
                line.last,
                line_count,
                line_word
            );
        }
        let start = line_starts[line.first - 1];
        let end = line_starts.get(line.last).copied().unwrap_or(text.len());
        ranges.push(start..end);
    }
    ranges.sort_unstable_by_key(|range| (range.start, range.end));
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
            continue;
        }
        merged.push(range);
    }
    Ok(merged)
}

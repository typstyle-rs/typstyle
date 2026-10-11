//! A framework for testing Typst formatters that ensures consistent output.
//!
//! The entry point is [`FormatterHarness`], which loads sources into a Typst universe and
//! formats them with a caller-provided formatter. [`compare_worlds`] then evaluates and
//! compiles both the original and the formatted sources, checking that content, metadata,
//! pages, and rendered output remain identical.

mod cmp;
mod err;
mod harness;
mod image_diff;
mod text_diff;
mod world;

pub use cmp::{CheckingOptions, ComparisonInput, compare_worlds};
pub use err::ErrorSink;
pub use harness::{FormattedSources, FormatterHarness};
pub use world::{FormattedWorld, SourceMap};

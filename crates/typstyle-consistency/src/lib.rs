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

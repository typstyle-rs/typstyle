//! Shared support for Typstyle's test runner and benchmarks.
//!
//! [`source`] reads source files, [`fixtures`] loads fixture directives,
//! [`git_checkout`] validates cached Git checkouts, and [`docs_corpus`]
//! acquires the pinned Typst documentation corpus.

pub mod docs_corpus;
pub mod fixtures;
pub mod git_checkout;
pub mod source;

// Used by the test runner (`tests/runner/`) behind the `consistency` feature;
// kept in `[dependencies]` because Cargo does not support optional `[dev-dependencies]`.
#[cfg(feature = "consistency")]
extern crate typstyle_consistency as _;

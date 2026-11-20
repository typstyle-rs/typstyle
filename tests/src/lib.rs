//! Shared support for Typstyle's test runner and benchmarks.
//!
//! [`source`] reads source files and [`fixtures`] loads fixture directives.

pub mod fixtures;
pub mod source;

// Used by the test runner (`tests/runner/`) behind the `consistency` feature;
// kept in `[dependencies]` because Cargo does not support optional `[dev-dependencies]`.
#[cfg(feature = "consistency")]
extern crate typstyle_consistency as _;

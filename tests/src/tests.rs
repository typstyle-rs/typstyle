//! Entry point of the libtest-mimic test harness for Typstyle.
//!
//! Collects the unit, partial, and (with the `consistency` feature) repository
//! end-to-end tests and runs them.

#[cfg(feature = "consistency")]
#[path = "repo-e2e.rs"]
mod repo_e2e;

mod partial;
mod unit;

use std::error::Error;

use libtest_mimic::Arguments;

fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_mut)]
    let mut tests = unit::collect_tests()?;
    tests.append(&mut partial::collect_tests());
    #[cfg(feature = "consistency")]
    tests.append(&mut repo_e2e::collect_tests());

    libtest_mimic::run(&Arguments::from_args(), tests).exit();
}

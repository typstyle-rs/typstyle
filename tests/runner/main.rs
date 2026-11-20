//! Entry point of the libtest-mimic test harness for Typstyle.
//!
//! Collects the formatting and docs end-to-end trials, plus the repository
//! end-to-end trials when the `consistency` feature is enabled, and runs them.

mod e2e;
mod partial;
mod whole_file;

use std::error::Error;

use libtest_mimic::Arguments;

fn main() -> Result<(), Box<dyn Error>> {
    let mut tests = whole_file::collect_tests()?;
    tests.append(&mut partial::collect_tests());
    tests.append(&mut e2e::docs::collect_tests());
    #[cfg(feature = "consistency")]
    tests.append(&mut e2e::repos::collect_tests());

    libtest_mimic::run(&Arguments::from_args(), tests).exit();
}

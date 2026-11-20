//! Entry point of the libtest-mimic test harness for Typstyle.
//!
//! Collects the formatting trials and, with the `consistency` feature, the
//! repository end-to-end trials, and runs them.

#[cfg(feature = "consistency")]
mod e2e;
mod partial;
mod whole_file;

use std::error::Error;

use libtest_mimic::Arguments;

fn main() -> Result<(), Box<dyn Error>> {
    let mut tests = whole_file::collect_tests()?;
    tests.append(&mut partial::collect_tests());
    #[cfg(feature = "consistency")]
    tests.append(&mut e2e::repos::collect_tests());

    libtest_mimic::run(&Arguments::from_args(), tests).exit();
}

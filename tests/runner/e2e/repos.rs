//! End-to-end tests that format and compile real-world Typst repositories.
//!
//! Each test case from the `repos.toml` fixture is cloned, formatted, and
//! compiled to verify that formatting preserves the compiled output.

use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, anyhow};
use libtest_mimic::{Failed, Trial};
use serde::Deserialize;
use typstyle_consistency::{CheckingOptions, ErrorSink, FormattedSources, FormatterHarness};
use typstyle_core::Config;
use typstyle_tests::fixtures::test_dir;

use crate::e2e::{configs, make_formatter};

/// The parsed content of the repository test cases.
const TEST_CONFIG: &str = include_str!("repos.toml");

/// The parsed content of the `repos.toml` fixture.
#[derive(Deserialize)]
struct TestConfig {
    /// The repository test cases to run.
    testcase: Vec<Testcase>,
}

/// A repository test case, as declared in `repos.toml`.
#[derive(Debug, Clone, Deserialize)]
struct Testcase {
    /// The name of this test case. Does not require to be unique.
    name: String,

    /// The Git repository URL of this test case, used for `git clone`.
    git: String,

    /// An optional revision to check out.
    /// Will use the latest commit if not specified.
    #[serde(default)]
    rev: Option<String>,

    /// Whether to skip this test case. Useful for temporarily skipping a failing test.
    #[serde(default)]
    skip: bool,

    /// An optional entrypoint file for compilation, such as a manual.
    #[serde(default)]
    entrypoint: Option<String>,

    /// An optional directory containing example files to compile as a whole.
    #[serde(default)]
    examples: Option<String>,

    /// A set of file or directory names to skip formatting.
    /// You should include typst sources that are read and directly displayed,
    /// which will fail the tests if formatted.
    #[serde(default)]
    blacklist: HashSet<String>,
}

/// Creates one test for each non-skipped test case in `repos.toml`.
pub fn collect_tests() -> Vec<Trial> {
    let config = toml::from_str::<TestConfig>(TEST_CONFIG).unwrap();

    config
        .testcase
        .into_iter()
        .filter(|case| !case.skip)
        .map(|case| {
            Trial::test(case.name.clone().to_string(), move || {
                run_testcase(case).map_err(|e| Failed::from(e.to_string()))
            })
            .with_kind("e2e")
        })
        .collect()
}

/// Runs a single test case: clones its repository, formats it, and compiles the results.
///
/// The cloned repository is removed afterwards.
fn run_testcase(testcase: Testcase) -> anyhow::Result<()> {
    let cache_dir = test_dir().join("e2e");
    // do mkdir -p project_root/tests/e2e
    let _ = fs::create_dir_all(&cache_dir);
    let testcase_dir = cache_dir.join(&*testcase.name);

    clone_testcase_repo(&testcase, &testcase_dir)?;

    let fmt_configs = configs();
    check_testcase(&testcase, &testcase_dir, &fmt_configs)?;

    let _ = fs::remove_dir_all(testcase_dir);
    Ok(())
}

/// Clones the test case's repository into the target directory, checking out the
/// configured revision if any.
///
/// An existing directory is kept as is, for quick debugging.
fn clone_testcase_repo(testcase: &Testcase, testcase_dir: &Path) -> anyhow::Result<()> {
    if fs::exists(testcase_dir)? {
        return Ok(()); // for quick debugging
    }

    // clean the testcase_dir
    let _ = fs::remove_dir_all(testcase_dir);
    // do git clone with submodule
    std::process::Command::new("git")
        .arg("clone")
        .arg(&testcase.git)
        .arg(testcase_dir)
        .arg("--recurse-submodules")
        .output()
        .context("failed to clone repo")?;
    if let Some(rev) = testcase.rev.as_ref() {
        // do git checkout testcase.revision
        std::process::Command::new("git")
            .arg("checkout")
            .arg(rev)
            .current_dir(testcase_dir)
            .output()
            .context("failed to checkout revision")?;
    }
    Ok(())
}

/// Formats every file of the cloned repository with each given configuration and
/// compiles the results for comparison.
///
/// The test case must declare an `entrypoint` or an `examples` directory, and the
/// compiled output must match the original.
fn check_testcase(
    testcase: &Testcase,
    testcase_dir: &Path,
    named_configs: &[(&str, Config)],
) -> anyhow::Result<()> {
    if testcase.entrypoint.is_none() && testcase.examples.is_none() {
        return Err(anyhow!(
            "The testcase `{}` does not have entrypoint or examples",
            testcase.name
        ));
    }

    let mut err_sink = ErrorSink::new(format!("e2e test `{}`", testcase.name));

    let mut harness = FormatterHarness::new(testcase.name.clone(), testcase_dir.to_path_buf())?;
    harness.add_all_files(testcase_dir, &testcase.blacklist)?;
    if let Some(examples) = testcase.examples.as_ref() {
        let entry_vpath = Path::new("__examples__.typ");
        harness.add_all_files_in_one(entry_vpath, &testcase_dir.join(examples))?;
    };

    let base_world = harness.snapshot();
    let mut fmt_sources = vec![];
    for (name, config) in named_configs {
        let mut sub_sink = ErrorSink::new(format!("formatting with {name}"));
        fmt_sources.push(FormattedSources {
            name: (*name).to_string(),
            sources: harness.format(&base_world, make_formatter(config.clone()), &mut sub_sink)?,
        });
        sub_sink.sink_to(&mut err_sink);
    }

    let check_options = CheckingOptions {
        expect_compile_success: true,
        ..Default::default()
    };

    if let Some(entrypoint) = testcase.entrypoint.as_ref() {
        harness.compile_and_compare(
            fmt_sources.iter(),
            Path::new(&entrypoint),
            check_options,
            &mut err_sink,
        )?;
    }
    if testcase.examples.is_some() {
        let entry_vpath = Path::new("__examples__.typ");
        harness.compile_and_compare(
            fmt_sources.iter(),
            entry_vpath,
            check_options,
            &mut err_sink,
        )?;
    };

    if err_sink.is_ok() {
        Ok(())
    } else {
        (&err_sink).into()
    }
}

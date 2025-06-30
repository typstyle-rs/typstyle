use std::{collections::HashSet, fs, path::Path};

use anyhow::{anyhow, bail, Context};
use libtest_mimic::{Failed, Trial};
use serde::Deserialize;
use typst_syntax::Source;
use typstyle_consistency::{ErrorSink, FormattedSources, FormatterHarness};
use typstyle_core::{Config, Typstyle};

use crate::common::{fixtures_dir, test_dir};

#[derive(Deserialize)]
struct TestConfig {
    testcase: Vec<Testcase>,
}

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

struct NamedConfig {
    name: &'static str,
    config: Config,
}

pub(super) fn collect_tests() -> Vec<Trial> {
    let config = toml::from_str::<TestConfig>(
        &fs::read_to_string(fixtures_dir().join("e2e-repos.toml")).unwrap(),
    )
    .unwrap();

    config
        .testcase
        .into_iter()
        .map(|case| {
            Trial::test(case.name.clone().to_string(), move || {
                run_testcase(case).map_err(|e| Failed::from(e.to_string()))
            })
            .with_kind("e2e")
        })
        .collect()
}

fn run_testcase(testcase: Testcase) -> anyhow::Result<()> {
    let e2e_dir = test_dir().join("e2e");
    // do mkdir -p project_root/tests/e2e
    let _ = fs::create_dir_all(&e2e_dir);
    let testcase_dir = e2e_dir.join(&*testcase.name);

    clone_testcase_repo(&testcase, &testcase_dir)?;

    let fmt_configs = &[
        NamedConfig {
            name: "default",
            config: Config {
                reorder_import_items: true,
                ..Default::default()
            },
        },
        NamedConfig {
            name: "reflow",
            config: Config {
                reorder_import_items: true,
                wrap_text: true,
                ..Default::default()
            },
        },
    ];
    check_testcase(&testcase, &testcase_dir, fmt_configs)?;

    let _ = fs::remove_dir_all(testcase_dir);
    Ok(())
}

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

fn check_testcase(
    testcase: &Testcase,
    testcase_dir: &Path,
    named_configs: &[NamedConfig],
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
    for config in named_configs {
        let mut sub_sink = ErrorSink::new(format!("formatting with {}", config.name));
        fmt_sources.push(FormattedSources {
            name: config.name.to_string(),
            sources: harness.format(
                &base_world,
                make_formatter(config.config.clone()),
                &mut sub_sink,
            )?,
        });
        sub_sink.sink_to(&mut err_sink);
    }

    if let Some(entrypoint) = testcase.entrypoint.as_ref() {
        harness.compile_and_compare(
            fmt_sources.iter(),
            Path::new(&entrypoint),
            true,
            &mut err_sink,
        )?;
    }
    if testcase.examples.is_some() {
        let entry_vpath = Path::new("__examples__.typ");
        harness.compile_and_compare(fmt_sources.iter(), entry_vpath, true, &mut err_sink)?;
    };

    if err_sink.is_ok() {
        Ok(())
    } else {
        (&err_sink).into()
    }
}

fn make_formatter(config: Config) -> impl Fn(Source) -> anyhow::Result<String> {
    move |source| {
        if source.root().erroneous() {
            bail!("the file has syntax errors: {:?}", source.root().errors());
        }
        let t = Typstyle::new(config.clone());
        let first_pass = t.format_source(source).render().context("first pass")?;
        let second_pass = t.format_text(&first_pass).render().context("second pass")?;
        if first_pass != second_pass {
            bail!(
                "the formatting does not converge:\n{}",
                similar_asserts::SimpleDiff::from_str(
                    &first_pass,
                    &second_pass,
                    "first_pass",
                    "second_pass"
                )
            )
        }
        Ok(first_pass)
    }
}

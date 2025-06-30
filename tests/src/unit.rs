use std::{env, error::Error, fs, path::Path};

use insta::internals::Content;
use libtest_mimic::{Failed, Trial};
use typst_syntax::Source;
use typstyle_core::Typstyle;

use crate::common::{fixtures_dir, read_source_with_options};

/// Creates one test for each `.typ` file in the current directory or
/// sub-directories of the current directory.
pub fn collect_tests() -> Result<Vec<Trial>, Box<dyn Error>> {
    // always use colors in the console output
    console::set_colors_enabled(true);
    console::set_colors_enabled_stderr(true);

    fn make_snapshot_test(path: &Path, name: &str, width: usize) -> Trial {
        let path = path.to_path_buf();
        Trial::test(format!("{name} - {width}char"), move || {
            check_snapshot(&path, width)
        })
        .with_kind("snapshot")
    }

    fn make_convergence_test(path: &Path, name: &str, width: usize) -> Trial {
        let path = path.to_path_buf();
        Trial::test(format!("{name} - {width}char"), move || {
            check_convergence(&path, width)
        })
        .with_kind("convergence")
    }

    #[cfg(feature = "consistency")]
    fn make_consistency_test(path: &Path, name: &str, width: usize) -> Trial {
        let path = path.to_path_buf();
        Trial::test(format!("{name} - {width}char"), move || {
            check_output_consistency(&path, width)
        })
        .with_kind("consistency")
    }

    fn visit_dir(path: &Path, tests: &mut Vec<Trial>) -> Result<(), Box<dyn Error>> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();

            if file_type.is_dir() {
                // Handle directories
                if path.file_name() == Some("partial".as_ref()) {
                    // Ignore partial tests
                    continue;
                }
                visit_dir(&path, tests)?;
                continue;
            } else if !(file_type.is_file() && path.extension() == Some("typ".as_ref())) {
                continue;
            }
            // Handle .typ files
            let name = path
                .strip_prefix(env::current_dir()?)?
                .display()
                .to_string()
                .replace('\\', "/");

            let is_no_snap = path
                .file_stem()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('_'));
            if !is_no_snap {
                tests.extend([
                    make_snapshot_test(&path, &name, 0),
                    make_snapshot_test(&path, &name, 40),
                    make_snapshot_test(&path, &name, 80),
                    make_snapshot_test(&path, &name, 120),
                ]);
            }
            tests.extend([
                make_convergence_test(&path, &name, 0),
                make_convergence_test(&path, &name, 40),
                make_convergence_test(&path, &name, 80),
            ]);
            #[cfg(feature = "consistency")]
            tests.extend([
                make_consistency_test(&path, &name, 0),
                make_consistency_test(&path, &name, 40),
                make_consistency_test(&path, &name, 80),
            ]);
        }

        Ok(())
    }

    // We recursively look for `.typ` files, starting from the current directory.
    let mut tests = Vec::new();
    let current_dir = fixtures_dir();
    visit_dir(&current_dir, &mut tests)?;

    Ok(tests)
}

fn check_snapshot(path: &Path, width: usize) -> Result<(), Failed> {
    let (source, opt) = read_source_with_options(path)?;
    let mut cfg = opt.config;

    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings.set_snapshot_path(path.parent().unwrap().join("snap"));
    settings.set_input_file(path);
    if source.root().erroneous() {
        settings.set_raw_info(&Content::Map(vec![("erroneous".into(), true.into())]));
    }
    settings.bind(|| {
        let snap_name = format!("{}-{width}", path.file_name().unwrap().to_str().unwrap());
        if source.root().erroneous() {
            insta::assert_snapshot!(snap_name, "");
        } else {
            cfg.max_width = width;
            let mut formatted = Typstyle::new(cfg).format_source(source).render().unwrap();
            if formatted.starts_with('\n') {
                formatted.insert_str(0, "// DUMMY\n");
            }
            if formatted.ends_with("\n\n") {
                formatted.push_str("// DUMMY\n");
            }

            insta::assert_snapshot!(snap_name, formatted);
        }
    });
    Ok(())
}

fn check_convergence(path: &Path, width: usize) -> Result<(), Failed> {
    let (source, opt) = read_source_with_options(path)?;
    let mut cfg = opt.config;
    if source.root().erroneous() {
        return Ok(());
    }

    cfg.max_width = width;
    let t = Typstyle::new(cfg);
    let mut first_pass = t.format_source(source).render()?;
    for i in 0..=opt.relax_convergence {
        let new_source = Source::detached(&first_pass);
        if new_source.root().erroneous() {
            panic!(
                "the source becomes erroneous after {} iterations:\n{:#?}",
                i + 1,
                new_source.root().errors()
            )
        }
        let second_pass = t.format_source(new_source).render()?;
        if first_pass == second_pass {
            return Ok(());
        }
        if i == opt.relax_convergence {
            similar_asserts::assert_eq!(
                first_pass,
                second_pass,
                "formatting does not converge in {} iterations!",
                opt.relax_convergence
            );
        }
        first_pass = second_pass;
    }

    Ok(())
}

#[cfg(feature = "consistency")]
fn check_output_consistency(path: &Path, width: usize) -> Result<(), Failed> {
    use std::path::PathBuf;

    use typstyle_consistency::{ErrorSink, FormattedSources, FormatterHarness};

    let (source, opt) = read_source_with_options(path)?;
    let mut cfg = opt.config;
    if source.root().erroneous() {
        return Ok(());
    }

    cfg.max_width = width;
    let t = Typstyle::new(cfg);

    let mut err_sink = ErrorSink::new(format!("consistency {}", path.display()));

    let mut harness = FormatterHarness::new("".to_string(), PathBuf::new())?;
    let main_vpath = path.strip_prefix(fixtures_dir())?;
    harness.add_source_file(main_vpath, source.text())?;

    let base_world = harness.snapshot();
    let fmt_sources = FormattedSources {
        name: format!("{}char", width),
        sources: harness.format(
            &base_world,
            |source| Ok(t.format_source(source).render()?),
            &mut err_sink,
        )?,
    };

    harness.compile_and_compare([fmt_sources].iter(), main_vpath, false, &mut err_sink)?;

    if err_sink.is_ok() {
        Ok(())
    } else {
        Err(err_sink.into())
    }
}

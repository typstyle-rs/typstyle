use std::path::PathBuf;

use anyhow::Result;
use tinymist_world::SourceWorld;
use typst::{
    diag::SourceDiagnostic,
    ecow::EcoVec,
    foundations::Smart,
    layout::{Page, PagedDocument},
};

use crate::{sink_assert_eq, ErrorSink};

pub struct Compiled<'a> {
    pub name: String,
    pub world: &'a dyn SourceWorld,
    pub result: Result<PagedDocument, EcoVec<SourceDiagnostic>>,
}

pub fn compile_world(name: String, world: &dyn SourceWorld) -> Result<Compiled<'_>> {
    let result = typst::compile(world.as_world()).output;

    Ok(Compiled {
        name,
        world,
        result,
    })
}

pub fn compare_docs(
    before: &Compiled,
    after: &Compiled,
    require_compile: bool,
    err_sink: &mut ErrorSink,
) -> Result<()> {
    let mut sub_sink = ErrorSink::new(format!("comparing with `{}`", after.name));
    compare_docs_impl(before, after, require_compile, &mut sub_sink)?;
    sub_sink.sink_to(err_sink);
    Ok(())
}

fn compare_docs_impl(
    before: &Compiled,
    after: &Compiled,
    require_compile: bool,
    sub_sink: &mut ErrorSink,
) -> Result<()> {
    match (&before.result, &after.result) {
        (Ok(doc_bf), Ok(doc_af)) => {
            check_doc_meta(doc_bf, doc_af, sub_sink);
            check_png(doc_bf, doc_af, &before.name, &after.name, sub_sink)?;
        }
        (Err(e1), Err(e2)) => {
            if require_compile {
                sub_sink.push("Both docs failed to compile.".to_string());
                print_diagnostics(before.world, e1.iter())?;
                return Ok(());
            }

            sink_assert_eq!(
                sub_sink,
                e1.len(),
                e2.len(),
                "The error counts are not consistent"
            );
            for (e1, e2) in e1.iter().zip(e2.iter()) {
                sink_assert_eq!(
                    sub_sink,
                    e1.message,
                    e2.message,
                    "The error messages are not consistent after formatting"
                );
            }
        }
        (Err(e1), _) => {
            sub_sink.push("Original doc failed to compile.".to_string());
            print_diagnostics(before.world, e1.iter())?;
        }
        (_, Err(e2)) => {
            sub_sink.push("Formatted doc failed to compile.".to_string());
            print_diagnostics(after.world, e2.iter())?;
        }
    }

    Ok(())
}

fn check_doc_meta(left: &PagedDocument, right: &PagedDocument, sink: &mut ErrorSink) {
    sink_assert_eq!(
        sink,
        left.pages.len(),
        right.pages.len(),
        "The page counts are not consistent"
    );
    sink_assert_eq!(
        sink,
        left.info.title,
        right.info.title,
        "The titles are not consistent"
    );
    sink_assert_eq!(
        sink,
        left.info.author,
        right.info.author,
        "The authors are not consistent"
    );
    sink_assert_eq!(
        sink,
        left.info.description,
        right.info.description,
        "The descriptions are not consistent"
    );
    sink_assert_eq!(
        sink,
        left.info.keywords,
        right.info.keywords,
        "The keywords are not consistent"
    );
}

fn check_png(
    before: &PagedDocument,
    after: &PagedDocument,
    before_name: &str,
    after_name: &str,
    sink: &mut ErrorSink,
) -> anyhow::Result<()> {
    let render_png = |page: &Page, number: u64| {
        typst_render::render(
            &Page {
                frame: page.frame.clone(),
                fill: Smart::Auto,
                numbering: None,
                supplement: Default::default(),
                number,
            },
            2.0,
        )
    };

    let save_diff = std::env::var("TYPSTYLE_SAVE_DIFF").ok().map(PathBuf::from);
    fn fix_name(name: &str) -> String {
        name.replace(['/', '\\'], "__")
    }

    for (i, (page_bf, page_af)) in before.pages.iter().zip(after.pages.iter()).enumerate() {
        check_page(i, page_bf, page_af, sink);

        let png_bf = render_png(page_bf, i as u64);
        let png_af = render_png(page_af, i as u64);
        if png_bf == png_af {
            continue;
        }

        // If the environment variable "TYPSTYLE_SAVE_DIFF" is set, save the differing PNGs.
        if let Some(save_path) = save_diff.as_ref() {
            std::fs::create_dir_all(save_path).ok();
            let save_name_before = format!("{}_{}.png", fix_name(before_name), i);
            let save_name_after = format!("{}_{}.png", fix_name(after_name), i);
            png_bf.save_png(save_path.join(&save_name_before))?;
            png_af.save_png(save_path.join(&save_name_after))?;
            sink.push(format!(
                "The output are not consistent for page {i}.\nSaved diff PNGs: `{save_name_before}` and `{save_name_after}`"
            ));
        } else {
            sink.push(format!("The output are not consistent for page {i}."));
        }
    }

    Ok(())
}

fn check_page(index: usize, before: &Page, after: &Page, sink: &mut ErrorSink) {
    sink_assert_eq!(
        sink,
        before.fill,
        after.fill,
        "The fills of page {index} are not consistent."
    );
    sink_assert_eq!(
        sink,
        before.numbering,
        after.numbering,
        "The numberings of page {index} are not consistent."
    );
    sink_assert_eq!(
        sink,
        before.supplement,
        after.supplement,
        "The supplements of page {index} are not consistent."
    );
    sink_assert_eq!(
        sink,
        before.number,
        after.number,
        "The numbers of page {index} are not consistent."
    );
    sink_assert_eq!(
        sink,
        before.frame.size(),
        after.frame.size(),
        "The frame sizes of page {index} are not consistent."
    );
    sink_assert_eq!(
        sink,
        before.frame.items().count(),
        after.frame.items().count(),
        "The frame item counts of page {index} are not consistent."
    );
}

fn print_diagnostics<'d, 'files>(
    world: &'files dyn SourceWorld,
    errors: impl Iterator<Item = &'d SourceDiagnostic>,
) -> Result<()> {
    Ok(tinymist_world::print_diagnostics(
        world,
        errors,
        tinymist_world::DiagnosticFormat::Human,
    )?)
}

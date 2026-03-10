use std::path::PathBuf;

use anyhow::Result;
use tinymist_world::SourceWorld;
use typst::{
    diag::{SourceDiagnostic, SourceResult},
    ecow::EcoVec,
    foundations::{Content, Repr, Smart},
    layout::{Page, PagedDocument},
};

use crate::{ErrorSink, image_diff::compute_diff_pixmap, sink_assert_eq, text_diff::CodeDiff};

/// Options for comparing formatted documents with original ones.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CheckingOptions {
    /// Whether the document is expected to fail compilation (unused yet).
    pub expect_error: bool,
    /// Whether to enforce strict content equality.
    pub strict_content_equality: bool,
}

/// Input for document comparison, consisting of a name and a world to compile.
pub struct ComparisonInput<'a> {
    pub name: String,
    pub world: &'a dyn SourceWorld,
}

/// Evaluate the main source file to get its content without full compilation.
/// This is cheaper than full compilation and used for the first equality check.
fn eval_content(world: &dyn SourceWorld) -> SourceResult<Content> {
    let main = world.source(world.main()).unwrap();
    Ok(typst_shim::eval::eval_compat(world, &main)?.content())
}

/// Compile a world into a paged document.
fn compile_world(world: &dyn SourceWorld) -> Result<PagedDocument, EcoVec<SourceDiagnostic>> {
    typst::compile(world.as_world()).output
}

/// Compare two documents for consistency, checking content equality first,
/// then page equality, and finally rendering PNGs for any inconsistent pages.
///
/// The checking order is optimized to avoid expensive compilation when possible:
/// 1. Check content equality (cheap) - if equal, done
/// 2. Compile documents (expensive, only if content differs)
/// 3. Check page equality by hash - if equal, done
/// 4. Render PNGs only for pages that differ
pub fn compare_worlds(
    original: &ComparisonInput,
    formatted: &ComparisonInput,
    options: CheckingOptions,
    err_sink: &mut ErrorSink,
) -> Result<()> {
    let mut sub_sink = ErrorSink::new(format!("comparing with `{}`", formatted.name));
    compare_worlds_impl(original, formatted, options, &mut sub_sink)?;
    sub_sink.sink_to(err_sink);
    Ok(())
}

fn compare_worlds_impl(
    original: &ComparisonInput,
    formatted: &ComparisonInput,
    options: CheckingOptions,
    sink: &mut ErrorSink,
) -> Result<()> {
    // Step 1: Check content equality (cheap evaluation without full compilation)
    let original_content = eval_content(original.world);
    let formatted_content = eval_content(formatted.world);

    match (&original_content, &formatted_content) {
        (Ok(orig), Ok(fmt)) => {
            // if options.expect_error {
            //     sink.push("Both docs evaluated successfully, but were expected to fail.");
            // }
            if orig == fmt {
                // Content matches - done
                return Ok(());
            }
            // Content differs, need to compile and check pages
            if options.strict_content_equality {
                // If we didn't expect content differences, report it as hard error
                let orig_repr = orig.repr();
                let fmt_repr = fmt.repr();
                let diff = CodeDiff::new(&orig_repr, &fmt_repr);
                sink.push(format!(
                    "Content differs between original and formatted documents:\n{diff}"
                ));
                if orig_repr == fmt_repr {
                    sink.push("However, their representations are identical.");
                }
            }
        }
        (Err(orig_err), Err(fmt_err)) => {
            // Both failed to evaluate
            // if !options.expect_error {
            //     // Not expected to fail
            //     sink.push("Both docs failed to evaluate, but were expected to succeed.");
            // }
            // Check that errors are consistent
            check_error_consistency(orig_err, fmt_err, sink);
            return Ok(());
        }
        (Err(e), _) => {
            sink.push(format!("Failed to evaluate original document: {e:#?}"));
            return Ok(());
        }
        (_, Err(e)) => {
            sink.push(format!("Failed to evaluate formatted document: {e:#?}"));
            return Ok(());
        }
    }

    // Step 2: Compile documents (expensive, only done if content differs)
    let original_result = compile_world(original.world);
    let formatted_result = compile_world(formatted.world);

    match (&original_result, &formatted_result) {
        (Ok(orig_doc), Ok(fmt_doc)) => {
            // if options.expect_error {
            //     sink.push("Both docs compiled successfully, but were expected to fail.");
            //     return Ok(());
            // }
            // Both compiled successfully, check page equality
            check_pages_equal(orig_doc, fmt_doc, &original.name, &formatted.name, sink)?;
        }
        (Err(orig_err), Err(fmt_err)) => {
            // if !options.expect_error {
            //     // Not expected to fail
            //     sink.push("Both docs failed to compile, but were expected to succeed.");
            // }
            // Check that errors are consistent
            check_error_consistency(orig_err, fmt_err, sink);
            print_diagnostics(original.world, orig_err.iter())?;
        }
        (Err(e), _) => {
            sink.push("Original doc failed to compile.");
            print_diagnostics(original.world, e.iter())?;
        }
        (_, Err(e)) => {
            sink.push("Formatted doc failed to compile.");
            print_diagnostics(formatted.world, e.iter())?;
        }
    }

    Ok(())
}

/// Check that two error lists are consistent (same count and messages).
fn check_error_consistency(
    original_errors: &[SourceDiagnostic],
    formatted_errors: &[SourceDiagnostic],
    sink: &mut ErrorSink,
) {
    sink_assert_eq!(
        sink,
        original_errors.len(),
        formatted_errors.len(),
        "The error counts are not consistent"
    );
    for (orig, fmt) in original_errors.iter().zip(formatted_errors.iter()) {
        sink_assert_eq!(
            sink,
            orig.message,
            fmt.message,
            "The error messages are not consistent after formatting"
        );
    }
}

/// Check that document metadata (page count, title, author, etc.) is consistent.
fn check_doc_meta(original: &PagedDocument, formatted: &PagedDocument, sink: &mut ErrorSink) {
    sink_assert_eq!(
        sink,
        original.pages.len(),
        formatted.pages.len(),
        "The page counts are not consistent"
    );
    sink_assert_eq!(
        sink,
        original.info.title,
        formatted.info.title,
        "The titles are not consistent"
    );
    sink_assert_eq!(
        sink,
        original.info.author,
        formatted.info.author,
        "The authors are not consistent"
    );
    sink_assert_eq!(
        sink,
        original.info.description,
        formatted.info.description,
        "The descriptions are not consistent"
    );
    sink_assert_eq!(
        sink,
        original.info.keywords,
        formatted.info.keywords,
        "The keywords are not consistent"
    );
}

/// Check page equality between two documents.
/// First checks metadata, then uses hash-based comparison for pages,
/// and finally renders PNGs only for pages with metadata differences.
fn check_pages_equal(
    original: &PagedDocument,
    formatted: &PagedDocument,
    original_name: &str,
    formatted_name: &str,
    sink: &mut ErrorSink,
) -> Result<()> {
    // Check document metadata
    check_doc_meta(original, formatted, sink);

    if original.pages.len() != formatted.pages.len() {
        sink.push(format!(
            "Page count mismatch: original has {} pages, formatted has {} pages",
            original.pages.len(),
            formatted.pages.len()
        ));
        // No early return here - continue to check pages
    }

    // Check page equality using hash comparison (fast)
    if pages_equal_by_hash(original, formatted) {
        // All pages match - we're done!
        return Ok(());
    }

    // Pages differ by hash - check which pages have metadata differences and collect all with hash diff
    let mut pages_with_hash_diff = Vec::new();
    for (i, (orig_page, fmt_page)) in (original.pages.iter())
        .zip(formatted.pages.iter())
        .enumerate()
    {
        if page_hash(orig_page) == page_hash(fmt_page) {
            continue;
        }

        pages_with_hash_diff.push(i);
        let mut meta_sink = ErrorSink::new(String::new());
        check_page_meta(i, orig_page, fmt_page, &mut meta_sink);
        meta_sink.sink_to(sink);
    }

    // If no pages have hash differences
    if pages_with_hash_diff.is_empty() {
        // All pages match
        return Ok(());
    }

    // Pages differ unexpectedly - render all pages with hash differences
    render_diff_pages(
        original,
        formatted,
        original_name,
        formatted_name,
        &pages_with_hash_diff,
        sink,
    )?;

    Ok(())
}

/// Check if all pages in two documents are equal by comparing their hashes.
/// This is much faster than rendering and comparing PNGs.
fn pages_equal_by_hash(original: &PagedDocument, formatted: &PagedDocument) -> bool {
    if original.pages.len() != formatted.pages.len() {
        return false;
    }

    original
        .pages
        .iter()
        .zip(formatted.pages.iter())
        .all(|(p1, p2)| page_hash(p1) == page_hash(p2))
}

/// Compute a hash for a page to quickly check equality.
fn page_hash(page: &Page) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    page.hash(&mut hasher);
    hasher.finish()
}

/// Check if page metadata differs (fill, numbering, supplement, number, frame size/items).
fn check_page_meta(index: usize, original: &Page, formatted: &Page, sink: &mut ErrorSink) {
    sink_assert_eq!(
        sink,
        original.fill,
        formatted.fill,
        "The fill of page {index} differs."
    );
    sink_assert_eq!(
        sink,
        original.numbering,
        formatted.numbering,
        "The numbering of page {index} differs."
    );
    sink_assert_eq!(
        sink,
        original.supplement,
        formatted.supplement,
        "The supplement of page {index} differs."
    );
    sink_assert_eq!(
        sink,
        original.number,
        formatted.number,
        "The number of page {index} differs."
    );
    sink_assert_eq!(
        sink,
        original.frame.size(),
        formatted.frame.size(),
        "The frame size of page {index} differs."
    );
    sink_assert_eq!(
        sink,
        original.frame.items().count(),
        formatted.frame.items().count(),
        "The frame item count of page {index} differs."
    );
}

/// Render and save PNG images for pages with metadata differences.
/// Only renders pages that are in the provided list of indices with differences.
fn render_diff_pages(
    original: &PagedDocument,
    formatted: &PagedDocument,
    original_name: &str,
    formatted_name: &str,
    pages_with_diff: &[usize],
    sink: &mut ErrorSink,
) -> Result<()> {
    let total_pages = original.pages.len();

    let save_dir = std::env::var("TYPSTYLE_SAVE_DIFF").ok().map(PathBuf::from);
    if let Some(dir) = save_dir.as_ref() {
        std::fs::create_dir_all(dir)?;
    }

    // Helper to render a page to PNG at 2x resolution
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

    // Render and compare only pages with differences
    for &i in pages_with_diff {
        let page_num = i + 1;

        let orig_page = &original.pages[i];
        let fmt_page = &formatted.pages[i];

        let orig_png = render_png(orig_page, i as u64);
        let fmt_png = render_png(fmt_page, i as u64);

        if orig_png.width() != fmt_png.width() {
            sink.push(format!(
                "Page {page_num}/{total_pages} width differs: original is {}px, formatted is {}px.",
                orig_png.width(),
                fmt_png.width()
            ));
        }
        if orig_png.height() != fmt_png.height() {
            sink.push(format!(
                "Page {page_num}/{total_pages} height differs: original is {}px, formatted is {}px.",
                orig_png.height(),
                fmt_png.height()
            ));
        }

        let diff_pixel_count = (orig_png.pixels().iter())
            .zip(fmt_png.pixels())
            .filter(|(p1, p2)| p1 != p2)
            .count();
        if diff_pixel_count == 0 {
            // No pixel differences despite metadata differences
            continue;
        }
        let mut msg =
            format!("Page {page_num}/{total_pages} differs by {diff_pixel_count} pixels.");

        // PNGs differ - save them if environment variable is set
        if let Some(save_path) = save_dir.as_ref() {
            let diff_pixmap = compute_diff_pixmap(&orig_png, &fmt_png);

            let orig_filename = format!("{}_{page_num}.png", sanitize_filename(original_name));
            let fmt_filename = format!("{}_{page_num}.png", sanitize_filename(formatted_name));
            let diff_filename = format!("{}_{page_num}_diff.png", sanitize_filename(original_name));
            orig_png.save_png(save_path.join(&orig_filename))?;
            fmt_png.save_png(save_path.join(&fmt_filename))?;
            diff_pixmap.save_png(save_path.join(&diff_filename))?;

            msg.push_str(&format!(
                "\nSaved PNGs: `{orig_filename}`, `{fmt_filename}`, and `{diff_filename}`"
            ));
        } else {
            msg.push_str(" Set env TYPSTYLE_SAVE_DIFF to save PNGs.");
        }

        sink.push(msg);
    }

    Ok(())
}

/// Sanitize a filename by replacing path separators with underscores.
fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\'], "__")
}

/// Print diagnostics to stderr in human-readable format.
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

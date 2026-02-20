use std::ops::Range;

use js_sys::Error;
use serde::Serialize;
use typst_syntax::Source;
use typstyle_core::{
    Config, SpanMapping, Typstyle, format_ast, format_ast_with_mapping, partial::format_range_ast,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str =
    include_str!(concat!(env!("OUT_DIR"), "/generated_config_interface.ts"));

#[wasm_bindgen(start)]
pub fn run() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parses the content and returns its AST.
#[wasm_bindgen]
pub fn parse(text: &str) -> Result<String, Error> {
    let root = typst_syntax::parse(text);
    Ok(format_ast(&root))
}

/// Formats the content using the provided configuration.
#[wasm_bindgen]
pub fn format(
    text: &str,
    #[wasm_bindgen(unchecked_param_type = "Partial<Config>")] config: JsValue,
) -> Result<String, Error> {
    let config = parse_config(config)?;
    let t = Typstyle::new(config);
    t.format_text(text).render().map_err(into_error)
}

/// Get the pretty IR for the content.
#[wasm_bindgen]
pub fn format_ir(
    text: &str,
    #[wasm_bindgen(unchecked_param_type = "Partial<Config>")] config: JsValue,
) -> Result<String, Error> {
    let config = parse_config(config)?;
    let t = Typstyle::new(config);
    t.format_text(text).render_ir().map_err(into_error)
}

/// The result of formatting a range within content.
#[wasm_bindgen(getter_with_clone)]
pub struct FormatRangeResult {
    /// Start UTF-16 code unit index of the actual formatted range
    pub start: usize,
    /// End UTF-16 code unit index of the actual formatted range
    pub end: usize,
    /// The formatted text for the range
    pub text: String,
}

/// Formats a specific range within the content using the provided configuration.
/// The `start` and `end` parameters are UTF-16 code unit indices, matching JavaScript string indexing.
/// Returns the formatted text and the actual UTF-16 range that was formatted.
///
/// The returned [`FormatRangeResult`] contains the formatted text for the specified range,
/// along with the start and end indices (UTF-16) of the formatted region. These indices
/// can be used directly with JavaScript string methods such as `slice` or `substring`
/// to replace or extract the corresponding part of the original string.
#[wasm_bindgen]
pub fn format_range(
    text: &str,
    start: usize,
    end: usize,
    #[wasm_bindgen(unchecked_param_type = "Partial<Config>")] config: JsValue,
) -> Result<FormatRangeResult, Error> {
    let config = parse_config(config)?;
    let t = Typstyle::new(config);
    let source = Source::detached(text);
    let utf8_range = to_utf16_range(&source, start, end)?;

    match t.format_source_range(source.clone(), utf8_range) {
        Ok(result) => Ok(FormatRangeResult {
            start: source
                .lines()
                .byte_to_utf16(result.source_range.start)
                .expect("Invalid start index"),
            end: source
                .lines()
                .byte_to_utf16(result.source_range.end)
                .expect("Invalid end index"),
            text: result.content,
        }),
        Err(e) => Err(into_error(e)),
    }
}

/// Gets the IR (Intermediate Representation) of a specific range within the content.
/// Takes UTF-16 code unit indices and returns the IR of the actual range.
#[wasm_bindgen]
pub fn format_range_ir(
    text: &str,
    start: usize,
    end: usize,
    #[wasm_bindgen(unchecked_param_type = "Partial<Config>")] config: JsValue,
) -> Result<String, Error> {
    let config = parse_config(config)?;
    let t = Typstyle::new(config);
    let source = Source::detached(text);
    let utf8_range = to_utf16_range(&source, start, end)?;

    match t.format_source_range_ir(source, utf8_range) {
        Ok(result) => Ok(result.content),
        Err(e) => Err(into_error(e)),
    }
}

/// Gets the AST representation of a specific range within the content.
/// Takes UTF-16 code unit indices and returns the AST of the actual range.
#[wasm_bindgen]
pub fn get_range_ast(text: &str, start: usize, end: usize) -> Result<String, Error> {
    let source = Source::detached(text);
    let utf8_range = to_utf16_range(&source, start, end)?;

    match format_range_ast(&source, utf8_range) {
        Ok(result) => Ok(result.content),
        Err(e) => Err(into_error(e)),
    }
}

fn parse_config(config: JsValue) -> Result<Config, Error> {
    serde_wasm_bindgen::from_value(config).map_err(into_error)
}

fn to_utf16_range(source: &Source, start: usize, end: usize) -> Result<Range<usize>, Error> {
    Ok(utf16_to_byte(source, start)?..utf16_to_byte(source, end)?)
}

fn utf16_to_byte(source: &Source, utf16_idx: usize) -> Result<usize, Error> {
    source.lines().utf16_to_byte(utf16_idx).ok_or_else(|| {
        Error::new(&format!(
            "Invalid UTF-16 index: {utf16_idx} in source length: {}",
            source.lines().len_utf16()
        ))
    })
}

fn into_error<E: std::fmt::Display>(err: E) -> Error {
    Error::new(&err.to_string())
}

/// Result of parsing/formatting with span mappings, serialized to JS via serde.
#[derive(Serialize)]
struct MappingResult {
    text: String,
    mapping: Vec<SpanMapping>,
}

/// Convert SpanMappings from byte offsets to UTF-16 offsets in-place.
fn convert_mappings_to_utf16(source: &Source, output_text: &str, mappings: &mut [SpanMapping]) {
    let output_source = Source::detached(output_text);
    for m in mappings.iter_mut() {
        m.src_start = source.lines().byte_to_utf16(m.src_start).unwrap_or(0);
        m.src_end = source.lines().byte_to_utf16(m.src_end).unwrap_or(0);
        m.out_start = output_source
            .lines()
            .byte_to_utf16(m.out_start)
            .unwrap_or(0);
        m.out_end = output_source.lines().byte_to_utf16(m.out_end).unwrap_or(0);
    }
}

/// Parses the content and returns its AST along with span mappings.
/// Returns a JS object: { text: string, mapping: SpanMapping[] }
#[wasm_bindgen]
pub fn parse_with_mapping(text: &str) -> Result<JsValue, Error> {
    let root = typst_syntax::parse(text);
    let source = Source::detached(text);
    let (ast_text, mut mappings) = format_ast_with_mapping(&root);

    convert_mappings_to_utf16(&source, &ast_text, &mut mappings);
    let result = MappingResult {
        text: ast_text,
        mapping: mappings,
    };
    serde_wasm_bindgen::to_value(&result).map_err(into_error)
}

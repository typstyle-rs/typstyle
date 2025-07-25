use js_sys::Error;
use typst_syntax::Source;
use typstyle_core::{partial::get_node_for_range, Config, Typstyle};
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
    Ok(format!("{root:#?}"))
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
    /// Start byte offset (in UTF8) of the actual formatted range
    pub start: usize,
    /// End byte offset (in UTF8) of the actual formatted range
    pub end: usize,
    /// The formatted text for the range
    pub text: String,
}

impl From<typstyle_core::partial::RangeResult> for FormatRangeResult {
    fn from(value: typstyle_core::partial::RangeResult) -> Self {
        FormatRangeResult {
            text: value.content,
            start: value.source_range.start,
            end: value.source_range.end,
        }
    }
}

/// Formats a specific range within the content using the provided configuration.
/// Returns the formatted text and the actual range that was formatted.
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
    let range = start..end;

    match t.format_source_range(source, range) {
        Ok(result) => Ok(result.into()),
        Err(e) => Err(into_error(e)),
    }
}

/// Gets the IR (Intermediate Representation) of a specific range within the content.
/// Returns the IR of the actual range.
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
    let range = start..end;

    match t.format_source_range_ir(source, range) {
        Ok(result) => Ok(result.content),
        Err(e) => Err(into_error(e)),
    }
}

/// Gets the AST representation of a specific range within the content.
/// Returns the AST of the actual range.
#[wasm_bindgen]
pub fn get_range_ast(text: &str, start: usize, end: usize) -> Result<String, Error> {
    let source = Source::detached(text);
    let range = start..end;

    match get_node_for_range(&source, range) {
        Ok(node) => Ok(format!("{node:#?}")),
        Err(e) => Err(into_error(e)),
    }
}

fn parse_config(config: JsValue) -> Result<Config, Error> {
    serde_wasm_bindgen::from_value(config).map_err(into_error)
}

fn into_error<E: std::fmt::Display>(err: E) -> Error {
    Error::new(&err.to_string())
}

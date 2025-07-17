use js_sys::Error;
use typstyle_core::{Config, Typstyle};
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

fn parse_config(config: JsValue) -> Result<Config, Error> {
    serde_wasm_bindgen::from_value(config).map_err(into_error)
}

fn into_error<E: std::fmt::Display>(err: E) -> Error {
    Error::new(&err.to_string())
}

#[wasm_bindgen]
pub fn pretty_print_wasm(content: &str, width: usize) -> String {
    let config = Config::new().with_width(width);
    let t = Typstyle::new(config);
    t.format_text(content)
        .render()
        .unwrap_or_else(|_| content.to_string())
}

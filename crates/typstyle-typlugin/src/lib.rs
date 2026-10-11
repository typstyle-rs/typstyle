//! Embeds Typstyle into Typst as a WASM plugin via the wasm-minimal-protocol.
//!
//! All exported functions take the source text and a JSON-serialized [`Config`] as UTF-8
//! bytes and return UTF-8 bytes, so they can be called from Typst through the plugin API.

use typstyle_core::{Config, Typstyle, format_ast};
use wasm_minimal_protocol::*;

initiate_protocol!();

/// A result carrying an error message across the plugin boundary.
type StrResult<T> = Result<T, String>;
/// The result of a fallible plugin function: UTF-8 bytes or an error message.
type WasmResult = Result<Vec<u8>, String>;

/// Parses the content and returns its AST.
#[wasm_func]
pub fn parse(text: &[u8]) -> WasmResult {
    let text = parse_text(text)?;

    let root = typst_syntax::parse(text);
    let ret = format_ast(&root);

    Ok(ret.into_bytes())
}

/// Formats the content using the provided configuration.
#[wasm_func]
pub fn format(text: &[u8], config: &[u8]) -> WasmResult {
    let text = parse_text(text)?;
    let config = parse_config(config)?;

    let t = Typstyle::new(config);
    let ret = t
        .format_text(text)
        .render()
        .map_err(|e| format!("Failed to format: {e}"))?;

    Ok(ret.into_bytes())
}

/// Returns empty bytes in case of failure.
#[wasm_func]
pub fn try_format(text: &[u8], config: &[u8]) -> Vec<u8> {
    format(text, config).unwrap_or_else(|_| vec![])
}

/// Formats the content, returns original text with error comment on error.
#[wasm_func]
pub fn format_with_error(text: &[u8], config: &[u8]) -> Vec<u8> {
    match format(text, config) {
        Ok(formatted) => formatted,
        Err(e) => {
            let original = String::from_utf8_lossy(text);
            format!("// Typstyle error: {e}\n{original}").into_bytes()
        }
    }
}

/// Get the pretty IR for the content.
#[wasm_func]
pub fn format_ir(text: &[u8], config: &[u8]) -> WasmResult {
    let text = parse_text(text)?;
    let config = parse_config(config)?;

    let t = Typstyle::new(config);
    let ret = t
        .format_text(text)
        .render_ir()
        .map_err(|e| format!("Failed to format: {e}"))?;

    Ok(ret.into_bytes())
}

/// Interprets the input bytes as UTF-8 text.
fn parse_text(text: &[u8]) -> StrResult<&str> {
    std::str::from_utf8(text).map_err(|_| "Invalid UTF-8 in input text".to_string())
}

/// Deserializes the formatter configuration from JSON bytes.
fn parse_config(config: &[u8]) -> StrResult<Config> {
    serde_json::from_slice(config).map_err(|e| format!("Failed to parse config: {e}"))
}

//! Metadata for canonical formatter options.
//!
//! The `#[options]` macro in `typstyle-macros` emits one [`OptionSpec`] per field. Downstream
//! generators consume that table together with the resolved defaults to produce TypeScript
//! definitions, JSON Schema, and UI metadata without re-parsing Rust source.

/// The value shape of an option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    /// A boolean toggle.
    Boolean,
    /// An integer.
    Integer,
    /// A floating-point number.
    Float,
    /// A free-form string.
    String,
    /// A closed set of string values.
    Enum,
}

/// A single option's declaration metadata.
#[derive(Debug, Clone, Copy)]
pub struct OptionSpec {
    /// Configuration key, which is also the Rust field name.
    pub key: &'static str,
    /// Human-readable label for user interfaces.
    pub label: &'static str,
    /// Documentation comment from the Rust field.
    pub description: &'static str,
    /// Value shape.
    pub kind: OptionKind,
    /// Allowed values for [`OptionKind::Enum`], in declaration order.
    pub choices: &'static [&'static str],
    /// UI group name.
    pub group: &'static str,
    /// Optional UI widget override.
    pub widget: Option<&'static str>,
    /// Rust type name, used for enum aliases.
    pub type_name: &'static str,
    /// Declaration order.
    pub order: usize,
}

/// String conversion for enum-valued options.
///
/// The macro uses this trait to build `clap` value parsers and metadata choices without
/// requiring `ValueEnum` derives on library enums.
pub trait OptionEnum: Sized + 'static {
    /// Allowed wire values, in display order.
    const CHOICES: &'static [&'static str];

    /// Converts a wire value into the enum variant.
    fn from_choice(value: &str) -> Option<Self>;
}

/// An invalid resolved option value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid value for `{field}`: {message}")]
pub struct OptionsError {
    /// The option key that failed validation.
    pub field: &'static str,
    /// A human-readable explanation.
    pub message: String,
}

impl OptionsError {
    /// Creates a validation error for the given option key.
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

/// Backwards-compatible alias for [`OptionsError`].
pub type ConfigError = OptionsError;

use std::collections::{BTreeSet, HashSet};

use anyhow::{Context, Result, bail, ensure};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Definitions {
    #[serde(default)]
    option: Vec<OptionDefinition>,
    #[serde(rename = "enum", default)]
    enums: Vec<EnumDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptionDefinition {
    name: String,
    #[serde(rename = "type")]
    ty: String,
    default: toml::Value,
    description: String,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    widget: Option<String>,
    #[serde(default)]
    cli: CliDefinition,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliDefinition {
    #[serde(default)]
    short: Option<String>,
    #[serde(default)]
    long: Option<String>,
    #[serde(default)]
    visible_alias: Option<String>,
    #[serde(default)]
    visible_short_alias: Option<String>,
    #[serde(default)]
    value_name: Option<String>,
    #[serde(default)]
    implicit_value: Option<String>,
    #[serde(default)]
    inverse: Option<InverseDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InverseDefinition {
    long: String,
    description: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumDefinition {
    name: String,
    description: String,
    default: String,
    variants: Vec<EnumVariantDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumVariantDefinition {
    name: String,
    value: String,
    description: String,
}

pub(crate) struct Model {
    pub(crate) enums: Vec<EnumModel>,
    pub(crate) options: Vec<OptionModel>,
}

pub(crate) struct EnumModel {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) default: String,
    pub(crate) variants: Vec<EnumVariant>,
}

pub(crate) struct EnumVariant {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) description: String,
}

#[derive(Clone, Copy)]
pub(crate) enum TypeKind {
    Boolean,
    Integer,
    Float,
    String,
    Enum(usize),
}

pub(crate) struct OptionModel {
    pub(crate) name: String,
    pub(crate) ty: String,
    pub(crate) kind: TypeKind,
    pub(crate) default: toml::Value,
    pub(crate) default_expr: TokenStream,
    pub(crate) default_display: String,
    pub(crate) description: String,
    pub(crate) group: String,
    pub(crate) label: String,
    pub(crate) widget: String,
    pub(crate) cli: CliModel,
    pub(crate) order: usize,
}

pub(crate) struct CliModel {
    pub(crate) short: Option<char>,
    pub(crate) long: String,
    pub(crate) visible_alias: Option<String>,
    pub(crate) visible_short_alias: Option<char>,
    pub(crate) value_name: Option<String>,
    pub(crate) implicit_value: Option<String>,
    pub(crate) inverse: Option<InverseModel>,
}

pub(crate) struct InverseModel {
    pub(crate) field: Ident,
    pub(crate) long: String,
    pub(crate) description: String,
}

impl Model {
    pub(crate) fn parse(text: &str) -> Result<Self> {
        let definitions: Definitions = toml::from_str(text)?;
        ensure!(
            !definitions.option.is_empty(),
            "options.toml must declare at least one option"
        );

        let mut enum_names = HashSet::new();
        let mut enums = Vec::new();
        for definition in definitions.enums {
            ensure!(
                enum_names.insert(definition.name.clone()),
                "duplicate enum `{}`",
                definition.name
            );
            enums.push(EnumModel::new(definition)?);
        }

        let mut model = Self {
            enums,
            options: Vec::new(),
        };
        let mut names = HashSet::new();
        for (order, definition) in definitions.option.into_iter().enumerate() {
            let option = OptionModel::new(definition, order, &model.enums)?;
            ensure!(
                names.insert(option.name.clone()),
                "duplicate option `{}`",
                option.name
            );
            model.options.push(option);
        }
        model.validate_cli()?;
        Ok(model)
    }

    fn validate_cli(&self) -> Result<()> {
        let mut longs = BTreeSet::new();
        let mut shorts = BTreeSet::new();
        for option in &self.options {
            for long in std::iter::once(&option.cli.long).chain(option.cli.visible_alias.iter()) {
                ensure!(
                    longs.insert(long.clone()),
                    "duplicate CLI long option `{long}`"
                );
            }
            for short in option
                .cli
                .short
                .into_iter()
                .chain(option.cli.visible_short_alias)
            {
                ensure!(shorts.insert(short), "duplicate CLI short option `{short}`");
            }
            if let Some(inverse) = &option.cli.inverse {
                ensure!(
                    longs.insert(inverse.long.clone()),
                    "duplicate CLI long option `{}`",
                    inverse.long
                );
            }
        }
        Ok(())
    }

    pub(crate) fn enum_model(&self, index: usize) -> &EnumModel {
        &self.enums[index]
    }
}

impl EnumModel {
    fn new(definition: EnumDefinition) -> Result<Self> {
        ensure!(
            is_identifier(&definition.name),
            "invalid enum name `{}`",
            definition.name
        );
        ensure!(
            !definition.variants.is_empty(),
            "enum `{}` must declare at least one variant",
            definition.name
        );
        ensure!(
            definition
                .variants
                .iter()
                .any(|v| v.value == definition.default),
            "enum `{}` has no variant with value `{}`",
            definition.name,
            definition.default
        );

        let mut names = HashSet::new();
        let mut values = HashSet::new();
        let mut variants = Vec::new();
        for variant in definition.variants {
            ensure!(
                is_identifier(&variant.name),
                "invalid variant name `{}` in enum `{}`",
                variant.name,
                definition.name
            );
            ensure!(
                names.insert(variant.name.clone()),
                "duplicate variant `{}` in enum `{}`",
                variant.name,
                definition.name
            );
            ensure!(
                values.insert(variant.value.clone()),
                "duplicate value `{}` in enum `{}`",
                variant.value,
                definition.name
            );
            variants.push(EnumVariant {
                name: variant.name,
                value: variant.value,
                description: variant.description,
            });
        }

        Ok(Self {
            name: definition.name,
            description: definition.description,
            default: definition.default,
            variants,
        })
    }
}

impl OptionModel {
    fn new(definition: OptionDefinition, order: usize, enums: &[EnumModel]) -> Result<Self> {
        ensure!(
            is_identifier(&definition.name),
            "invalid option name `{}`",
            definition.name
        );
        ensure!(is_identifier(&definition.ty), "invalid option type");

        let kind = match definition.ty.as_str() {
            "bool" => TypeKind::Boolean,
            "usize" | "u8" | "u16" | "u32" | "u64" | "isize" | "i8" | "i16" | "i32" | "i64" => {
                TypeKind::Integer
            }
            "f32" | "f64" => TypeKind::Float,
            "String" => TypeKind::String,
            ty => {
                let index = enums
                    .iter()
                    .position(|candidate| candidate.name == ty)
                    .with_context(|| format!("unknown option type `{ty}`"))?;
                TypeKind::Enum(index)
            }
        };

        let (default_expr, default_display) = resolve_default(&definition, kind, enums)?;
        let label = definition
            .label
            .unwrap_or_else(|| humanize(&definition.name));
        let group = definition.group.unwrap_or_else(|| "General".to_owned());
        let widget = definition
            .widget
            .unwrap_or_else(|| default_widget(kind).to_owned());
        let short = parse_short(definition.cli.short.as_deref(), "short")?;
        let visible_short_alias = parse_short(
            definition.cli.visible_short_alias.as_deref(),
            "visible_short_alias",
        )?;
        let long = definition
            .cli
            .long
            .unwrap_or_else(|| definition.name.replace('_', "-"));
        validate_long(&long, "long")?;
        if let Some(alias) = &definition.cli.visible_alias {
            validate_long(alias, "visible_alias")?;
        }
        let value_name = definition
            .cli
            .value_name
            .or_else(|| matches!(kind, TypeKind::Boolean).then(|| "BOOL".to_owned()));
        if let Some(value) = &definition.cli.implicit_value {
            ensure!(
                matches!(kind, TypeKind::Enum(_)),
                "`implicit_value` is only supported for enum options"
            );
            let TypeKind::Enum(index) = kind else {
                unreachable!()
            };
            ensure!(
                enums[index]
                    .variants
                    .iter()
                    .any(|variant| &variant.value == value),
                "implicit value `{value}` is not a variant of `{}`",
                enums[index].name
            );
        }
        let inverse = definition
            .cli
            .inverse
            .map(|inverse| {
                ensure!(
                    matches!(kind, TypeKind::Boolean),
                    "inverse CLI flags are only supported for boolean options"
                );
                validate_long(&inverse.long, "inverse.long")?;
                let field_name = inverse.long.replace('-', "_");
                ensure!(
                    is_identifier(&field_name),
                    "invalid inverse flag name `{}`",
                    inverse.long
                );
                ensure!(
                    !inverse.description.trim().is_empty(),
                    "inverse flag `{}` needs a description",
                    inverse.long
                );
                Ok(InverseModel {
                    field: rust_ident(&field_name),
                    long: inverse.long,
                    description: inverse.description,
                })
            })
            .transpose()?;

        Ok(Self {
            name: definition.name,
            ty: definition.ty,
            kind,
            default: definition.default,
            default_expr,
            default_display,
            description: definition.description,
            group,
            label,
            widget,
            cli: CliModel {
                short,
                long,
                visible_alias: definition.cli.visible_alias,
                visible_short_alias,
                value_name,
                implicit_value: definition.cli.implicit_value,
                inverse,
            },
            order,
        })
    }

    pub(crate) fn type_tokens(&self) -> TokenStream {
        let ty = rust_ident(&self.ty);
        quote!(#ty)
    }

    pub(crate) fn cli_type_tokens(&self, model: &Model) -> TokenStream {
        match self.kind {
            TypeKind::Enum(index) => {
                let ty = rust_ident(&model.enum_model(index).name);
                quote!(typstyle_core::#ty)
            }
            _ => self.type_tokens(),
        }
    }

    pub(crate) fn default_json(&self) -> Value {
        match &self.default {
            toml::Value::Boolean(value) => json!(value),
            toml::Value::Integer(value) => json!(value),
            toml::Value::Float(value) => json!(value),
            toml::Value::String(value) => json!(value),
            _ => unreachable!("validated option defaults"),
        }
    }

    pub(crate) fn choices<'a>(&self, model: &'a Model) -> &'a [EnumVariant] {
        match self.kind {
            TypeKind::Enum(index) => &model.enum_model(index).variants,
            _ => &[],
        }
    }

    pub(crate) fn help(&self) -> String {
        let mut text = self.description.trim().to_owned();
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(&format!("[default: {}]", self.default_display));
        text
    }
}

fn resolve_default(
    definition: &OptionDefinition,
    kind: TypeKind,
    enums: &[EnumModel],
) -> Result<(TokenStream, String)> {
    match kind {
        TypeKind::Boolean => {
            let value = definition
                .default
                .as_bool()
                .with_context(|| format!("default for `{}` must be a boolean", definition.name))?;
            Ok((quote!(#value), value.to_string()))
        }
        TypeKind::Integer => {
            let value = definition
                .default
                .as_integer()
                .with_context(|| format!("default for `{}` must be an integer", definition.name))?;
            if !definition.ty.starts_with('i') && value < 0 {
                bail!(
                    "default for unsigned option `{}` must be non-negative",
                    definition.name
                );
            }
            let literal = Literal::i64_unsuffixed(value);
            Ok((quote!(#literal), value.to_string()))
        }
        TypeKind::Float => {
            let value = definition
                .default
                .as_float()
                .or_else(|| definition.default.as_integer().map(|value| value as f64))
                .with_context(|| format!("default for `{}` must be a number", definition.name))?;
            let literal = Literal::f64_unsuffixed(value);
            Ok((quote!(#literal), value.to_string()))
        }
        TypeKind::String => {
            let value = definition
                .default
                .as_str()
                .with_context(|| format!("default for `{}` must be a string", definition.name))?;
            Ok((quote!(#value), value.to_owned()))
        }
        TypeKind::Enum(index) => {
            let value = definition.default.as_str().with_context(|| {
                format!("default for `{}` must be a variant value", definition.name)
            })?;
            let model = &enums[index];
            let variant = model
                .variants
                .iter()
                .find(|variant| variant.value == value)
                .with_context(|| {
                    format!("default `{value}` is not a variant of `{}`", model.name)
                })?;
            let ty = rust_ident(&model.name);
            let variant = rust_ident(&variant.name);
            Ok((quote!(#ty::#variant), value.to_owned()))
        }
    }
}

fn default_widget(kind: TypeKind) -> &'static str {
    match kind {
        TypeKind::Boolean => "checkbox",
        TypeKind::Integer | TypeKind::Float => "number",
        TypeKind::String => "text",
        TypeKind::Enum(_) => "select",
    }
}

fn parse_short(value: Option<&str>, field: &str) -> Result<Option<char>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut chars = value.chars();
    let character = chars
        .next()
        .with_context(|| format!("`{field}` must be a single character"))?;
    ensure!(
        chars.next().is_none(),
        "`{field}` must be a single character"
    );
    ensure!(
        character.is_ascii_alphanumeric(),
        "`{field}` must be an ASCII letter or digit"
    );
    Ok(Some(character))
}

fn validate_long(value: &str, field: &str) -> Result<()> {
    ensure!(!value.is_empty(), "`{field}` must not be empty");
    ensure!(
        value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-'),
        "`{field}` may only contain ASCII letters, digits, and hyphens"
    );
    ensure!(
        !value.starts_with('-') && !value.ends_with('-'),
        "`{field}` must not start or end with a hyphen"
    );
    Ok(())
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(character) if character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

pub(crate) fn rust_ident(value: &str) -> Ident {
    Ident::new(value, Span::call_site())
}

fn humanize(name: &str) -> String {
    let mut words = name.split('_');
    let first = words.next().unwrap_or_default();
    let mut label = String::from(first);
    if let Some(first_char) = label.chars().next() {
        label.replace_range(
            0..first_char.len_utf8(),
            &first_char.to_uppercase().to_string(),
        );
    }
    for word in words {
        label.push(' ');
        label.push_str(word);
    }
    label
}

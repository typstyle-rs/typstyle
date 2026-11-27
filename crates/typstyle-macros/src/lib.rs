//! Procedural macro for canonical Typstyle option definitions.
//!
//! `#[options]` keeps the annotated struct as the resolved configuration and generates:
//!
//! - a `Partial*` companion with one `Option<T>` per field,
//! - `Default` for both types,
//! - merge, apply, resolve, and validation methods,
//! - `serde` and `clap` derives behind crate features,
//! - a static `typstyle_core::option::OptionSpec` table for documentation and binding
//!   generation.
//!
//! The macro deliberately does not generate serialization internals. It only emits ordinary
//! derives and attribute forwarding.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Expr, Field, Fields, Ident, ItemStruct, Lit, Meta, Token, Type, parse::Parser,
    parse_macro_input, punctuated::Punctuated,
};

/// Declares a canonical option struct with a generated partial companion.
#[proc_macro_attribute]
pub fn options(args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    match expand(args.into(), item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand(args: TokenStream2, item: ItemStruct) -> syn::Result<TokenStream2> {
    let args = MacroArgs::parse(args)?;
    let name = &item.ident;
    let partial_name = args
        .partial
        .clone()
        .unwrap_or_else(|| format_ident!("Partial{name}"));
    let serde_feature = &args.serde_feature;
    let clap_feature = &args.clap_feature;

    let fields = match &item.fields {
        Fields::Named(named) => named.named.iter().cloned().collect::<Vec<_>>(),
        _ => {
            return Err(syn::Error::new_spanned(
                &item,
                "`#[options]` requires a struct with named fields",
            ));
        }
    };
    if fields.is_empty() {
        return Err(syn::Error::new_spanned(
            &item,
            "`#[options]` requires at least one field",
        ));
    }

    let specs = fields
        .iter()
        .enumerate()
        .map(|(index, field)| FieldSpec::parse(field, index))
        .collect::<syn::Result<Vec<_>>>()?;

    let mut clean_item = item.clone();
    if let Fields::Named(named) = &mut clean_item.fields {
        for field in &mut named.named {
            field.attrs.retain(|attr| {
                !attr.path().is_ident("option")
                    && !attr.path().is_ident("cli")
                    && !attr.path().is_ident("ui")
            });
        }
    }

    let partial_fields = specs.iter().map(|spec| {
        let ident = &spec.ident;
        let ty = &spec.ty;
        let doc = &spec.doc;
        let cli_attr = spec.cli_attr_tokens(clap_feature);
        quote! {
            #[doc = #doc]
            #[cfg_attr(feature = #serde_feature, serde(skip_serializing_if = "Option::is_none"))]
            #cli_attr
            pub #ident: Option<#ty>,
        }
    });

    let default_fields = specs.iter().map(|spec| {
        let ident = &spec.ident;
        let default = &spec.default;
        quote! { #ident: #default }
    });
    let none_fields = specs.iter().map(|spec| {
        let ident = &spec.ident;
        quote! { #ident: None }
    });
    let merge_fields = specs.iter().map(|spec| {
        let ident = &spec.ident;
        quote! { #ident: newer.#ident.or(self.#ident) }
    });
    let apply_fields = specs.iter().map(|spec| {
        let ident = &spec.ident;
        quote! {
            if let Some(value) = overrides.#ident {
                this.#ident = value;
            }
        }
    });
    let spec_entries = specs.iter().map(FieldSpec::metadata);
    let parser_fns = specs
        .iter()
        .filter_map(|spec| spec.clap_parser(clap_feature));

    Ok(quote! {
        #clean_item

        #[doc = concat!("Explicit overrides for [`", stringify!(#name), "`].")]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = #serde_feature, derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = #serde_feature, serde(default, deny_unknown_fields))]
        #[cfg_attr(feature = #clap_feature, derive(clap::Args))]
        pub struct #partial_name {
            #(#partial_fields)*
        }

        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl Default for #partial_name {
            fn default() -> Self {
                Self {
                    #(#none_fields),*
                }
            }
        }

        impl #partial_name {
            #[doc = "Combines overrides, with `newer` taking precedence over `self`."]
            pub fn merge(self, newer: Self) -> Self {
                Self {
                    #(#merge_fields),*
                }
            }

            #[doc = "Resolves these overrides against the built-in defaults."]
            pub fn resolve(self) -> Result<#name, ::typstyle_core::option::OptionsError> {
                #name::default().apply(self)
            }
        }

        impl #name {
            #[doc = "Option metadata in declaration order, used to generate bindings and docs."]
            pub const OPTION_SPECS: &'static [::typstyle_core::option::OptionSpec] = &[
                #(#spec_entries),*
            ];

            #[doc = "Applies explicit overrides and validates the result."]
            pub fn apply(
                self,
                overrides: #partial_name,
            ) -> Result<Self, ::typstyle_core::option::OptionsError> {
                let mut this = self;
                #(#apply_fields)*
                this.validate()?;
                Ok(this)
            }

            #[doc = "Validates cross-field constraints. No constraints are currently declared."]
            pub fn validate(&self) -> Result<(), ::typstyle_core::option::OptionsError> {
                Ok(())
            }
        }

        #(#parser_fns)*
    })
}

struct MacroArgs {
    partial: Option<Ident>,
    serde_feature: String,
    clap_feature: String,
}

impl MacroArgs {
    fn parse(tokens: TokenStream2) -> syn::Result<Self> {
        let mut args = Self {
            partial: None,
            serde_feature: "serde".to_owned(),
            clap_feature: "clap".to_owned(),
        };
        if tokens.is_empty() {
            return Ok(args);
        }
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(tokens)?;
        for meta in metas {
            let Meta::NameValue(value) = meta else {
                return Err(syn::Error::new_spanned(
                    meta,
                    "expected `name = value` in `#[options]`",
                ));
            };
            let string = string_value(&value.value)?;
            if value.path.is_ident("partial") {
                args.partial = Some(syn::parse_str(&string)?);
            } else if value.path.is_ident("serde_feature") {
                args.serde_feature = string;
            } else if value.path.is_ident("clap_feature") {
                args.clap_feature = string;
            } else {
                return Err(syn::Error::new_spanned(
                    value.path,
                    "unknown `#[options]` argument; expected `partial`, `serde_feature`, or `clap_feature`",
                ));
            }
        }
        Ok(args)
    }
}

struct FieldSpec {
    ident: Ident,
    ty: Type,
    key: String,
    label: String,
    doc: String,
    default: Expr,
    default_display: Option<String>,
    cli_attrs: Vec<Meta>,
    kind: OptionKind,
    type_name: String,
    group: String,
    widget: Option<String>,
    order: usize,
}

#[derive(Clone, Copy)]
enum OptionKind {
    Boolean,
    Integer,
    Float,
    String,
    Enum,
}

impl FieldSpec {
    fn parse(field: &Field, order: usize) -> syn::Result<Self> {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(field, "options must be named fields"))?;
        let doc = doc_comment(&field.attrs);
        let (kind, type_name) = classify(&field.ty);
        let mut default = None;
        let mut default_display = None;
        let mut cli_attrs = Vec::new();
        let mut group = None;
        let mut widget = None;
        let mut label = None;

        for attr in &field.attrs {
            if attr.path().is_ident("option") {
                for meta in nested_metas(attr)? {
                    match &meta {
                        Meta::NameValue(value) if value.path.is_ident("default") => {
                            default = Some(value.value.clone());
                        }
                        Meta::NameValue(value) if value.path.is_ident("default_display") => {
                            default_display = Some(string_value(&value.value)?);
                        }
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "unknown `#[option]` argument; expected `default` or `default_display`",
                            ));
                        }
                    }
                }
            } else if attr.path().is_ident("cli") {
                cli_attrs.extend(nested_metas(attr)?);
            } else if attr.path().is_ident("ui") {
                for meta in nested_metas(attr)? {
                    match &meta {
                        Meta::NameValue(value) if value.path.is_ident("group") => {
                            group = Some(string_value(&value.value)?);
                        }
                        Meta::NameValue(value) if value.path.is_ident("widget") => {
                            widget = Some(string_value(&value.value)?);
                        }
                        Meta::NameValue(value) if value.path.is_ident("label") => {
                            label = Some(string_value(&value.value)?);
                        }
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "unknown `#[ui]` argument; expected `group`, `widget`, or `label`",
                            ));
                        }
                    }
                }
            }
        }

        let default = default
            .ok_or_else(|| syn::Error::new_spanned(field, "missing `#[option(default = ...)]`"))?;
        let key = ident.to_string();
        let label = label.unwrap_or_else(|| humanize(&key));
        let group = group.unwrap_or_else(|| "General".to_owned());
        let default_display = default_display.or_else(|| literal_display(&default));

        Ok(Self {
            ident,
            ty: field.ty.clone(),
            key,
            label,
            doc,
            default,
            default_display,
            cli_attrs,
            kind,
            type_name,
            group,
            widget,
            order,
        })
    }

    fn metadata(&self) -> TokenStream2 {
        let key = &self.key;
        let label = &self.label;
        let description = &self.doc;
        let kind = match self.kind {
            OptionKind::Boolean => quote!(::typstyle_core::option::OptionKind::Boolean),
            OptionKind::Integer => quote!(::typstyle_core::option::OptionKind::Integer),
            OptionKind::Float => quote!(::typstyle_core::option::OptionKind::Float),
            OptionKind::String => quote!(::typstyle_core::option::OptionKind::String),
            OptionKind::Enum => quote!(::typstyle_core::option::OptionKind::Enum),
        };
        let choices = if matches!(self.kind, OptionKind::Enum) {
            let ty = &self.ty;
            quote!(<#ty as ::typstyle_core::option::OptionEnum>::CHOICES)
        } else {
            quote!(&[])
        };
        let type_name = &self.type_name;
        let group = &self.group;
        let widget = match &self.widget {
            Some(widget) => quote!(Some(#widget)),
            None => quote!(None),
        };
        let order = self.order;
        quote! {
            ::typstyle_core::option::OptionSpec {
                key: #key,
                label: #label,
                description: #description,
                kind: #kind,
                choices: #choices,
                group: #group,
                widget: #widget,
                type_name: #type_name,
                order: #order,
            }
        }
    }

    fn clap_parser(&self, clap_feature: &str) -> Option<TokenStream2> {
        if !matches!(self.kind, OptionKind::Enum)
            || self
                .cli_attrs
                .iter()
                .any(|meta| meta.path().is_ident("value_parser"))
        {
            return None;
        }
        let name = format_ident!("value_parser_{}", self.ident);
        let ty = &self.ty;
        Some(quote! {
            #[cfg(feature = #clap_feature)]
            fn #name() -> impl ::clap::builder::TypedValueParser<Value = #ty> {
                use ::clap::builder::TypedValueParser as _;
                ::clap::builder::PossibleValuesParser::new(
                    <#ty as ::typstyle_core::option::OptionEnum>::CHOICES,
                )
                .map(|value| {
                    <#ty as ::typstyle_core::option::OptionEnum>::from_choice(&value)
                        .expect("possible values come from OptionEnum::CHOICES")
                })
            }
        })
    }
}

impl FieldSpec {
    fn cli_attr_tokens(&self, clap_feature: &str) -> TokenStream2 {
        let long_default = self.key.replace('_', "-");
        let has_long = self
            .cli_attrs
            .iter()
            .any(|meta| meta.path().is_ident("long"));
        let has_global = self
            .cli_attrs
            .iter()
            .any(|meta| meta.path().is_ident("global"));
        let has_help = self
            .cli_attrs
            .iter()
            .any(|meta| meta.path().is_ident("help"));
        let has_value_parser = self
            .cli_attrs
            .iter()
            .any(|meta| meta.path().is_ident("value_parser"));
        let has_value_name = self
            .cli_attrs
            .iter()
            .any(|meta| meta.path().is_ident("value_name"));

        let mut extras = self
            .cli_attrs
            .iter()
            .map(ToTokens::to_token_stream)
            .collect::<Vec<_>>();
        if !has_long {
            extras.push(quote!(long = #long_default));
        }
        if !has_global {
            extras.push(quote!(global = true));
        }
        if !has_help {
            let help = help_text(&self.doc, self.default_display.as_deref());
            extras.push(quote!(help = #help));
        }
        if !has_value_parser && matches!(self.kind, OptionKind::Enum) {
            let parser = format_ident!("value_parser_{}", self.ident);
            extras.push(quote!(value_parser = #parser()));
        }
        if !has_value_name && matches!(self.kind, OptionKind::Boolean) {
            extras.push(quote!(value_name = "BOOL"));
        }
        quote! {
            #[cfg_attr(feature = #clap_feature, arg(#(#extras),*))]
        }
    }
}

fn classify(ty: &Type) -> (OptionKind, String) {
    let Type::Path(path) = ty else {
        return (OptionKind::String, String::new());
    };
    let Some(segment) = path.path.segments.last() else {
        return (OptionKind::String, String::new());
    };
    let name = segment.ident.to_string();
    match name.as_str() {
        "bool" => (OptionKind::Boolean, String::new()),
        "usize" | "u8" | "u16" | "u32" | "u64" | "isize" | "i8" | "i16" | "i32" | "i64" => {
            (OptionKind::Integer, String::new())
        }
        "f32" | "f64" => (OptionKind::Float, String::new()),
        "String" | "str" | "PathBuf" => (OptionKind::String, String::new()),
        _ => (OptionKind::Enum, name),
    }
}

fn nested_metas(attr: &Attribute) -> syn::Result<Vec<Meta>> {
    let list = match &attr.meta {
        Meta::List(list) => list,
        _ => {
            return Err(syn::Error::new_spanned(
                attr,
                "expected an attribute list, for example `#[option(default = 1)]`",
            ));
        }
    };
    Ok(list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
        .into_iter()
        .collect())
}

fn string_value(expr: &Expr) -> syn::Result<String> {
    if let Expr::Lit(lit) = expr
        && let Lit::Str(value) = &lit.lit
    {
        return Ok(value.value());
    }
    Err(syn::Error::new_spanned(expr, "expected a string literal"))
}

fn literal_display(expr: &Expr) -> Option<String> {
    let Expr::Lit(lit) = expr else {
        return None;
    };
    match &lit.lit {
        Lit::Int(value) => Some(value.base10_digits().to_owned()),
        Lit::Bool(value) => Some(value.value.to_string()),
        Lit::Str(value) => Some(value.value()),
        _ => None,
    }
}

fn help_text(doc: &str, default: Option<&str>) -> String {
    let mut text = doc.trim().to_owned();
    if let Some(default) = default {
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(&format!("[default: {default}]"));
    }
    text
}

fn doc_comment(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs {
        if let Meta::NameValue(value) = &attr.meta
            && value.path.is_ident("doc")
            && let Expr::Lit(lit) = &value.value
            && let Lit::Str(text) = &lit.lit
        {
            lines.push(text.value().trim().to_owned());
        }
    }
    lines.join(" ")
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

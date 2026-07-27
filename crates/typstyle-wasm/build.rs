use std::{env, fs, path::Path};

use syn::{Attribute, FieldsNamed, Item, ItemStruct, Lit, Meta, Type, parse::Parser};

fn main() {
    // Define the path to the Config struct in the typstyle-core crate.
    // Adjust this relative path if your directory structure is different.
    let core_config_path = Path::new("../typstyle-core/src/config.rs");

    // Tell Cargo to rerun this build script if the core config file changes.
    println!("cargo:rerun-if-changed={}", core_config_path.display());

    let core_config_content = fs::read_to_string(core_config_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", core_config_path.display(), e));

    let ast = syn::parse_file(&core_config_content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", core_config_path.display(), e));

    let config_struct_option = find_config_struct(&ast);

    let ts_interface_fields = if let Some(config_struct) = config_struct_option {
        generate_ts_fields_for_config_struct(&ast, config_struct)
    } else {
        eprintln!(
            "cargo:warning=Config struct not found in {}. Proceeding with an empty TypeScript interface.",
            core_config_path.display()
        );
        String::new() // Default to empty if Config struct is not found
    };

    let ts_interface = format!("export interface Config {{\n{ts_interface_fields}}}");

    let out_dir = env::var_os("OUT_DIR")
        .expect("OUT_DIR environment variable not set. This script should be run by Cargo.");
    let dest_path = Path::new(&out_dir).join("generated_config_interface.ts");

    fs::write(&dest_path, ts_interface).unwrap_or_else(|e| {
        panic!(
            "Failed to write generated TypeScript interface to {}: {}",
            dest_path.display(),
            e
        )
    });

    // Tell Cargo to rerun this build script if the build script itself changes.
    println!("cargo:rerun-if-changed=build.rs");
}

/// Finds a struct definition by name (e.g., "Config") within a parsed Rust file AST.
fn find_config_struct(ast: &syn::File) -> Option<&ItemStruct> {
    for item in &ast.items {
        if let Item::Struct(item_struct @ ItemStruct { ident, .. }) = item
            && ident == "Config"
        {
            // Ensure this matches the target struct name
            return Some(item_struct);
        }
    }
    None
}

/// Generates TypeScript interface field definitions from a Rust struct's fields.
/// Includes JSDoc comments extracted from Rust doc comments.
///
/// Assume `Config` is at the top level.
fn generate_ts_fields_for_config_struct(ast: &syn::File, config_struct: &ItemStruct) -> String {
    let mut ts_fields_string = String::new();

    // Optional: Extract and add doc comments for the interface itself
    // let interface_doc = extract_doc_comments(&config_struct.attrs);
    // ts_fields_string.push_str(&interface_doc);

    if let syn::Fields::Named(FieldsNamed { named, .. }) = &config_struct.fields {
        for field in named {
            let field_doc_comments = extract_doc_comments(&field.attrs);
            if let (Some(field_ident), Some(ts_type)) =
                (&field.ident, rust_type_to_ts_type(ast, &field.ty))
            {
                ts_fields_string.push_str(&field_doc_comments);
                ts_fields_string.push_str(&format!("    {field_ident}: {ts_type},\n"));
            } else {
                eprintln!(
                    "cargo:warning=Could not map type for field {:?} in Config struct. It will be omitted from the TypeScript interface.",
                    field.ident
                );
            }
        }
    }
    ts_fields_string
}

/// Converts a Rust type identifier to its corresponding TypeScript type string.
fn rust_type_to_ts_type(ast: &syn::File, ty: &Type) -> Option<String> {
    if let Type::Path(type_path) = ty
        && type_path.qself.is_none()
    {
        let last_segment = type_path.path.segments.last()?;
        let ident_str = last_segment.ident.to_string();
        // Map common Rust types to TypeScript types.
        return match ident_str.as_str() {
            "usize" | "u8" | "u16" | "u32" | "u64" | "isize" | "i8" | "i16" | "i32" | "i64"
            | "f32" | "f64" => Some("number".to_string()),
            "bool" => Some("boolean".to_string()),
            "String" => Some("string".to_string()),
            _ => ast.items.iter().find_map(|item| {
                let Item::Enum(item_enum) = item else {
                    return None;
                };
                if item_enum.ident != ident_str
                    || item_enum
                        .variants
                        .iter()
                        .any(|variant| !matches!(variant.fields, syn::Fields::Unit))
                {
                    return None;
                }
                let rename_all = extract_serde_value(&item_enum.attrs, "rename_all");
                Some(
                    item_enum
                        .variants
                        .iter()
                        .map(|variant| {
                            let variant_name = variant.ident.to_string();
                            let name = extract_serde_value(&variant.attrs, "rename")
                                .or_else(|| {
                                    rename_all
                                        .as_deref()
                                        .map(|rule| apply_serde_rename_all(&variant_name, rule))
                                })
                                .unwrap_or(variant_name);
                            format!("\"{name}\"")
                        })
                        .collect::<Vec<_>>()
                        .join(" | "),
                )
            }),
        };
    }
    None
}

fn extract_serde_value(attrs: &[Attribute], key: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.path().is_ident("serde") {
            return extract_serde_value_from_meta(&attr.meta, key);
        }
        if !attr.path().is_ident("cfg_attr") {
            return None;
        }
        let Meta::List(list) = &attr.meta else {
            return None;
        };
        let nested = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .ok()?;
        nested
            .iter()
            .find_map(|meta| extract_serde_value_from_meta(meta, key))
    })
}

fn extract_serde_value_from_meta(meta: &Meta, key: &str) -> Option<String> {
    let Meta::List(list) = meta else {
        return None;
    };
    if !list.path.is_ident("serde") {
        return None;
    }
    let nested = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .ok()?;
    nested.iter().find_map(|meta| {
        let Meta::NameValue(name_value) = meta else {
            return None;
        };
        if !name_value.path.is_ident(key) {
            return None;
        }
        let syn::Expr::Lit(expr) = &name_value.value else {
            return None;
        };
        let Lit::Str(value) = &expr.lit else {
            return None;
        };
        Some(value.value())
    })
}

/// Applies Serde's enum-variant `rename_all` rules.
fn apply_serde_rename_all(variant: &str, rule: &str) -> String {
    let snake_case = || {
        let mut renamed = String::new();
        for (index, ch) in variant.char_indices() {
            if index > 0 && ch.is_uppercase() {
                renamed.push('_');
            }
            renamed.push(ch.to_ascii_lowercase());
        }
        renamed
    };

    match rule {
        "lowercase" => variant.to_ascii_lowercase(),
        "UPPERCASE" => variant.to_ascii_uppercase(),
        "PascalCase" => variant.to_owned(),
        "camelCase" => {
            let mut chars = variant.chars();
            chars
                .next()
                .map(|first| first.to_ascii_lowercase().to_string() + chars.as_str())
                .unwrap_or_default()
        }
        "snake_case" => snake_case(),
        "SCREAMING_SNAKE_CASE" => snake_case().to_ascii_uppercase(),
        "kebab-case" => snake_case().replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => snake_case().replace('_', "-").to_ascii_uppercase(),
        _ => panic!("unsupported serde rename_all rule: {rule}"),
    }
}

/// Extracts Rust doc comments (#[doc = "..."]) from attributes and formats them as JSDoc.
fn extract_doc_comments(attrs: &[Attribute]) -> String {
    let mut doc_lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &nv.value
            && let Lit::Str(lit_str) = &expr_lit.lit
        {
            doc_lines.push(lit_str.value().trim().to_string());
        }
    }

    if doc_lines.is_empty() {
        String::new()
    } else {
        let mut comment = "    /**\n".to_string();
        for line in doc_lines {
            comment.push_str(&format!("     * {line}\n"));
        }
        comment.push_str("     */\n");
        comment
    }
}

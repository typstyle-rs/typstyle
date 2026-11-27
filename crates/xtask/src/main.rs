//! Repository automation tasks.

mod bindings;
mod model;
mod render;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use self::bindings::{
    render_defaults, render_markdown, render_schema, render_types, render_ui_schema,
};
use self::model::Model;
use self::render::{render_cli, render_core};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("generate-options") => {
            let check = args.next().as_deref() == Some("--check");
            if args.next().is_some() {
                bail!("usage: cargo xtask generate-options [--check]");
            }
            generate_options(check)
        }
        _ => bail!("usage: cargo xtask generate-options [--check]"),
    }
}

fn generate_options(check: bool) -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("xtask manifest has no repository root")?
        .to_owned();

    let schema_path = root.join("options.toml");
    let schema = fs::read_to_string(&schema_path)
        .with_context(|| format!("reading {}", schema_path.display()))?;
    let model =
        Model::parse(&schema).with_context(|| format!("parsing {}", schema_path.display()))?;

    let generated = root.join("packages/options/generated");
    let outputs = [
        (
            root.join("crates/typstyle-core/src/config/generated.rs"),
            render_core(&model)?,
        ),
        (
            root.join("crates/typstyle/src/generated_args.rs"),
            render_cli(&model)?,
        ),
        (generated.join("types.ts"), render_types(&model)),
        (generated.join("defaults.json"), render_defaults(&model)),
        (generated.join("schema.json"), render_schema(&model)),
        (generated.join("ui-schema.json"), render_ui_schema(&model)),
        (generated.join("options.md"), render_markdown(&model)),
    ];

    let mut stale = Vec::new();
    for (path, content) in outputs {
        if check {
            match fs::read_to_string(&path) {
                Ok(existing) if existing == content => {}
                _ => stale.push(path),
            }
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
        }
    }

    if !stale.is_empty() {
        for path in stale {
            eprintln!("stale generated option artifact: {}", path.display());
        }
        bail!("run `cargo xtask generate-options` and commit the result");
    }
    if !check {
        println!("generated formatter options");
    }
    Ok(())
}

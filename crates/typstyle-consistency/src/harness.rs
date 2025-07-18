use std::{
    borrow::Cow,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rustc_hash::{FxHashMap, FxHashSet};
use tinymist_world::{
    config::CompileOpts, EntryOpts, EntryReader, ShadowApi, TaskInputs, TypstSystemUniverse,
    TypstSystemWorld,
};
use typst::{
    foundations::Bytes,
    syntax::{FileId, Source},
    World,
};
use walkdir::WalkDir;

use crate::{compare_docs, compile_world, ErrorSink, FormattedWorld, SourceMap};

pub struct FormattedSources {
    /// The name of the formatter configuration, used for debugging.
    pub name: String,
    /// Map of file IDs to their formatted sources.
    pub sources: SourceMap,
}

pub struct FormatterHarness {
    /// The name of the test case, used for debugging.
    name: String,
    /// The root directory of the project in the physical filesystem.
    project_root: PathBuf,
    /// The set of file IDs that refer to formattable sources.
    formattable: FxHashSet<FileId>,
    /// The universe used for compiling.
    verse: TypstSystemUniverse,
}

impl FormatterHarness {
    fn vroot() -> &'static Path {
        Path::new(".")
    }

    pub fn new(name: String, project_root: PathBuf) -> Result<Self> {
        Ok(Self {
            name,
            project_root,
            formattable: Default::default(),
            verse: TypstSystemUniverse::new(CompileOpts {
                entry: EntryOpts::new_workspace(Self::vroot().to_path_buf()),
                with_embedded_fonts: typst_assets::fonts().map(Cow::Borrowed).collect(),
                ..Default::default()
            })?,
        })
    }

    /// Add all files in the given directory to the workspace.
    pub fn add_all_files(
        &mut self,
        source_dir: &Path,
        blacklist: &HashSet<String>,
    ) -> Result<&mut Self> {
        // map all files within the testcase_dir
        let walk = WalkDir::new(source_dir).into_iter().filter_entry(|e| {
            !e.file_name()
                .to_str()
                .is_some_and(|name| name.starts_with('.'))
        });
        for entry in walk {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let rel_path = path.strip_prefix(&self.project_root)?;

            if path.extension() == Some("typ".as_ref())
                && !is_blacklisted(path, source_dir, blacklist)
            {
                let content =
                    Bytes::from_string(strip_trailing_whitespace(&fs::read_to_string(path)?));
                self.add_fmt_file(rel_path, content)?;
            } else {
                let content = Bytes::new(fs::read(path)?);
                self.add_raw_file(rel_path, content)?;
            }
        }

        Ok(self)
    }

    /// Add a source file to the workspace, which `#include`s all source files in the given directory.
    pub fn add_all_files_in_one(
        &mut self,
        one_path: &Path,
        source_dir: &Path,
    ) -> Result<&mut Self> {
        let mut entry_content = String::new();
        // map all files within the testcase_dir
        for entry in WalkDir::new(source_dir) {
            let entry = entry?;
            if !(entry.file_type().is_file() && entry.path().extension() == Some("typ".as_ref())) {
                continue;
            }

            let rel_path = entry.path().strip_prefix(&self.project_root)?;
            let include_path = rel_path.to_str().unwrap().replace('\\', "/");
            entry_content.push_str(&format!("#include \"{include_path}\"\n"));
        }
        self.add_raw_file(one_path, Bytes::from_string(entry_content))?;

        Ok(self)
    }

    /// Add a single source file to the workspace.
    pub fn add_source_file(&mut self, path: &Path, content: &str) -> Result<&mut Self> {
        self.add_fmt_file(path, Bytes::from_string(strip_trailing_whitespace(content)))?;

        Ok(self)
    }

    /// Add a formattable file to the workspace.
    fn add_fmt_file(&mut self, path: &Path, content: Bytes) -> Result<()> {
        let vpath = &Self::vroot().join(path);
        self.verse.map_shadow(vpath, content)?;
        self.formattable
            .insert(self.verse.id_for_path(vpath).unwrap());

        Ok(())
    }

    /// Add a raw file to the workspace, which should keep unchanged.
    fn add_raw_file(&mut self, path: &Path, content: Bytes) -> Result<()> {
        let vpath = &Self::vroot().join(path);
        self.verse.map_shadow(vpath, content)?;

        Ok(())
    }

    pub fn snapshot(&self) -> TypstSystemWorld {
        self.verse.snapshot()
    }

    pub fn format<'a>(
        &'a mut self,
        world: &'a TypstSystemWorld,
        formatter: impl Fn(Source) -> Result<String>,
        err_sink: &mut ErrorSink,
    ) -> Result<SourceMap> {
        let mut formatted = FxHashMap::default();

        for &fid in &self.formattable {
            let source = world.source(fid)?;
            let formatted_str = formatter(source);
            match formatted_str {
                Ok(res) => {
                    formatted.insert(fid, Source::new(fid, res));
                }
                Err(err) => {
                    err_sink.push(format!(
                        "failed to format file at `{}`: {:#}",
                        self.verse.path_for_id(fid)?.as_path().display(),
                        err
                    ));
                }
            }
        }

        Ok(formatted)
    }

    pub fn compile_and_compare<'a>(
        &mut self,
        formatted: impl Iterator<Item = &'a FormattedSources>,
        entry_path: &Path,
        require_compile: bool,
        err_sink: &mut ErrorSink,
    ) -> Result<()> {
        let mut sub_sink =
            ErrorSink::new(format!("compiling entry point `{}`", entry_path.display()));

        let base_world = self.verse.snapshot_with(Some(TaskInputs {
            entry: Some(self.verse.entry_state().select_in_workspace(entry_path)),
            ..Default::default()
        }));

        let name = if self.name.is_empty() {
            entry_path.display().to_string()
        } else {
            format!("{} - {}", self.name, entry_path.display())
        };
        let base_result = compile_world(format!("{name} - original"), &base_world)?;

        for sources in formatted {
            let world = FormattedWorld {
                base: &base_world,
                formatted: sources.sources.clone(),
            };
            let name = format!("{} - {}", name, sources.name);
            let fmt_result = compile_world(name.clone(), &world)?;

            compare_docs(&base_result, &fmt_result, require_compile, &mut sub_sink)?;
        }

        sub_sink.sink_to(err_sink);
        Ok(())
    }
}

fn is_blacklisted(path: &Path, source_dir: &Path, blacklist: &HashSet<String>) -> bool {
    // Get the relative path starting at source_dir.
    if let Ok(rel_path) = path.strip_prefix(source_dir) {
        // Return true if any component's name is in the blacklist.
        return rel_path
            .components()
            .filter_map(|comp| {
                if let std::path::Component::Normal(os_str) = comp {
                    os_str.to_str()
                } else {
                    None
                }
            })
            .any(|name| blacklist.contains(name));
    }
    false
}

fn strip_trailing_whitespace(s: &str) -> String {
    if s.is_empty() {
        return "".to_string();
    }
    let mut res = String::with_capacity(s.len());
    for line in s.lines() {
        res.push_str(line.trim_end());
        res.push('\n');
    }
    res
}

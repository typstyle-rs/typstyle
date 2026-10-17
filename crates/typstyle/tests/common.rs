//! Shared helpers for the CLI integration tests: a fake project workspace and
//! snapshot utilities.

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use insta_cmd::get_cargo_bin;
use tempfile::TempDir;

/// Regular expression filters applied to CLI snapshot output before comparison.
///
/// They normalize nondeterministic content such as durations, temporary paths,
/// path separators, and the `.exe` suffix.
#[allow(dead_code)]
pub const INSTA_FILTERS: &[(&str, &str)] = &[
    (r"(?m)^(.+) in [\w.]+$", "$1 in [DURATION]"),
    (
        r"(\b[A-Z]:)?[\\/].*?[\\/]\.typstyle-tests---[^\\/]+[\\/]",
        "[TEMP_PATH]/",
    ),
    (r"\\\\?([\w\d.])", "/$1"),
    (r"typstyle.exe", "typstyle"),
];

/// A temporary directory that acts as the project of a CLI test run.
pub struct Workspace {
    /// The temporary directory holding the project, removed when dropped.
    #[allow(unused)]
    tempdir: TempDir,
    /// The project directory inside the temporary directory.
    project_dir: PathBuf,
    /// Records file last modified time.
    tracked: HashMap<PathBuf, SystemTime>,
}

#[allow(dead_code)]
impl Workspace {
    /// Creates a workspace with a fresh, empty project directory.
    pub fn new() -> Workspace {
        let tempdir = marked_tempdir();
        let project_dir = tempdir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        Workspace {
            tempdir,
            project_dir,
            tracked: Default::default(),
        }
    }

    /// Returns the path of the project directory.
    pub fn project_path(&self) -> &Path {
        &self.project_dir
    }

    /// Creates a [`Command`] running the given program in the project directory.
    pub fn cmd(&self, cmd: impl AsRef<OsStr>) -> Command {
        let mut rv = Command::new(cmd);
        rv.current_dir(self.project_path());
        rv
    }

    /// Creates a [`Command`] running the built `typstyle` binary in the project
    /// directory, with the Git environment isolated from the developer's
    /// configuration.
    ///
    /// `HOME`, `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`, and `GIT_CONFIG_SYSTEM`
    /// point inside the project directory, so tests can opt into global ignore
    /// rules by writing `gitconfig-global` in the project.
    pub fn cli(&self) -> Command {
        let mut cmd = self.cmd(get_cargo_bin("typstyle"));
        cmd.env("HOME", self.project_path().join("home"))
            .env("XDG_CONFIG_HOME", self.project_path().join("xdg"))
            .env(
                "GIT_CONFIG_GLOBAL",
                self.project_path().join("gitconfig-global"),
            )
            .env(
                "GIT_CONFIG_SYSTEM",
                self.project_path().join("gitconfig-system"),
            )
            .env("GIT_CONFIG_NOSYSTEM", "1");
        cmd
    }

    /// Marks the project as Git repository root.
    pub fn git_init(&self) {
        self.write(".git/HEAD", "ref: refs/heads/main\n");
    }

    /// Writes the contents to the project-relative path, creating parent directories.
    pub fn write(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        self.write_impl(path.as_ref(), contents.as_ref());
    }

    /// Write file to project and track its modified time.
    pub fn write_tracked(&mut self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        let p = self.write_impl(path.as_ref(), contents.as_ref());
        if let Ok(time) = p.metadata().unwrap().modified() {
            self.tracked.insert(p, time);
        }
    }

    /// Writes the contents and returns the full path of the written file.
    fn write_impl(&self, path: &Path, contents: &[u8]) -> PathBuf {
        let p = self.project_path().join(path);
        fs::create_dir_all(p.parent().unwrap()).ok();
        fs::write(&p, contents).unwrap();
        p
    }

    /// Reads the file at the project-relative path as a string.
    pub fn read_string(&self, path: impl AsRef<Path>) -> String {
        let p = self.project_path().join(path.as_ref());
        fs::read_to_string(p).unwrap()
    }

    /// Returns whether the tracked file's last modified time has changed.
    pub fn is_modified(&self, path: impl AsRef<Path>) -> bool {
        !self.is_unmodified(path)
    }

    /// Returns whether the tracked file still has its recorded last modified time.
    ///
    /// Files never written through this workspace are reported as modified.
    pub fn is_unmodified(&self, path: impl AsRef<Path>) -> bool {
        let p = self.project_path().join(path.as_ref());
        if let Ok(time) = p.metadata().unwrap().modified() {
            self.tracked
                .get(&p)
                .is_some_and(|old_time| *old_time == time)
        } else {
            true
        }
    }

    /// Returns whether every tracked file is still unmodified.
    pub fn all_unmodified(&self) -> bool {
        self.tracked
            .iter()
            .all(|(p, old_time)| match p.metadata().unwrap().modified() {
                Ok(time) => *old_time == time,
                Err(_) => true,
            })
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a temp dir whose name is masked by [`INSTA_FILTERS`] in snapshots.
fn marked_tempdir() -> TempDir {
    TempDir::with_prefix(".typstyle-tests---").unwrap()
}

#[allow(unused)]
/// Asserts a snapshot of the given command's output, with [`INSTA_FILTERS`] applied.
macro_rules! typstyle_cmd_snapshot {
    ($cmd:expr, @$snapshot:literal) => {
        let mut settings = insta::Settings::clone_current();
        for (matcher, replacement) in $crate::common::INSTA_FILTERS {
            settings.add_filter(matcher, *replacement);
        }
        let _guard = settings.bind_to_scope();
        insta_cmd::assert_cmd_snapshot!($cmd, @$snapshot);
    };
}

#[allow(unused_imports)]
pub(crate) use typstyle_cmd_snapshot;

use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use thiserror::Error;

use crate::{Config, ConfigError, GenerationError, refresh_generated_fixtures};

#[derive(Clone, Debug)]
pub struct TestOptions {
    pub project_root: PathBuf,
    pub filter: Option<String>,
}

impl TestOptions {
    #[must_use]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            filter: None,
        }
    }

    #[must_use]
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

/// Regenerates the private preview target and runs its explicit inline Rust tests.
///
/// # Errors
/// Returns an error when configuration, generation, process startup, or Cargo tests fail.
pub fn run_tests(options: &TestOptions) -> Result<(), TestError> {
    let project_root = canonical_project_root(&options.project_root)?;
    let config = Config::load(&project_root)?;
    let generated = refresh_generated_fixtures(&project_root, &config)?;
    println!(
        "Testing {} Hblank fixture files with Cargo",
        generated.fixture_files.len()
    );

    let manifest = project_root.join(".hblank/Cargo.toml");
    let target = project_root.join(".hblank/target");
    let mut command = Command::new("cargo");
    command
        .arg("test")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target)
        .stdin(Stdio::null());
    if let Some(filter) = &options.filter {
        command.arg(filter);
    }
    let status = command.status().map_err(TestError::Process)?;
    if status.success() {
        Ok(())
    } else {
        Err(TestError::TestsFailed(status))
    }
}

fn canonical_project_root(path: &Path) -> Result<PathBuf, TestError> {
    path.canonicalize()
        .map_err(|source| TestError::ProjectRoot {
            path: path.to_path_buf(),
            source,
        })
}

#[derive(Debug, Error)]
pub enum TestError {
    #[error("could not resolve Hblank project root {path}: {source}")]
    ProjectRoot {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Generation(#[from] GenerationError),
    #[error("could not run Hblank tests: {0}")]
    Process(std::io::Error),
    #[error("Hblank tests exited unsuccessfully: {0}")]
    TestsFailed(ExitStatus),
}

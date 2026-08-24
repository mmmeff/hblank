use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use thiserror::Error;

use crate::{Config, ConfigError, GenerationError, refresh_generated_fixtures};

#[derive(Clone, Debug)]
pub struct CatalogOptions {
    pub project_root: PathBuf,
}

impl CatalogOptions {
    #[must_use]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }
}

/// Builds the private preview and prints its registered component catalog.
///
/// # Errors
/// Returns an error when config, generation, build, manifest, or preview execution fails.
pub fn run_list(options: &CatalogOptions) -> Result<(), CatalogError> {
    let project_root = canonical_project_root(&options.project_root)?;
    let config = Config::load(&project_root)?;
    refresh_generated_fixtures(&project_root, &config)?;
    build_preview(&project_root)?;
    let output = catalog_output(&project_root)?;
    print!("{output}");
    Ok(())
}

pub(crate) fn fixture_ids(project_root: &Path) -> Result<Vec<String>, CatalogError> {
    let output = catalog_output(project_root)?;
    Ok(output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            (fields.next() == Some("fixture"))
                .then(|| fields.next().map(str::to_owned))
                .flatten()
        })
        .collect())
}

pub(crate) fn build_preview(project_root: &Path) -> Result<(), CatalogError> {
    let manifest = project_root.join(".hblank/Cargo.toml");
    let target = project_root.join(".hblank/target");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target)
        .stdin(Stdio::null())
        .status()
        .map_err(CatalogError::Process)?;
    if status.success() {
        Ok(())
    } else {
        Err(CatalogError::BuildFailed(status))
    }
}

pub(crate) fn preview_binary(project_root: &Path) -> Result<PathBuf, CatalogError> {
    let package_name = preview_package_name(project_root)?;
    let mut binary = project_root.join(".hblank/target/debug").join(package_name);
    if cfg!(windows) {
        binary.set_extension("exe");
    }
    Ok(binary)
}

fn catalog_output(project_root: &Path) -> Result<String, CatalogError> {
    let binary = preview_binary(project_root)?;
    let output = Command::new(&binary)
        .env("HBLANK_PROJECT_ROOT", project_root)
        .env("HBLANK_LIST_CATALOG", "1")
        .stdin(Stdio::null())
        .output()
        .map_err(CatalogError::Process)?;
    if !output.status.success() {
        return Err(CatalogError::ListFailed(output.status));
    }
    String::from_utf8(output.stdout).map_err(CatalogError::NonUtf8)
}

fn canonical_project_root(path: &Path) -> Result<PathBuf, CatalogError> {
    path.canonicalize()
        .map_err(|source| CatalogError::ProjectRoot {
            path: path.to_path_buf(),
            source,
        })
}

fn preview_package_name(project_root: &Path) -> Result<String, CatalogError> {
    let path = project_root.join(".hblank/Cargo.toml");
    let source = fs::read_to_string(&path).map_err(|source| CatalogError::ReadManifest {
        path: path.clone(),
        source,
    })?;
    let manifest =
        toml::from_str::<toml::Value>(&source).map_err(|source| CatalogError::ParseManifest {
            path: path.clone(),
            source,
        })?;
    manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or(CatalogError::MissingPackageName(path))
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("could not resolve Hblank project root {path}: {source}")]
    ProjectRoot {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Generation(#[from] GenerationError),
    #[error("could not run Hblank catalog command: {0}")]
    Process(std::io::Error),
    #[error("preview build exited unsuccessfully: {0}")]
    BuildFailed(ExitStatus),
    #[error("catalog listing exited unsuccessfully: {0}")]
    ListFailed(ExitStatus),
    #[error("catalog listing was not UTF-8: {0}")]
    NonUtf8(std::string::FromUtf8Error),
    #[error("could not read preview manifest at {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse preview manifest at {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("preview manifest at {0} has no package name")]
    MissingPackageName(PathBuf),
}

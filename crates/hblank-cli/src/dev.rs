use std::{
    ffi::OsStr,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::mpsc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use notify::{RecursiveMode, Watcher};
use thiserror::Error;

use crate::{
    Config, ConfigError, DiscoveredFixtureFile, GenerationError, refresh_generated_fixtures,
};

const DEBOUNCE: Duration = Duration::from_millis(160);
const IDLE_POLL: Duration = Duration::from_millis(250);

fn dev_session_id() -> String {
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{started}", std::process::id())
}

#[derive(Clone, Debug)]
pub struct DevOptions {
    pub project_root: PathBuf,
    pub fixture: Option<PathBuf>,
}

impl DevOptions {
    #[must_use]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            fixture: None,
        }
    }

    /// Requests one discovered fixture source file for the initial preview selection.
    #[must_use]
    pub fn with_fixture(mut self, fixture: impl Into<PathBuf>) -> Self {
        self.fixture = Some(fixture.into());
        self
    }
}

/// Builds, launches, watches, and automatically reloads an initialized Hblank project.
///
/// # Errors
/// Returns an error when configuration, discovery, watching, building, or process control fails.
pub fn run_dev(options: &DevOptions) -> Result<(), DevError> {
    let project_root =
        options
            .project_root
            .canonicalize()
            .map_err(|source| DevError::ProjectRoot {
                path: options.project_root.clone(),
                source,
            })?;
    let mut config = Config::load(&project_root)?;
    let initial = refresh_generated_fixtures(&project_root, &config)?;
    let fixture = options
        .fixture
        .as_deref()
        .map(|path| resolve_initial_fixture(&project_root, path, &initial.fixture_files))
        .transpose()?;
    let mut fingerprint = source_fingerprint(&project_root)?;
    let session_id = dev_session_id();
    println!(
        "Discovered {} Hblank fixture files",
        initial.fixture_files.len()
    );
    if let Some(fixture) = &fixture {
        println!("Opening fixture {}", fixture.display());
    }

    let mut preview =
        PreviewProcess::build_and_start(&project_root, &config, fixture.as_deref(), &session_id)?;
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(sender).map_err(DevError::Watcher)?;
    watcher
        .watch(&project_root, RecursiveMode::Recursive)
        .map_err(DevError::Watcher)?;

    loop {
        match receiver.recv_timeout(IDLE_POLL) {
            Ok(event) => {
                let mut relevant = event.map_err(DevError::WatchEvent)?.paths;
                while let Ok(event) = receiver.recv_timeout(DEBOUNCE) {
                    relevant.extend(event.map_err(DevError::WatchEvent)?.paths);
                }
                relevant.retain(|path| is_relevant_change(&project_root, path));
                relevant.sort();
                relevant.dedup();
                if relevant.is_empty() {
                    continue;
                }
                let next_fingerprint = source_fingerprint(&project_root)?;
                if next_fingerprint == fingerprint {
                    continue;
                }
                fingerprint = next_fingerprint;
                if relevant
                    .iter()
                    .any(|path| path == &project_root.join(crate::CONFIG_PATH))
                {
                    match Config::load(&project_root) {
                        Ok(next) => config = next,
                        Err(error) => {
                            eprintln!("Hblank config error; keeping the current preview: {error}");
                            continue;
                        }
                    }
                }
                match rebuild(&project_root, &config, &session_id, &mut preview) {
                    Ok(count) => println!("Reloaded {count} Hblank fixture files"),
                    Err(error) => {
                        eprintln!("Hblank rebuild failed; keeping the current preview: {error}");
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if preview.has_exited()? {
                    println!("Hblank preview closed");
                    return Ok(());
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(DevError::WatcherDisconnected),
        }
    }
}

fn resolve_initial_fixture(
    project_root: &Path,
    requested: &Path,
    fixture_files: &[DiscoveredFixtureFile],
) -> Result<PathBuf, DevError> {
    let path = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        project_root.join(requested)
    };
    let canonical = path
        .canonicalize()
        .map_err(|source| DevError::FixturePath {
            path: path.clone(),
            source,
        })?;
    if fixture_files
        .iter()
        .any(|fixture_file| fixture_file.absolute_path == canonical)
    {
        Ok(canonical)
    } else {
        Err(DevError::FixtureNotDiscovered(canonical))
    }
}

fn rebuild(
    project_root: &Path,
    config: &Config,
    session_id: &str,
    preview: &mut PreviewProcess,
) -> Result<usize, DevError> {
    let generated = refresh_generated_fixtures(project_root, config)?;
    build_preview(project_root)?;
    let replacement = spawn_preview(project_root, config, None, session_id)?;
    preview.replace(replacement)?;
    Ok(generated.fixture_files.len())
}

fn source_fingerprint(project_root: &Path) -> Result<u64, DevError> {
    let mut hasher = DefaultHasher::new();
    for entry in walkdir::WalkDir::new(project_root).sort_by_file_name() {
        let entry = entry.map_err(DevError::FingerprintWalk)?;
        let path = entry.path();
        if !entry.file_type().is_file() || !is_relevant_change(project_root, path) {
            continue;
        }
        path.strip_prefix(project_root)
            .expect("walked source must remain inside the project")
            .hash(&mut hasher);
        fs::read(path)
            .map_err(|source| DevError::FingerprintRead {
                path: path.to_path_buf(),
                source,
            })?
            .hash(&mut hasher);
    }
    Ok(hasher.finish())
}

fn is_relevant_change(project_root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(project_root) else {
        return false;
    };
    if relative.starts_with("target")
        || relative.starts_with(".git")
        || (relative.starts_with(".hblank") && relative != Path::new(crate::CONFIG_PATH))
    {
        return false;
    }
    relative == Path::new(crate::CONFIG_PATH)
        || relative.file_name() == Some(OsStr::new("Cargo.toml"))
        || relative.file_name() == Some(OsStr::new("Cargo.lock"))
        || relative.extension() == Some(OsStr::new("rs"))
}

struct PreviewProcess {
    child: Child,
}

impl PreviewProcess {
    fn build_and_start(
        project_root: &Path,
        config: &Config,
        fixture: Option<&Path>,
        session_id: &str,
    ) -> Result<Self, DevError> {
        build_preview(project_root)?;
        let child = spawn_preview(project_root, config, fixture, session_id)?;
        Ok(Self { child })
    }

    fn replace(&mut self, replacement: Child) -> Result<(), DevError> {
        self.stop()?;
        self.child = replacement;
        Ok(())
    }

    fn has_exited(&mut self) -> Result<bool, DevError> {
        self.child
            .try_wait()
            .map(|status| status.is_some())
            .map_err(DevError::Process)
    }

    fn stop(&mut self) -> Result<(), DevError> {
        if self.child.try_wait().map_err(DevError::Process)?.is_none() {
            self.child.kill().map_err(DevError::Process)?;
            self.child.wait().map_err(DevError::Process)?;
        }
        Ok(())
    }
}

impl Drop for PreviewProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn build_preview(project_root: &Path) -> Result<(), DevError> {
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
        .map_err(DevError::Process)?;
    if status.success() {
        Ok(())
    } else {
        Err(DevError::BuildFailed(status))
    }
}

fn spawn_preview(
    project_root: &Path,
    config: &Config,
    fixture: Option<&Path>,
    session_id: &str,
) -> Result<Child, DevError> {
    let binary_name = preview_package_name(project_root)?;
    let mut binary = project_root.join(".hblank/target/debug").join(binary_name);
    if cfg!(windows) {
        binary.set_extension("exe");
    }
    let mut command = Command::new(&binary);
    command
        .env("HBLANK_PROJECT_ROOT", project_root)
        .env("HBLANK_WINDOW_TITLE", &config.window.title)
        .env("HBLANK_WINDOW_WIDTH", config.window.width.to_string())
        .env("HBLANK_WINDOW_HEIGHT", config.window.height.to_string())
        .env("HBLANK_SESSION_ID", session_id)
        .stdin(Stdio::null());
    if let Some(theme_hook) = &config.theme_hook {
        command.env("HBLANK_THEME_HOOK", theme_hook);
    }
    if let Some(fixture) = fixture {
        command.env("HBLANK_INITIAL_FIXTURE", fixture);
    }
    command
        .spawn()
        .map_err(|source| DevError::Spawn { binary, source })
}

fn preview_package_name(project_root: &Path) -> Result<String, DevError> {
    let path = project_root.join(".hblank/Cargo.toml");
    let source = std::fs::read_to_string(&path).map_err(|source| DevError::ReadManifest {
        path: path.clone(),
        source,
    })?;
    let manifest =
        toml::from_str::<toml::Value>(&source).map_err(|source| DevError::ParseManifest {
            path: path.clone(),
            source,
        })?;
    manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or(DevError::MissingPreviewPackage(path))
}

#[derive(Debug, Error)]
pub enum DevError {
    #[error("could not resolve project root {path}: {source}")]
    ProjectRoot {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Generation(#[from] GenerationError),
    #[error("could not resolve requested fixture path {path}: {source}")]
    FixturePath {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("requested fixture {0} is not matched by the configured fixture file patterns")]
    FixtureNotDiscovered(PathBuf),
    #[error("could not scan watched project sources: {0}")]
    FingerprintWalk(walkdir::Error),
    #[error("could not read watched project source {path}: {source}")]
    FingerprintRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not start filesystem watcher: {0}")]
    Watcher(notify::Error),
    #[error("filesystem watcher failed: {0}")]
    WatchEvent(notify::Error),
    #[error("filesystem watcher disconnected")]
    WatcherDisconnected,
    #[error("preview build exited unsuccessfully: {0}")]
    BuildFailed(ExitStatus),
    #[error("could not manage preview process: {0}")]
    Process(std::io::Error),
    #[error("could not launch preview binary {binary}: {source}")]
    Spawn {
        binary: PathBuf,
        source: std::io::Error,
    },
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
    MissingPreviewPackage(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::{DevError, is_relevant_change, resolve_initial_fixture, source_fingerprint};
    use crate::DiscoveredFixtureFile;
    use std::{fs, path::Path};

    #[test]
    fn filters_generated_and_build_events() {
        let root = Path::new("/project");
        assert!(is_relevant_change(
            root,
            Path::new("/project/src/button.rs")
        ));
        assert!(is_relevant_change(
            root,
            Path::new("/project/.hblank/config.toml")
        ));
        assert!(is_relevant_change(root, Path::new("/project/Cargo.toml")));
        assert!(!is_relevant_change(
            root,
            Path::new("/project/.hblank/Cargo.lock")
        ));
        assert!(!is_relevant_change(
            root,
            Path::new("/project/target/debug/app")
        ));
        assert!(!is_relevant_change(
            root,
            Path::new("/project/.hblank/generated/fixtures.rs")
        ));
        assert!(!is_relevant_change(
            root,
            Path::new("/project/assets/icon.png")
        ));
    }

    #[test]
    fn fingerprint_changes_only_for_watched_inputs() {
        let project = tempfile::tempdir().expect("temporary project should be created");
        let source = project.path().join("src/button.rs");
        fs::create_dir_all(source.parent().expect("source has a parent"))
            .expect("source directory should be created");
        fs::write(&source, "pub const LABEL: &str = \"Before\";\n")
            .expect("source should be written");
        let before = source_fingerprint(project.path()).expect("source should fingerprint");

        let generated = project.path().join(".hblank/target/debug/preview");
        fs::create_dir_all(generated.parent().expect("generated path has a parent"))
            .expect("generated directory should be created");
        fs::write(&generated, "build output").expect("generated output should be written");
        let after_generated =
            source_fingerprint(project.path()).expect("generated output should fingerprint");

        fs::write(&source, "pub const LABEL: &str = \"After\";\n")
            .expect("source should be updated");
        let after_source = source_fingerprint(project.path()).expect("source should fingerprint");

        assert_eq!(before, after_generated);
        assert_ne!(before, after_source);
    }

    #[test]
    fn resolves_relative_and_absolute_discovered_fixture_file_paths() {
        let project = tempfile::tempdir().expect("temporary project should be created");
        let project_root = project
            .path()
            .canonicalize()
            .expect("project should resolve");
        let fixture = project_root.join("src/card.hblank.rs");
        fs::create_dir_all(fixture.parent().expect("fixture has a parent"))
            .expect("fixture directory should be created");
        fs::write(&fixture, "// fixture\n").expect("fixture should be written");
        let fixture_files = vec![DiscoveredFixtureFile {
            relative_path: Path::new("src/card.hblank.rs").to_path_buf(),
            absolute_path: fixture.clone(),
            module_name: "__hblank_card".to_owned(),
        }];

        let relative = resolve_initial_fixture(
            &project_root,
            Path::new("src/card.hblank.rs"),
            &fixture_files,
        )
        .expect("relative fixture should resolve");
        let absolute = resolve_initial_fixture(&project_root, &fixture, &fixture_files)
            .expect("absolute fixture should resolve");

        assert_eq!(relative, fixture);
        assert_eq!(absolute, fixture);
    }

    #[test]
    fn rejects_missing_and_undiscovered_fixture_paths() {
        let project = tempfile::tempdir().expect("temporary project should be created");
        let project_root = project
            .path()
            .canonicalize()
            .expect("project should resolve");
        let unmatched = project_root.join("src/unmatched.rs");
        fs::create_dir_all(unmatched.parent().expect("fixture has a parent"))
            .expect("fixture directory should be created");
        fs::write(&unmatched, "// not discovered\n").expect("fixture should be written");

        let unmatched_error = resolve_initial_fixture(&project_root, &unmatched, &[])
            .expect_err("unmatched fixture should fail");
        let missing_error =
            resolve_initial_fixture(&project_root, Path::new("src/missing.hblank.rs"), &[])
                .expect_err("missing fixture should fail");

        assert!(matches!(
            unmatched_error,
            DevError::FixtureNotDiscovered(path) if path == unmatched
        ));
        assert!(matches!(
            missing_error,
            DevError::FixturePath { path, .. }
                if path == project_root.join("src/missing.hblank.rs")
        ));
    }
}

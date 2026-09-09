use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::Duration,
};

use semver::Version;
use serde::Deserialize;
use thiserror::Error;

mod manifest;

const RELEASE_PACKAGES: [&str; 4] = ["hblank-cli", "hblank", "hblank-core", "hblank-macros"];

#[derive(Clone, Debug)]
pub struct UpdateOptions {
    pub project_root: PathBuf,
    pub version: Option<String>,
}

impl UpdateOptions {
    #[must_use]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            version: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UpdateReport {
    pub version: String,
    pub updated_manifests: Vec<PathBuf>,
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("cannot update this Hblank project: {0}")]
    InvalidProject(String),
    #[error("could not access {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not run {operation}: {source}")]
    Process {
        operation: &'static str,
        source: std::io::Error,
    },
    #[error("{operation} failed: {status}")]
    CargoFailed {
        operation: &'static str,
        status: ExitStatus,
    },
    #[error(
        "invalid release version {0:?}; use an exact version such as --version 0.6.0, without 'v', '=', or a version range"
    )]
    InvalidVersion(String),
    #[error("could not resolve a complete crates.io Hblank release: {0}")]
    Registry(String),
    #[error(
        "{source}; rollback was incomplete: {failures}. Restore these files from your backup before retrying"
    )]
    Rollback {
        #[source]
        source: Box<Self>,
        failures: String,
    },
}

/// Updates the project's runtime dependencies and installed CLI to one crates.io release.
///
/// Stop any running `hblank dev` process before updating. This does not migrate GPUI,
/// convert dependency sources, regenerate the preview, or run project tests.
///
/// # Errors
/// Returns an error for unsupported projects, unavailable releases, filesystem errors,
/// or unsuccessful Cargo commands. After mutation, failures restore the original
/// manifests and lockfiles; an incomplete restoration is reported with the original error.
pub fn run_update(options: &UpdateOptions) -> Result<UpdateReport, UpdateError> {
    println!("Checking Hblank dependency sources and workspace manifests");
    let project = manifest::prepare(&options.project_root)?;
    let requested = options.version.as_deref().map(parse_version).transpose()?;
    println!("Looking up a complete Hblank release on crates.io");
    let version = resolve_release(requested.as_ref())?.to_string();
    let edits = project.edits(&version)?;

    println!("Updating Hblank CLI and runtimes to {version}");
    if edits.is_empty() {
        println!(
            "Runtime manifests already pin this release; refreshing lockfiles and checking the CLI installation"
        );
    }
    for edit in &edits {
        println!("  {}", edit.path.display());
    }
    apply_release(&project, &version, &edits, run_cargo)?;
    Ok(UpdateReport {
        version,
        updated_manifests: edits.into_iter().map(|edit| edit.path).collect(),
    })
}

fn parse_version(value: &str) -> Result<Version, UpdateError> {
    Version::parse(value).map_err(|_| UpdateError::InvalidVersion(value.to_owned()))
}

#[derive(Deserialize)]
struct IndexRelease {
    name: String,
    vers: String,
    yanked: bool,
}

fn available_versions(package: &str, index: &str) -> Result<BTreeSet<Version>, UpdateError> {
    let mut versions = BTreeSet::new();
    for (line, record) in index.lines().enumerate() {
        if record.trim().is_empty() {
            continue;
        }
        let release: IndexRelease = serde_json::from_str(record).map_err(|error| {
            UpdateError::Registry(format!(
                "invalid {package} index record on line {}: {error}; retry when crates.io is available",
                line + 1
            ))
        })?;
        if release.name != package {
            return Err(UpdateError::Registry(format!(
                "expected {package} index record, received {}",
                release.name
            )));
        }
        let version = Version::parse(&release.vers).map_err(|error| {
            UpdateError::Registry(format!(
                "invalid {package} release {}: {error}",
                release.vers
            ))
        })?;
        if !release.yanked {
            versions.insert(version);
        }
    }
    Ok(versions)
}

fn select_release(
    releases: &[BTreeSet<Version>; 4],
    requested: Option<&Version>,
) -> Result<Version, UpdateError> {
    if let Some(version) = requested {
        let unavailable: Vec<_> = RELEASE_PACKAGES
            .iter()
            .zip(releases)
            .filter_map(|(package, versions)| (!versions.contains(version)).then_some(*package))
            .collect();
        if unavailable.is_empty() {
            return Ok(version.clone());
        }
        return Err(UpdateError::Registry(format!(
            "version {version} is missing or yanked for {}; choose a release published and unyanked for all four Hblank crates",
            unavailable.join(", ")
        )));
    }
    releases[0]
        .iter()
        .rev()
        .find(|version| {
            version.pre.is_empty() && releases[1..].iter().all(|set| set.contains(*version))
        })
        .cloned()
        .ok_or_else(|| {
            UpdateError::Registry(
                "no common stable, unyanked version exists for hblank-cli, hblank, hblank-core, and hblank-macros; wait for publishing to finish or choose a complete release with --version".to_owned(),
            )
        })
}

fn resolve_release(requested: Option<&Version>) -> Result<Version, UpdateError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .https_only(true)
        .build()
        .into();
    let mut releases: [BTreeSet<Version>; 4] = std::array::from_fn(|_| BTreeSet::new());
    for (package, versions) in RELEASE_PACKAGES.iter().zip(&mut releases) {
        let url = format!("https://index.crates.io/hb/la/{package}");
        let mut response = agent
            .get(&url)
            .header(
                "User-Agent",
                concat!("hblank-cli/", env!("CARGO_PKG_VERSION")),
            )
            .call()
            .map_err(|error| {
                UpdateError::Registry(format!(
                    "fetching {url}: {error}; check your network and retry"
                ))
            })?;
        let body = response.body_mut().read_to_string().map_err(|error| {
            UpdateError::Registry(format!(
                "reading {url}: {error}; check your network and retry"
            ))
        })?;
        *versions = available_versions(package, &body)?;
    }
    select_release(&releases, requested)
}

struct FileSnapshot {
    path: PathBuf,
    contents: Option<Vec<u8>>,
}

fn snapshots(
    edits: &[manifest::ManifestUpdate],
    lockfiles: &[PathBuf],
) -> Result<Vec<FileSnapshot>, UpdateError> {
    let mut snapshots: Vec<_> = edits
        .iter()
        .map(|edit| FileSnapshot {
            path: edit.path.clone(),
            contents: Some(edit.original.as_bytes().to_vec()),
        })
        .collect();
    for path in lockfiles {
        if snapshots.iter().any(|snapshot| snapshot.path == *path) {
            continue;
        }
        let contents = match fs::read(path) {
            Ok(contents) => Some(contents),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(source) => {
                return Err(UpdateError::Io {
                    path: path.clone(),
                    source,
                });
            }
        };
        snapshots.push(FileSnapshot {
            path: path.clone(),
            contents,
        });
    }
    Ok(snapshots)
}

fn restore(snapshots: &[FileSnapshot], original: UpdateError) -> UpdateError {
    let mut failures = Vec::new();
    for snapshot in snapshots {
        let result = match &snapshot.contents {
            Some(contents) => fs::write(&snapshot.path, contents),
            None => match fs::remove_file(&snapshot.path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                result => result,
            },
        };
        if let Err(error) = result {
            failures.push(format!("{}: {error}", snapshot.path.display()));
        }
    }
    if failures.is_empty() {
        eprintln!("Update failed; original manifests and lockfiles restored.");
        original
    } else {
        UpdateError::Rollback {
            source: Box::new(original),
            failures: failures.join("; "),
        }
    }
}

fn apply_release(
    project: &manifest::ProjectUpdate,
    version: &str,
    edits: &[manifest::ManifestUpdate],
    mut process: impl FnMut(&mut Command, &'static str) -> Result<(), UpdateError>,
) -> Result<(), UpdateError> {
    for edit in edits {
        let current = fs::read(&edit.path).map_err(|source| UpdateError::Io {
            path: edit.path.clone(),
            source,
        })?;
        if current != edit.original.as_bytes() {
            return Err(UpdateError::InvalidProject(format!(
                "{} changed while checking the release; no files were written, rerun the update",
                edit.path.display()
            )));
        }
    }
    let snapshots = snapshots(edits, &project.lockfiles)?;
    let result = (|| {
        for edit in edits {
            fs::write(&edit.path, &edit.updated).map_err(|source| UpdateError::Io {
                path: edit.path.clone(),
                source,
            })?;
        }
        if project.host_has_runtime {
            update_lockfile(
                &project.host_manifest,
                "updating the host lockfile",
                &mut process,
            )?;
        }
        update_lockfile(
            &project.preview_manifest,
            "updating the preview lockfile",
            &mut process,
        )?;
        println!(
            "Installing hblank-cli {version} from crates.io (already installed versions are kept)"
        );
        let mut command = Command::new("cargo");
        command.args([
            "install",
            "hblank-cli",
            "--version",
            &format!("={version}"),
            "--registry",
            "crates-io",
            "--locked",
        ]);
        process(&mut command, "installing hblank-cli")
    })();
    result.map_err(|error| restore(&snapshots, error))
}

fn update_lockfile(
    manifest: &Path,
    operation: &'static str,
    process: &mut impl FnMut(&mut Command, &'static str) -> Result<(), UpdateError>,
) -> Result<(), UpdateError> {
    println!("Refreshing Hblank in {}", manifest.display());
    let mut command = Command::new("cargo");
    command
        .current_dir(
            manifest
                .parent()
                .expect("prepared manifest has an absolute path"),
        )
        .arg("update")
        .arg("--workspace")
        .arg("--manifest-path")
        .arg(manifest);
    process(&mut command, operation)
}

fn run_cargo(command: &mut Command, operation: &'static str) -> Result<(), UpdateError> {
    let status = command
        .stdin(Stdio::null())
        .status()
        .map_err(|source| UpdateError::Process { operation, source })?;
    if status.success() {
        Ok(())
    } else {
        Err(UpdateError::CargoFailed { operation, status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(package: &str, versions: &[(&str, bool)]) -> BTreeSet<Version> {
        let records = versions
            .iter()
            .map(|(version, yanked)| {
                serde_json::json!({"name": package, "vers": version, "yanked": yanked}).to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        available_versions(package, &records).expect("valid index")
    }

    #[test]
    fn latest_is_highest_stable_release_available_in_every_package() {
        let mut releases = RELEASE_PACKAGES.map(|package| {
            index(
                package,
                &[("0.9.0", false), ("0.10.0", false), ("1.0.0-rc.1", false)],
            )
        });
        releases[0].insert(Version::parse("0.11.0").unwrap());
        assert_eq!(
            select_release(&releases, None).unwrap().to_string(),
            "0.10.0"
        );
        releases[3] = index("hblank-macros", &[("0.9.0", false), ("0.10.0", true)]);
        assert_eq!(
            select_release(&releases, None).unwrap().to_string(),
            "0.9.0"
        );
    }

    #[test]
    fn explicit_release_allows_prerelease_but_rejects_missing_or_yanked_crates() {
        let mut releases = RELEASE_PACKAGES
            .map(|package| index(package, &[("0.6.0", false), ("0.7.0-rc.1", false)]));
        let requested = parse_version("0.7.0-rc.1").unwrap();
        assert_eq!(
            select_release(&releases, Some(&requested)).unwrap(),
            requested
        );
        assert!(select_release(&releases, Some(&parse_version("0.8.0").unwrap())).is_err());
        releases[2] = index("hblank-core", &[("0.6.0", false), ("0.7.0-rc.1", true)]);
        let error = select_release(&releases, Some(&requested)).unwrap_err();
        assert!(error.to_string().contains("hblank-core"));
    }

    #[test]
    fn no_stable_common_release_and_invalid_pins_are_errors() {
        let releases = RELEASE_PACKAGES.map(|package| index(package, &[("1.0.0-beta.1", false)]));
        assert!(select_release(&releases, None).is_err());
        for invalid in ["=0.6.0", "v0.6.0", "^0.6.0", "0.6", "*", " 0.6.0"] {
            assert!(matches!(
                parse_version(invalid),
                Err(UpdateError::InvalidVersion(_))
            ));
        }
        assert!(available_versions("hblank", "not JSON").is_err());
        assert!(
            available_versions(
                "hblank",
                r#"{"name":"other","vers":"0.6.0","yanked":false}"#
            )
            .is_err()
        );
    }

    fn project_fixture() -> (tempfile::TempDir, manifest::ProjectUpdate) {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("src")).unwrap();
        fs::create_dir_all(directory.path().join(".hblank/src")).unwrap();
        fs::write(
            directory.path().join(crate::CONFIG_PATH),
            crate::Config::for_project("update-fixture")
                .to_toml()
                .unwrap(),
        )
        .unwrap();
        fs::write(directory.path().join("src/lib.rs"), "").unwrap();
        fs::write(
            directory.path().join(".hblank/src/main.rs"),
            "fn main() {}\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            "[package]\nname = \"update-fixture\"\nversion = \"0.1.0\"\n[dependencies]\nhblank = \"=0.5.0\"\n",
        ).unwrap();
        fs::write(
            directory.path().join(".hblank/Cargo.toml"),
            "[package]\nname = \"update-preview\"\nversion = \"0.1.0\"\n[workspace]\n[dependencies]\nhblank = { version = \"=0.5.0\", features = [\"test-support\"] }\n",
        ).unwrap();
        let project = manifest::prepare(directory.path()).expect("valid initialized project");
        (directory, project)
    }

    fn process_failure(operation: &'static str) -> UpdateError {
        UpdateError::Process {
            operation,
            source: std::io::Error::other("injected process failure"),
        }
    }

    #[test]
    fn update_failure_restores_manifests_and_existing_and_new_lockfiles() {
        let (_directory, project) = project_fixture();
        let edits = project.edits("0.6.0").unwrap();
        let host_lock = project.host_manifest.with_file_name("Cargo.lock");
        let preview_lock = project.preview_manifest.with_file_name("Cargo.lock");
        fs::write(&host_lock, b"original host lock\n").unwrap();
        let result = apply_release(&project, "0.6.0", &edits, |_, operation| {
            fs::write(&host_lock, b"resolved host lock\n").unwrap();
            fs::write(&preview_lock, b"new preview lock\n").unwrap();
            Err(process_failure(operation))
        });
        assert!(result.is_err());
        for edit in edits {
            assert_eq!(fs::read_to_string(&edit.path).unwrap(), edit.original);
        }
        assert_eq!(fs::read(&host_lock).unwrap(), b"original host lock\n");
        assert!(!preview_lock.exists());
    }

    #[test]
    fn installer_failure_restores_both_successfully_resolved_lockfiles() {
        let (_directory, project) = project_fixture();
        let edits = project.edits("0.6.0").unwrap();
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(1 << 8)
        };
        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(1)
        };
        let locks: Vec<_> = project
            .lockfiles
            .iter()
            .map(|path| {
                let bytes = format!("original {}\n", path.display()).into_bytes();
                fs::write(path, &bytes).unwrap();
                (path, bytes)
            })
            .collect();
        let result = apply_release(&project, "0.6.0", &edits, |_, operation| {
            for (path, _) in &locks {
                fs::write(path, b"resolved release lock\n").unwrap();
            }
            if operation == "installing hblank-cli" {
                Err(UpdateError::CargoFailed { operation, status })
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        for edit in edits {
            assert_eq!(fs::read_to_string(&edit.path).unwrap(), edit.original);
        }
        for (path, original) in locks {
            assert_eq!(fs::read(path).unwrap(), original);
        }
    }

    #[test]
    fn successful_repeat_keeps_exact_pins_and_reports_no_manifest_edits() {
        let (directory, project) = project_fixture();
        let edits = project.edits("0.6.0").unwrap();
        apply_release(&project, "0.6.0", &edits, |_, _| Ok(())).unwrap();
        let project = manifest::prepare(directory.path()).unwrap();
        assert!(project.edits("0.6.0").unwrap().is_empty());
    }

    #[test]
    fn rollback_failure_preserves_original_error_and_restores_other_files() {
        let (_directory, project) = project_fixture();
        let edits = project.edits("0.6.0").unwrap();
        let result = apply_release(&project, "0.6.0", &edits, |_, operation| {
            fs::remove_file(&edits[0].path).unwrap();
            fs::create_dir(&edits[0].path).unwrap();
            Err(process_failure(operation))
        });
        let error = result.unwrap_err();
        assert!(matches!(&error, UpdateError::Rollback { .. }));
        assert!(error.to_string().contains("injected process failure"));
        assert!(
            error
                .to_string()
                .contains(&edits[0].path.display().to_string())
        );
        for edit in &edits[1..] {
            assert_eq!(fs::read_to_string(&edit.path).unwrap(), edit.original);
        }
    }

    #[test]
    fn concurrent_manifest_edit_is_not_overwritten_or_rolled_back() {
        let (_directory, project) = project_fixture();
        let edits = project.edits("0.6.0").unwrap();
        let user_edit = format!("{}# changed by the user\n", edits[0].original);
        fs::write(&edits[0].path, &user_edit).unwrap();
        let result = apply_release(&project, "0.6.0", &edits, |_, _| {
            panic!("Cargo must not run after a concurrent manifest change")
        });
        assert!(matches!(result, Err(UpdateError::InvalidProject(_))));
        assert_eq!(fs::read_to_string(&edits[0].path).unwrap(), user_edit);
        for edit in &edits[1..] {
            assert_eq!(fs::read_to_string(&edit.path).unwrap(), edit.original);
        }
        assert!(project.lockfiles.iter().all(|path| !path.exists()));
    }
}

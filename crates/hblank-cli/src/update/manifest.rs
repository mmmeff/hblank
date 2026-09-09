use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use toml_edit::{DocumentMut, Item, Value};

use super::UpdateError;

const PREVIEW_MANIFEST: &str = ".hblank/Cargo.toml";
const PREVIEW_LOCKFILE: &str = ".hblank/Cargo.lock";
const HBLANK_CRATES: &[&str] = &["hblank", "hblank-core", "hblank-macros", "hblank-cli"];

#[derive(Debug)]
pub(super) struct ProjectUpdate {
    pub(super) host_manifest: PathBuf,
    pub(super) preview_manifest: PathBuf,
    pub(super) lockfiles: Vec<PathBuf>,
    pub(super) host_has_runtime: bool,
    manifests: Vec<ManifestFile>,
    targets: Vec<VersionTarget>,
}

#[derive(Debug)]
struct ManifestFile {
    path: PathBuf,
    original: String,
    document: DocumentMut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VersionTarget {
    manifest: PathBuf,
    key_path: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ManifestUpdate {
    pub(super) path: PathBuf,
    pub(super) original: String,
    pub(super) updated: String,
}

#[derive(Debug)]
struct DependencyDecl {
    key_path: Vec<String>,
    package: String,
    workspace: bool,
    source_override: Option<&'static str>,
}

/// Reads and validates the host and private preview manifests without writing files.
///
/// The returned plan contains all dependency version locations that can be changed later,
/// including locations inherited through a workspace root.
///
/// # Errors
/// Returns an error if the project is not initialized, Cargo cannot identify a workspace,
/// a manifest is malformed, or a Hblank dependency uses a source override.
pub(super) fn prepare(project_root: &Path) -> Result<ProjectUpdate, UpdateError> {
    let project_root = canonicalize_path(project_root, "project root")?;
    crate::Config::load(&project_root)
        .map_err(|error| invalid(format!("invalid Hblank configuration: {error}")))?;

    let host_manifest = canonicalize_file(&project_root.join("Cargo.toml"), "host manifest")?;
    let preview_manifest =
        canonicalize_file(&project_root.join(PREVIEW_MANIFEST), "preview manifest")?;

    let mut manifests = Vec::new();
    load_manifest(&mut manifests, &host_manifest)?;
    load_manifest(&mut manifests, &preview_manifest)?;
    // Reject direct overrides before Cargo tries to interpret mixed source declarations.
    for manifest in &manifests {
        preflight_manifest(manifest)?;
    }
    let host_workspace = locate_workspace(&host_manifest)?;
    let preview_workspace = locate_workspace(&preview_manifest)?;
    load_manifest(&mut manifests, &host_workspace)?;
    load_manifest(&mut manifests, &preview_workspace)?;
    for manifest in &manifests {
        preflight_manifest(manifest)?;
    }

    let mut targets = Vec::new();
    let mut preview_has_runtime = false;
    for (is_host, source_path, workspace_path) in [
        (true, host_manifest.clone(), host_workspace.clone()),
        (false, preview_manifest.clone(), preview_workspace.clone()),
    ] {
        let source = manifest(&manifests, &source_path)?;
        for declaration in dependency_declarations(&source.document)? {
            if declaration
                .key_path
                .first()
                .is_some_and(|key| key == "workspace")
            {
                continue;
            }

            let (target_path, target_decl) = if declaration.workspace {
                let owner = manifest(&manifests, &workspace_path)?;
                let key = declaration.key_path.last().map_or("", String::as_str);
                let owner_path = vec![
                    "workspace".to_owned(),
                    "dependencies".to_owned(),
                    key.to_owned(),
                ];
                let owner_item = item_at(&owner.document, &owner_path).ok_or_else(|| {
                    invalid(format!(
                        "{} inherits Hblank dependency '{}' with workspace = true, but {} does not define it in [workspace.dependencies]; add that declaration manually and retry",
                        source_path.display(),
                        key,
                        workspace_path.display()
                    ))
                })?;
                let owner_decl = dependency_declaration(key, owner_path, owner_item)?;
                if !is_hblank(&owner_decl.package) {
                    continue;
                }
                reject_dependency_override(&source_path, &declaration)?;
                reject_dependency_override(&workspace_path, &owner_decl)?;
                (workspace_path.clone(), owner_decl)
            } else {
                (source_path.clone(), declaration)
            };

            if !is_hblank(&target_decl.package) {
                continue;
            }
            reject_dependency_override(&target_path, &target_decl)?;
            if !is_host && target_decl.package == "hblank" {
                preview_has_runtime = true;
            }
            add_target(&mut targets, &target_path, &target_decl.key_path);
        }
    }
    let host_has_runtime = targets
        .iter()
        .any(|target| target.manifest == host_manifest || target.manifest == host_workspace);

    if !preview_has_runtime {
        return Err(invalid(format!(
            "Hblank preview manifest at {} does not declare the hblank runtime; restore its crates.io hblank dependency manually and retry",
            preview_manifest.display()
        )));
    }

    let mut lockfiles = Vec::new();
    for workspace in [host_workspace, preview_workspace] {
        if let Some(parent) = workspace.parent() {
            push_unique(&mut lockfiles, parent.join("Cargo.lock"));
        }
    }
    push_unique(&mut lockfiles, project_root.join(PREVIEW_LOCKFILE));

    Ok(ProjectUpdate {
        host_manifest,
        preview_manifest,
        lockfiles,
        host_has_runtime,
        manifests,
        targets,
    })
}

impl ProjectUpdate {
    /// Produces coordinated in-memory manifest edits for one exact crates.io version.
    pub(super) fn edits(&self, version: &str) -> Result<Vec<ManifestUpdate>, UpdateError> {
        let mut updates = Vec::new();
        for manifest in &self.manifests {
            let mut document = manifest.document.clone();
            let mut changed = false;
            for target in self
                .targets
                .iter()
                .filter(|target| target.manifest == manifest.path)
            {
                let item = item_at_mut(&mut document, &target.key_path).ok_or_else(|| {
                    invalid(format!(
                        "planned Hblank dependency disappeared from {}",
                        manifest.path.display()
                    ))
                })?;
                changed |= set_dependency_version(item, version)
                    .map_err(|error| invalid(format!("{error} in {}", manifest.path.display())))?;
            }
            if changed {
                updates.push(ManifestUpdate {
                    path: manifest.path.clone(),
                    original: manifest.original.clone(),
                    updated: document.to_string(),
                });
            }
        }
        Ok(updates)
    }
}

fn load_manifest(manifests: &mut Vec<ManifestFile>, path: &Path) -> Result<(), UpdateError> {
    if manifests.iter().any(|manifest| manifest.path == path) {
        return Ok(());
    }
    let original = fs::read_to_string(path).map_err(|source| UpdateError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let document = original.parse::<DocumentMut>().map_err(|error| {
        invalid(format!(
            "could not parse Rust manifest at {}: {error}",
            path.display()
        ))
    })?;
    manifests.push(ManifestFile {
        path: path.to_path_buf(),
        original,
        document,
    });
    Ok(())
}

fn preflight_manifest(manifest: &ManifestFile) -> Result<(), UpdateError> {
    reject_source_overrides(manifest)?;
    for declaration in dependency_declarations(&manifest.document)? {
        if is_hblank(&declaration.package) {
            reject_dependency_override(&manifest.path, &declaration)?;
            if !declaration.workspace {
                let item = item_at(&manifest.document, &declaration.key_path)
                    .expect("declaration came from this document");
                let version = if item.is_str() {
                    Some(item)
                } else {
                    item.get("version")
                };
                if !version.is_some_and(Item::is_str) {
                    return Err(invalid(format!(
                        "{} declares Hblank crate '{}' without a string crates.io version; set its version manually and retry",
                        manifest.path.display(),
                        declaration.package
                    )));
                }
            }
        }
    }
    Ok(())
}

fn canonicalize_path(path: &Path, description: &str) -> Result<PathBuf, UpdateError> {
    path.canonicalize().map_err(|source| UpdateError::Io {
        path: path.to_path_buf(),
        source: std::io::Error::new(
            source.kind(),
            format!("could not canonicalize {description}: {source}"),
        ),
    })
}

fn canonicalize_file(path: &Path, description: &str) -> Result<PathBuf, UpdateError> {
    if !path.is_file() {
        return Err(invalid(format!(
            "{description} is missing at {}; initialize the project before updating",
            path.display()
        )));
    }
    canonicalize_path(path, description)
}

fn locate_workspace(manifest: &Path) -> Result<PathBuf, UpdateError> {
    let mut command = Command::new("cargo");
    command
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format")
        .arg("plain")
        .arg("--manifest-path")
        .arg(manifest)
        .stdin(Stdio::null());
    let output = command.output().map_err(|source| UpdateError::Process {
        operation: "locate Cargo workspace",
        source,
    })?;
    if !output.status.success() {
        return Err(invalid(format!(
            "Cargo could not locate the workspace for {}: {}; repair the manifest or update non-crates.io Hblank sources manually and retry",
            manifest.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let located = stdout.trim();
    if located.is_empty() {
        return Err(invalid(format!(
            "Cargo returned no workspace manifest for {}; locate the workspace manually and retry",
            manifest.display()
        )));
    }
    let path = Path::new(located);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    };
    canonicalize_file(&path, "Cargo workspace manifest")
}

fn dependency_declarations(document: &DocumentMut) -> Result<Vec<DependencyDecl>, UpdateError> {
    let mut declarations = Vec::new();
    let mut path = Vec::new();
    walk_dependency_tables(document.as_item(), &mut path, &mut declarations)?;
    Ok(declarations)
}

fn walk_dependency_tables(
    item: &Item,
    path: &mut Vec<String>,
    declarations: &mut Vec<DependencyDecl>,
) -> Result<(), UpdateError> {
    let Some(table) = item.as_table_like() else {
        return Ok(());
    };
    for (key, child) in table.iter() {
        let key = key.to_owned();
        path.push(key.clone());
        if is_dependency_section(path) {
            if let Some(dependency_table) = child.as_table_like() {
                for (dependency_key, dependency_item) in dependency_table.iter() {
                    let dependency_key = dependency_key.to_owned();
                    let mut key_path = path.clone();
                    key_path.push(dependency_key.clone());
                    declarations.push(dependency_declaration(
                        &dependency_key,
                        key_path,
                        dependency_item,
                    )?);
                }
            }
        }
        if path.len() < 3 {
            walk_dependency_tables(child, path, declarations)?;
        }
        path.pop();
    }
    Ok(())
}

fn is_dependency_section(path: &[String]) -> bool {
    let dependency_key = path.last().is_some_and(|key| {
        matches!(
            key.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        )
    });
    dependency_key
        && (path.len() == 1
            || (path.len() == 2 && path[0] == "workspace" && path[1] == "dependencies")
            || (path.len() == 3 && path[0] == "target"))
}

fn dependency_declaration(
    key: &str,
    key_path: Vec<String>,
    item: &Item,
) -> Result<DependencyDecl, UpdateError> {
    let package = field(item, "package")
        .map(|value| {
            value
                .as_value()
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| {
                    invalid(format!("dependency '{key}' has a non-string package field"))
                })
        })
        .transpose()?
        .unwrap_or_else(|| key.to_owned());
    let workspace = field(item, "workspace")
        .map(|value| {
            value.as_value().and_then(Value::as_bool).ok_or_else(|| {
                invalid(format!(
                    "dependency '{key}' has a non-boolean workspace field"
                ))
            })
        })
        .transpose()?
        .unwrap_or(false);
    let source_override = [
        ("git", "git"),
        ("path", "path"),
        ("registry", "alternate registry"),
        ("registry-index", "alternate registry"),
    ]
    .into_iter()
    .find_map(|(field_name, description)| {
        field(item, field_name).and_then(|value| {
            (field_name != "registry" || value.as_str() != Some("crates-io")).then_some(description)
        })
    });

    Ok(DependencyDecl {
        key_path,
        package,
        workspace,
        source_override,
    })
}

fn field<'a>(item: &'a Item, name: &str) -> Option<&'a Item> {
    item.get(name)
}

fn field_mut<'a>(item: &'a mut Item, name: &str) -> Option<&'a mut Item> {
    item.get_mut(name)
}

fn item_at<'a>(document: &'a DocumentMut, key_path: &[String]) -> Option<&'a Item> {
    let mut item = document.as_item();
    for key in key_path {
        item = item.get(key.as_str())?;
    }
    Some(item)
}

fn item_at_mut<'a>(document: &'a mut DocumentMut, key_path: &[String]) -> Option<&'a mut Item> {
    let mut item = document.as_item_mut();
    for key in key_path {
        item = item.get_mut(key.as_str())?;
    }
    Some(item)
}

fn set_dependency_version(item: &mut Item, version: &str) -> Result<bool, &'static str> {
    let exact = format!("={version}");
    let version_item = if item.is_str() {
        item
    } else {
        field_mut(item, "version").ok_or("dependency requires a crates.io version")?
    };
    let value = version_item
        .as_value_mut()
        .ok_or("dependency version must be a string")?;
    let Value::String(formatted) = value else {
        return Err("dependency version must be a string");
    };
    if formatted.value() == &exact {
        return Ok(false);
    }
    // Keep the original string delimiter as well as its whitespace and comments.
    let representation = formatted.display_repr();
    let delimiter = if representation.starts_with("'''") {
        "'''"
    } else if representation.starts_with('\'') {
        "'"
    } else if representation.starts_with("\"\"\"") {
        "\"\"\""
    } else {
        "\""
    };
    let mut replacement = format!("{delimiter}{exact}{delimiter}")
        .parse::<Value>()
        .map_err(|_| "invalid exact dependency version")?;
    *replacement.decor_mut() = value.decor().clone();
    *value = replacement;
    Ok(true)
}

fn reject_source_overrides(manifest: &ManifestFile) -> Result<(), UpdateError> {
    let document = &manifest.document;
    if let Some(patch) = document.get("patch") {
        if let Some(table) = patch.as_table_like() {
            for (source, overrides) in table.iter() {
                if let Some(overrides) = overrides.as_table_like() {
                    for (key, item) in overrides.iter() {
                        let package = field(item, "package").and_then(Item::as_str).unwrap_or(key);
                        if is_hblank(package) {
                            return Err(invalid(format!(
                                "{} patches Hblank crate '{package}' in [patch.{source}]; remove the override or update it manually before retrying the crates.io update",
                                manifest.path.display()
                            )));
                        }
                    }
                }
            }
        }
    }
    if let Some(replace) = document.get("replace") {
        if let Some(table) = replace.as_table_like() {
            for (key, item) in table.iter() {
                let package = key.split(':').next().unwrap_or(key);
                let package = field(item, "package")
                    .and_then(|value| value.as_value().and_then(Value::as_str))
                    .unwrap_or(package);
                if is_hblank(package) {
                    return Err(invalid(format!(
                        "{} replaces Hblank crate '{}'; remove the [replace] override or convert it to a crates.io dependency manually, then retry",
                        manifest.path.display(),
                        package
                    )));
                }
            }
        }
    }
    Ok(())
}

fn reject_dependency_override(
    path: &Path,
    declaration: &DependencyDecl,
) -> Result<(), UpdateError> {
    if let Some(source) = declaration.source_override {
        return Err(invalid(format!(
            "{} declares Hblank crate '{}' with a {source} source; convert it to a crates.io dependency manually and retry",
            path.display(),
            declaration.package
        )));
    }
    Ok(())
}

fn add_target(targets: &mut Vec<VersionTarget>, manifest: &Path, key_path: &[String]) {
    let target = VersionTarget {
        manifest: manifest.to_path_buf(),
        key_path: key_path.to_vec(),
    };
    if !targets.iter().any(|existing| existing == &target) {
        targets.push(target);
    }
}

fn manifest<'a>(
    manifests: &'a [ManifestFile],
    path: &Path,
) -> Result<&'a ManifestFile, UpdateError> {
    manifests
        .iter()
        .find(|manifest| manifest.path == path)
        .ok_or_else(|| {
            invalid(format!(
                "Cargo workspace manifest was not loaded: {}",
                path.display()
            ))
        })
}

fn is_hblank(package: &str) -> bool {
    HBLANK_CRATES.contains(&package)
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn invalid(message: String) -> UpdateError {
    UpdateError::InvalidProject(message)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn project(host: &str, preview: &str) -> TempDir {
        let directory = tempfile::tempdir().expect("temporary project");
        let root = directory.path();
        fs::create_dir_all(root.join(".hblank/src")).expect("preview source");
        fs::create_dir_all(root.join("src")).expect("host source");
        fs::write(root.join("src/lib.rs"), "pub fn fixture() {}\n").expect("host source file");
        fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n{host}"
            ),
        )
        .expect("host manifest");
        fs::write(
            root.join(".hblank/config.toml"),
            crate::Config::for_project("sample")
                .to_toml()
                .expect("config serialization"),
        )
        .expect("config");
        fs::write(
            root.join(".hblank/Cargo.toml"),
            format!(
                "[package]\nname = \"sample-preview\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\n[workspace]\n\n[dependencies]\n{preview}"
            ),
        )
        .expect("preview manifest");
        fs::write(root.join(".hblank/src/main.rs"), "fn main() {}\n").expect("preview source");
        directory
    }

    #[test]
    fn edits_preserve_comments_features_and_exact_pin() {
        let directory = project(
            "[dependencies]\n# Keep host options\nhblank = { version = \"0.4\", features = [\"test-support\"], default-features = false } # host\n",
            "# Keep preview options\nhblank = { version = \"0.4\", features = [\"test-support\"] } # preview\n",
        );
        let plan = prepare(directory.path()).expect("preflight");
        let updates = plan.edits("1.2.3").expect("edits");
        assert_eq!(updates.len(), 2);
        let host = updates
            .iter()
            .find(|update| update.path == directory.path().join("Cargo.toml"))
            .expect("host update");
        assert_eq!(host.updated, host.original.replace("\"0.4\"", "\"=1.2.3\""));
        for update in updates {
            assert_eq!(
                fs::read_to_string(&update.path).expect("unchanged manifest"),
                update.original
            );
        }
    }

    #[test]
    fn workspace_inherited_dependency_changes_owner_only() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        fs::create_dir_all(root.join("member/src")).expect("member source");
        fs::create_dir_all(root.join("member/.hblank/src")).expect("preview source");
        fs::write(root.join("member/src/lib.rs"), "pub fn fixture() {}\n").expect("source");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"member\"]\n\n[workspace.dependencies]\nhb = { package = \"hblank\", version = \"0.4\", features = [\"test-support\"], default-features = false }\nother_core = { package = \"hblank-core\", version = \"0.1\" }\n").expect("workspace");
        fs::write(root.join("member/Cargo.toml"), "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nhb = { workspace = true, default-features = false }\n\n[target.'cfg(unix)'.dev-dependencies]\nhb.workspace = true\n").expect("member");
        fs::write(
            root.join("member/.hblank/config.toml"),
            crate::Config::for_project("sample")
                .to_toml()
                .expect("config"),
        )
        .expect("config");
        fs::write(root.join("member/.hblank/Cargo.toml"), "[package]\nname = \"sample-preview\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[workspace]\n\n[dependencies]\nhblank = \"0.4\"\n").expect("preview");
        fs::write(root.join("member/.hblank/src/main.rs"), "fn main() {}\n")
            .expect("preview source");

        let plan = prepare(root.join("member").as_path()).expect("preflight");
        let updates = plan.edits("1.2.3").expect("edits");
        let workspace = updates
            .iter()
            .find(|update| update.path == root.join("Cargo.toml"))
            .expect("workspace update");
        assert!(workspace.updated.contains("version = \"=1.2.3\""));
        assert!(
            workspace
                .updated
                .contains("other_core = { package = \"hblank-core\", version = \"0.1\" }")
        );
        assert!(
            !updates
                .iter()
                .any(|update| update.path == root.join("member/Cargo.toml"))
        );
        assert_eq!(
            plan.lockfiles,
            vec![
                root.join("Cargo.lock"),
                root.join("member/.hblank/Cargo.lock")
            ]
        );
    }

    #[test]
    fn aliases_and_target_dependencies_are_updated() {
        let directory = project(
            "[dependencies]\nhb = { package = \"hblank\", version = \"0.4\", optional = true }\n\n[target.'cfg(unix)'.dev-dependencies]\ncore = { package = \"hblank-core\", version = \"0.4\" }\n\n[build-dependencies.hblank-macros]\nversion = '0.4' # macro version\n\n[package.metadata.dependencies]\nhblank = \"unrelated\"\n",
            "hblank = \"0.4\"\n",
        );
        let plan = prepare(directory.path()).expect("preflight");
        let updates = plan.edits("1.2.3").expect("edits");
        let host = updates
            .iter()
            .find(|update| update.path == directory.path().join("Cargo.toml"))
            .expect("host update");
        assert_eq!(
            host.updated,
            host.original
                .replace("\"0.4\"", "\"=1.2.3\"")
                .replace("'0.4'", "'=1.2.3'")
        );
    }

    #[test]
    fn source_overrides_are_rejected_without_writes() {
        for (host, preview) in [
            (
                "[dependencies]\nhblank = { version = \"0.4\", git = \"https://example.invalid/hblank\" }\n",
                "hblank = \"0.4\"\n",
            ),
            (
                "[dependencies]\nhb = { package = \"hblank\", version = \"0.4\", path = \"../local\" }\n",
                "hblank = \"0.4\"\n",
            ),
            (
                "",
                "hblank = { version = \"0.4\", registry = \"private\" }\n",
            ),
            (
                "[patch.crates-io]\npatched = { package = \"hblank-core\", path = \"../local\" }\n",
                "hblank = \"0.4\"\n",
            ),
            (
                "[replace]\n\"hblank-macros:0.4.0\" = { path = \"../local\" }\n",
                "hblank = \"0.4\"\n",
            ),
            (
                "[dependencies]\nhblank = { version = \"0.4\", git = \"https://example.invalid/hblank\", path = \"../local\" }\n",
                "hblank = { version = \"0.4\", registry = \"private\" }\n",
            ),
        ] {
            let directory = project(host, preview);
            let host_path = directory.path().join("Cargo.toml");
            let preview_path = directory.path().join(".hblank/Cargo.toml");
            let host_before = fs::read_to_string(&host_path).expect("host");
            let preview_before = fs::read_to_string(&preview_path).expect("preview");
            let error = prepare(directory.path()).expect_err("override rejection");
            assert!(matches!(error, UpdateError::InvalidProject(_)));
            assert_eq!(
                fs::read_to_string(host_path).expect("host after"),
                host_before
            );
            assert_eq!(
                fs::read_to_string(preview_path).expect("preview after"),
                preview_before
            );
            assert!(!directory.path().join("Cargo.lock").exists());
            assert!(!directory.path().join(".hblank/Cargo.lock").exists());
        }
    }

    #[test]
    fn missing_host_runtime_and_already_pinned_preview_need_no_edits() {
        let directory = project("", "hblank = '=1.2.3' # pinned\n");
        let plan = prepare(directory.path()).expect("preflight");
        assert!(!plan.host_has_runtime);
        assert!(plan.edits("1.2.3").expect("edits").is_empty());
    }

    #[test]
    fn family_only_host_requires_lockfile_refresh() {
        let directory = project(
            "[dependencies]\ncore = { package = \"hblank-core\", version = \"0.4\" }\n",
            "hblank = \"0.4\"\n",
        );
        let plan = prepare(directory.path()).expect("preflight");
        assert!(plan.host_has_runtime);
        let updates = plan.edits("1.2.3").expect("edits");
        assert!(
            updates
                .iter()
                .any(|update| update.path == plan.host_manifest)
        );
    }

    #[test]
    fn preview_must_declare_runtime_even_with_family_dependencies() {
        let directory = project(
            "[dependencies]\nhblank = \"0.4\"\n",
            "hblank-core = \"0.4\"\n",
        );
        assert!(matches!(
            prepare(directory.path()),
            Err(UpdateError::InvalidProject(_))
        ));
    }
}

use std::{collections::BTreeMap, fs, path::Path};

use hblank_cli::{Config, InitError, InitOptions, initialize};
use tempfile::TempDir;

fn project() -> TempDir {
    let directory = tempfile::tempdir().expect("temporary project should be created");
    fs::create_dir(directory.path().join("src")).expect("source directory should be created");
    fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\nname = \"demo-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest should be written");
    fs::write(directory.path().join("src/lib.rs"), "pub fn marker() {}\n")
        .expect("source should be written");
    directory
}

fn generated_bytes(project: &Path) -> BTreeMap<String, Vec<u8>> {
    [
        ".hblank/config.toml",
        ".hblank/Cargo.toml",
        ".hblank/src/main.rs",
        ".hblank/generated/fixtures.rs",
        ".hblank/.gitignore",
    ]
    .into_iter()
    .map(|relative| {
        (
            relative.to_owned(),
            fs::read(project.join(relative)).expect("generated file should be readable"),
        )
    })
    .collect()
}

#[test]
fn initializes_complete_preview_without_touching_host_files() {
    let project = project();
    let manifest_before = fs::read(project.path().join("Cargo.toml")).expect("manifest exists");
    let source_before = fs::read(project.path().join("src/lib.rs")).expect("source exists");

    let report = initialize(&InitOptions::new(project.path())).expect("init should succeed");

    assert_eq!(report.created.len(), 5);
    assert_eq!(
        fs::read(project.path().join("Cargo.toml")).expect("manifest exists"),
        manifest_before
    );
    assert_eq!(
        fs::read(project.path().join("src/lib.rs")).expect("source exists"),
        source_before
    );

    let config = Config::load(project.path()).expect("generated config should parse");
    assert_eq!(config.fixtures, ["src/**/*.hblank.rs"]);
    assert_eq!(config.window.title, "demo-app · Hblank");

    let preview_manifest = fs::read_to_string(project.path().join(".hblank/Cargo.toml"))
        .expect("preview manifest should exist");
    let parsed = toml::from_str::<toml::Value>(&preview_manifest)
        .expect("preview manifest should be valid TOML");
    assert_eq!(
        parsed["package"]["name"].as_str(),
        Some("demo-app-hblank-preview")
    );
    let source_runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../hblank")
        .canonicalize()
        .expect("source runtime should exist");
    assert_eq!(
        parsed["dependencies"]["hblank"]["path"].as_str(),
        Some(source_runtime.to_string_lossy().as_ref())
    );
    assert_eq!(
        parsed["dependencies"]["hblank_project"]["package"].as_str(),
        Some("demo-app")
    );
}

#[test]
fn refuses_rerun_without_changing_any_generated_byte() {
    let project = project();
    initialize(&InitOptions::new(project.path())).expect("first init should succeed");
    let before = generated_bytes(project.path());

    let error = initialize(&InitOptions::new(project.path())).expect_err("rerun must fail");

    let InitError::ExistingFiles(paths) = error else {
        panic!("expected existing-files error, received {error}");
    };
    assert_eq!(paths.len(), 5);
    assert_eq!(generated_bytes(project.path()), before);
}

#[test]
fn existing_config_prevents_all_other_writes() {
    let project = project();
    fs::create_dir(project.path().join(".hblank")).expect("hblank directory should be created");
    fs::write(
        project.path().join(".hblank/config.toml"),
        "user-owned = true\n",
    )
    .expect("sentinel config should be written");

    let error = initialize(&InitOptions::new(project.path())).expect_err("init must fail");

    assert!(matches!(error, InitError::ExistingFiles(_)));
    assert_eq!(
        fs::read_to_string(project.path().join(".hblank/config.toml"))
            .expect("sentinel should remain"),
        "user-owned = true\n"
    );
    assert!(!project.path().join(".hblank/Cargo.toml").exists());
    assert!(!project.path().join(".hblank/src/main.rs").exists());
}

#[test]
fn local_runtime_path_is_resolved_for_the_preview_manifest() {
    let project = project();
    let runtime = project.path().join("vendor/hblank");
    fs::create_dir_all(&runtime).expect("runtime directory should be created");
    let mut options = InitOptions::new(project.path());
    options.runtime_path = Some(Path::new("vendor/hblank").to_path_buf());

    initialize(&options).expect("init should accept local runtime");

    let preview_manifest = fs::read_to_string(project.path().join(".hblank/Cargo.toml"))
        .expect("preview manifest should exist");
    let parsed =
        toml::from_str::<toml::Value>(&preview_manifest).expect("preview manifest should parse");
    assert_eq!(
        parsed["dependencies"]["hblank"]["path"].as_str(),
        Some(
            runtime
                .canonicalize()
                .expect("runtime should canonicalize")
                .to_string_lossy()
                .as_ref()
        )
    );
}

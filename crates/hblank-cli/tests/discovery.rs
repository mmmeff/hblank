use std::fs;

use hblank_cli::{Config, discover_examples, refresh_generated_examples};

fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, contents).expect("fixture file should be written");
}

#[test]
fn discovers_direct_and_nested_examples_in_stable_order() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write(&project.path().join("src/zeta.hblank.rs"), "// zeta\n");
    write(
        &project.path().join("src/components/alpha.hblank.rs"),
        "// alpha\n",
    );
    write(
        &project.path().join("src/component.rs"),
        "// not an example\n",
    );
    write(
        &project.path().join("target/generated.hblank.rs"),
        "// ignored\n",
    );

    let examples =
        discover_examples(project.path(), &Config::default()).expect("discovery should succeed");

    assert_eq!(examples.len(), 2);
    assert_eq!(
        examples[0].relative_path,
        std::path::Path::new("src/components/alpha.hblank.rs")
    );
    assert_eq!(
        examples[1].relative_path,
        std::path::Path::new("src/zeta.hblank.rs")
    );
    assert_ne!(examples[0].module_name, examples[1].module_name);
}

#[test]
fn applies_custom_patterns_and_ignores() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write(
        &project.path().join("examples/a.fixture.rs"),
        "// include\n",
    );
    write(
        &project.path().join("examples/private/b.fixture.rs"),
        "// ignore\n",
    );
    let config = Config {
        examples: vec!["examples/**/*.fixture.rs".to_owned()],
        ignore: vec!["examples/private/**".to_owned()],
        ..Config::default()
    };

    let examples = discover_examples(project.path(), &config).expect("discovery should succeed");

    assert_eq!(examples.len(), 1);
    assert_eq!(
        examples[0].relative_path,
        std::path::Path::new("examples/a.fixture.rs")
    );
}

#[test]
fn refreshes_generated_imports_only_when_discovery_changes() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    let first_path = project.path().join("src/first.hblank.rs");
    let second_path = project.path().join("src/nested/second.hblank.rs");
    write(&first_path, "// first\n");

    let first = refresh_generated_examples(project.path(), &Config::default())
        .expect("initial generation should succeed");
    let unchanged = refresh_generated_examples(project.path(), &Config::default())
        .expect("unchanged generation should succeed");
    write(&second_path, "// second\n");
    let added = refresh_generated_examples(project.path(), &Config::default())
        .expect("added example should regenerate");
    fs::remove_file(&first_path).expect("first example should be removed");
    let removed = refresh_generated_examples(project.path(), &Config::default())
        .expect("removed example should regenerate");

    assert!(first.changed);
    assert!(!unchanged.changed);
    assert!(added.changed);
    assert_eq!(added.examples.len(), 2);
    assert!(removed.changed);
    assert_eq!(removed.examples.len(), 1);

    let generated = fs::read_to_string(project.path().join(".hblank/generated/examples.rs"))
        .expect("generated source should exist");
    assert!(generated.contains("second.hblank.rs"));
    assert!(!generated.contains("first.hblank.rs"));
    assert!(generated.contains("#[path = "));
}

#[test]
fn duplicate_file_stems_receive_distinct_module_names() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write(&project.path().join("src/a/card.hblank.rs"), "// a\n");
    write(&project.path().join("src/b/card.hblank.rs"), "// b\n");

    let examples =
        discover_examples(project.path(), &Config::default()).expect("discovery should succeed");

    assert_eq!(examples.len(), 2);
    assert_ne!(examples[0].module_name, examples[1].module_name);
}

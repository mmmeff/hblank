<div align="center">

<img src="assets/logo/hblank-mark.svg" alt="Hblank logo" width="128" height="128">

# hblank

**Develop GPUI components in isolation.**

[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-202124?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![GPUI 0.2.2](https://img.shields.io/badge/GPUI-0.2.2-7357d8?style=flat-square)](https://gpui.rs/)
[![Status: pre-1.0](https://img.shields.io/badge/status-pre--1.0-258b63?style=flat-square)](#project-status)
<p><a href="docs/README.md">docs</a></p>

</div>

Building a component should not require booting the whole application, loading test data, and clicking through three screens to reach the state you care about.

Hblank gives each GPUI component a small Rust fixture file. Run one command to browse its states, edit real props, read its docs, and check changes in a native GPUI window.

![Hblank's component catalog with an isolated preview and generated property controls](assets/hblank-harness.svg)

## Why Hblank

Most component work happens in states that the full app makes awkward to reach. Empty lists. Long labels. Disabled buttons. One exact error. Hblank puts those states one click away.

- Work on a component without starting its parent screen or backend.
- Change typed Rust props through generated controls and see the result at once.
- Save a Rust file and let Hblank rebuild the preview. A bad build leaves the last good window open.
- Keep examples, docs, and tests beside the component instead of rebuilding them in a browser tool.

There is no JavaScript layer and no second JSON version of your props. Hblank runs the same GPUI code your application runs.

## Add Hblank to a project

Run these commands from the existing GPUI project:

```bash
cargo install hblank-cli
cargo add hblank
hblank init
hblank dev
```

`hblank init` creates a private preview crate under `.hblank/`. It leaves your host manifest and source files alone.

Read [Getting started](docs/getting-started.md) to add the first fixture to an existing project.

## Updating hblank

Update the **CLI and both runtime dependencies**: your host package's `Cargo.toml` and its separate `.hblank/Cargo.toml`. Updating the CLI alone does not update the UI.

Stop `hblank dev`, choose a [published release](https://github.com/mmmeff/hblank/releases), and replace `VERSION` below with its version number (without `v`). Run from the package containing `.hblank/`:

```bash
cargo install hblank-cli --version '=VERSION' --locked
cargo add 'hblank@=VERSION'
cargo add --manifest-path .hblank/Cargo.toml 'hblank@=VERSION' --features test-support
hblank --version
hblank test
hblank dev
```

For workspace-inherited dependencies, update the owning `[workspace.dependencies]` entry too. Keep both runtime dependencies on the same version **and source**, preserve project-specific features, and commit the changed manifests and tracked lockfiles. `cargo update` alone cannot change exact pins or escape a manifest's version range (for example, `"0.5"` does not allow `0.6`). Do not rerun `hblank init`; it refuses to overwrite existing setup. Check [migration notes](docs/migration-0.3.md) and [GPUI compatibility](docs/crates.md) when upgrading older projects.

**Using unreleased changes:** a Git push is not a crates.io release. Either wait for publishing or point both runtime dependencies at the same Git revision/local checkout. For Git, update both `rev` pins and refresh both lockfiles with `cargo update -p hblank` and `cargo update --manifest-path .hblank/Cargo.toml -p hblank` (these also update tracked branches). Reinstall the CLI with `cargo install --git https://github.com/mmmeff/hblank --rev REV hblank-cli --locked --force`. For local `path` dependencies, pull the checkout and reinstall with `cargo install --path /path/to/hblank/crates/hblank-cli --locked --force`. Restart `hblank dev` to rebuild; no reinitialization is needed.

## Define the states that matter

Hblank turns ordinary Rust props into controls:

```rust
use hblank::{HblankEnum, HblankProps};

#[derive(Clone, Copy, Default, HblankEnum)]
pub enum Tone {
    #[default]
    Neutral,
    Warning,
}

#[derive(Clone, HblankProps)]
pub struct BadgeProps {
    /// Text rendered inside the badge.
    pub label: String,
    /// Semantic color treatment.
    pub tone: Tone,
}
```

A matched `*.hblank.rs` file registers the component and its named states:

```rust
use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{BadgeProps, Tone, badge};

#[hblank::component(title = "Badge", group = "Components")]
/// A compact status badge.
fn badge_component(
    props: &BadgeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    badge(props, window, cx)
}

#[hblank::fixture(component = badge_component, title = "Default")]
fn badge_default() -> BadgeProps {
    BadgeProps::default()
}

#[hblank::fixture(component = badge_component, title = "Warning")]
fn badge_warning() -> BadgeProps {
    BadgeProps {
        tone: Tone::Warning,
        ..BadgeProps::default()
    }
}
```

The catalog shows **Badge** once, with **Default** and **Warning** nested beneath it. Controls edit the same `BadgeProps` values passed to the production render function.

The [component authoring guide](docs/authoring.md) covers controls, adapters, documentation pages, themes, and multiple variants.

## Documentation

- [Getting started](docs/getting-started.md) installs Hblank and walks through the first fixture.
- [Components and fixtures](docs/authoring.md) covers props, controls, variants, Rustdoc, and custom documentation.
- [CLI reference](docs/cli.md) documents every command, path rule, and reload behavior.
- [Testing components](docs/testing.md) covers inline tests, GPUI test contexts, and typed handles.
- [Crates and GPUI backends](docs/crates.md) explains the four crates and the GPUI backend.
- [Troubleshooting](docs/troubleshooting.md) starts with the common discovery, build, control, theme, and test failures.
- [Migrating to 0.3](docs/migration-0.3.md) moves old fixtures to the component and variant model.
- [Releasing](docs/releasing.md) documents the semantic-release and crates.io publishing setup.

## Agent skill

The repository includes an `hblank` skill for coding agents:

```bash
npx skills add mmmeff/hblank
```

The skill teaches the same component model and CLI workflow as the guides, then requires the agent to check the running GPUI window.

## Project status

Hblank is pre-1.0 and targets the [`gpui` 0.2.2 package published on crates.io](https://crates.io/crates/gpui/0.2.2). It does not target a Zed release tag or the GPUI crate on Zed's `main` branch. Those sources can expose different APIs while they declare the same GPUI version. The GPUI adapter works end to end today. `hblank-core` keeps the catalog, controls, docs, and theme types independent of GPUI so other Rust UI frameworks can add their own adapters later.

APIs may change before 1.0. The dogfood project is the compatibility check: Hblank must be able to build and inspect its own components.

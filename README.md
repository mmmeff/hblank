<div align="center">

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

## Try it

Clone the repository and open the dogfood project:

```bash
git clone https://github.com/mmmeff/hblank.git
cd hblank
cargo run -p hblank-cli -- dev --project fixtures/dogfood
```

The window contains the fixture card and the components used to build Hblank itself. Change a label in `fixtures/dogfood/src/lib.rs`, save it, and watch the preview reload.

To open one exact fixture:

```bash
cargo run -p hblank-cli -- dev --project fixtures/dogfood --fixture-id 'src/fixture_card.hblank.rs#fixture_card_default'
```

## Add Hblank to a project

`main` is at 0.3.0, but crates.io does not yet have the complete four-crate release. Install from a checkout for now:

```bash
# In the Hblank checkout
cargo install --path crates/hblank-cli

# In your GPUI project
cargo add hblank --path /path/to/hblank/crates/hblank
hblank init
hblank dev
```

`hblank init` creates a private preview crate under `.hblank/`. It leaves your host manifest and source files alone.

Read [Getting started](docs/getting-started.md) to add the first fixture to an existing project.

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
- [Crates and GPUI backends](docs/crates.md) explains the four crates and backend feature selection.
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

Hblank is pre-1.0 and targets GPUI 0.2.2. The GPUI adapter works end to end today. `hblank-core` keeps the catalog, controls, docs, and theme types independent of GPUI so other Rust UI frameworks can add their own adapters later.

APIs may change before 1.0. The dogfood project is the compatibility check: Hblank must be able to build and inspect its own components.

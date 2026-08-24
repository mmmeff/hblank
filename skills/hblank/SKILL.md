---
name: hblank
description: Build, run, debug, and iterate Rust GPUI components with Hblank. Use when an agent works in a project containing .hblank/config.toml, edits *.hblank.rs fixtures, derives HblankProps or HblankEnum, runs hblank init or hblank dev, opens a fixture by path, or maintains the Hblank framework itself.
---

# Hblank

Hblank is the agent skill for component development with the Hblank framework. The crates and executable are also named `hblank`; use repository and `--help` evidence for exact commands.

## Outcome

Deliver a real GPUI component and fixture that:

- compiles in the host project and Hblank preview crate;
- appears through configured discovery;
- renders from typed presentation props;
- exposes every intended control;
- displays Rustdoc in the harness;
- updates through the running development loop;
- remains independently testable and presentational.

## Workflow

### 1. Ground in the project

Read, in order:

1. the host `Cargo.toml` and component module;
2. `.hblank/config.toml` when it exists;
3. one existing `*.hblank.rs` fixture;
4. current command help:

```bash
hblank --help
hblank dev --help
```

Determine the project root, configured fixture file globs, ignored paths, Hblank dependency source, component interface, and local naming conventions. Do not add a second fixture convention beside an existing one.

**Complete when:** the exact host component, props type, matched fixture path, and verification command are known.

### 2. Initialize only when needed

If `.hblank/config.toml` is absent, initialize from the host package root:

```bash
hblank init
```

For a package elsewhere:

```bash
hblank init --project crates/ui
```

Initialization refuses to overwrite existing Hblank files. Never delete or replace a partial `.hblank/` directory to force initialization; inspect and reconcile it deliberately.

The host crate needs the `hblank` dependency when its source types derive `HblankProps` or `HblankEnum`. Reuse the workspace's existing version, Git source, or path source. Do not guess a dependency source.

**Complete when:** `.hblank/config.toml`, the preview manifest, preview entry point, and generated directory exist and parse.

### 3. Keep the component presentational

A component accepts presentation data and returns GPUI elements. It must not own application state, filesystem access, watchers, processes, networking, or global mutation.

Use application/container modules for state and side effects. Pass values and callbacks into presentational functions.

Props owned by a component must implement:

- `Clone`;
- `HblankProps`;
- `Send + 'static` through their field types.

Supported automatic controls:

| Rust type | Harness control |
|---|---|
| `bool` | toggle |
| `String` | editable text |
| integer or float | numeric stepper |
| unit enum deriving `HblankEnum` | option buttons |

Put user-facing explanations in field doc comments. If production props contain unsupported or stateful fields, define a fixture-only presentation props type from supported fields and map it into the production props. Do not weaken the production model for the harness.

Read [references/component-fixtures.md](references/component-fixtures.md) before creating a new component or fixture.

**Complete when:** changing each derived value produces a valid production component render without hidden state.

### 4. Register a component and fixture variants

Create a source file matched by `.hblank/config.toml`; the default is `src/**/*.hblank.rs`. A component owns the typed renderer and metadata. Each fixture variant takes no arguments and returns that component's props.

```rust
#[hblank::component(title = "Badge", group = "Components")]
/// Documentation shown automatically in the Hblank Docs panel.
fn badge_fixture(
    props: &BadgeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    badge(props, window, cx)
}

#[hblank::fixture(component = badge_fixture, title = "Default")]
fn badge_default() -> BadgeProps {
    BadgeProps::default()
}
```

Rules:

- import host types through `hblank_project` inside fixture files;
- canonical component and fixture ids derive from project-relative path plus function symbol; never add a second id source;
- use “component” and “fixture,” not “story,” in project terminology;
- use component Rustdoc for the Docs panel, fixture Rustdoc for variant notes, and field Rustdoc for control help;
- multiple components and variants per file are supported and sorted by group, component, variant, then canonical id;
- never edit `.hblank/generated/fixtures.rs` manually.

**Complete when:** discovery imports the file and the registry contains the expected component, variants, canonical ids, docs, and controls.

### 5. Run the exact fixture

Launch directly into the file being developed:

```bash
hblank dev --fixture src/badge.hblank.rs
```

With a separate project root:

```bash
hblank dev --project crates/ui --fixture src/badge.hblank.rs
```

Relative fixture paths resolve from `--project`; absolute paths also work. The file must match configured discovery. If it contains multiple fixtures, the first in deterministic navigation order opens.

Keep `hblank dev` running while editing. A successful build replaces the preview automatically and preserves the user's latest selection. A failed build leaves the last successful preview open; fix the compiler error rather than restarting repeatedly.

**Complete when:** the actual GPUI window opens on the requested fixture and the selected id is visible in navigation.

### 6. Iterate through observable states

In the running harness:

1. exercise every boolean, text, numeric, and enum control;
2. verify the isolated component visibly changes after each control action;
3. open Docs and verify function Rustdoc plus source location;
4. edit a visible source label or treatment and save;
5. verify automatic rebuild and the new render without restarting `hblank dev`;
6. test empty, long-content, disabled, and semantic variants relevant to the component.

Do not treat a successful compile as UI verification. Inspect the actual GPUI surface.

**Complete when:** every intended state is reachable through controls and the source-edit loop is visibly proven.

### 7. Verify before delivery

Run the repository's own gates. In this framework repository, the baseline is:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Also run the narrow command that builds the changed preview and exercise the fixture in the GPUI harness. Keep generated files, host tests, docs, and fixture registration consistent.

**Complete when:** command gates pass and direct harness evidence covers discovery, controls, docs, navigation, and reload behavior.

## Hard rules

- Never hand-edit generated imports or preview target output.
- Never hide an unsupported prop; use a deliberate fixture adapter.
- Never move component state into a presentational function to make a fixture easier.
- Never use unsafe dynamic-library loading for reload; Hblank uses supervised Rust rebuilds.
- Never claim hot reload from compilation alone; observe the updated GPUI render.
- Never replace persisted selection on every reload; `--fixture` controls initial launch only.
- Preserve existing project fixture file patterns and terminology.

## Reference routing

- Read [references/framework.md](references/framework.md) when changing configuration, discovery, macros, registration, preview startup, or hot reload.
- Read [references/component-fixtures.md](references/component-fixtures.md) when creating props, enums, components, fixture adapters, or fixture functions.
- Read [references/troubleshooting.md](references/troubleshooting.md) when discovery, compilation, controls, docs, direct fixture launch, or reload fails.


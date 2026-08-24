---
name: hblank
description: Build, run, test, debug, and migrate Rust GPUI components with Hblank. Use for projects containing .hblank/config.toml; editing *.hblank.rs files; authoring HblankProps, HblankEnum, components, fixture variants, DocPage content, theme hooks, or typed test handles; running hblank init, dev, list, or test; or maintaining Hblank itself.
---

# Hblank

Hblank is the agent skill for component development with the Hblank framework. The crates and executable are also named `hblank`; use repository and `--help` evidence for exact commands.

## Outcome

Deliver a real GPUI component and fixture that:

- compiles in the host project and Hblank preview crate;
- appears through configured discovery;
- renders from typed presentation props;
- exposes every intended control;
- surfaces generated or explicitly authored component documentation in the harness;
- updates through the running development loop;
- remains independently testable and presentational.

## Workflow

### 1. Ground in the project

Read, in order:

1. the host `Cargo.toml` and component module;
2. `.hblank/config.toml` and `.hblank/Cargo.toml` when they exist;
3. one existing `*.hblank.rs` fixture;
4. current command help:

```bash
hblank --help
hblank init --help
hblank dev --help
hblank list --help
hblank test --help
```

Determine the project root, configured fixture globs, ignored paths, Hblank dependency source, selected GPUI backend, component interface generation, and local naming conventions. Do not add a second fixture convention beside an existing one.

If existing fixtures use the removed `#[hblank::fixture(id, title, group)] fn(&Props, Window, App)` interface, treat the work as a clean component-first migration: move rendering to `#[hblank::component]`, create zero-argument fixture factories for named variants, remove explicit ids, and migrate every fixture in scope before adding new ones.

**Complete when:** the exact host component, props type, matched fixture path, GPUI backend, API generation, and verification command are known.

### 2. Initialize only when needed

If `.hblank/config.toml` is absent, initialize from the host package root:

```bash
hblank init
```

For a package elsewhere:

```bash
hblank init --project crates/ui
```

When developing against a local Hblank checkout, point the preview dependency at that runtime during initialization:

```bash
hblank init --project crates/ui
```

Initialization refuses to overwrite existing Hblank files. Never delete or replace a partial `.hblank/` directory to force initialization; inspect and reconcile it deliberately.

The host crate needs the `hblank` dependency when its source types derive `HblankProps` or `HblankEnum`. Reuse the workspace's existing version, Git source, or path source; never guess one.

Fresh preview manifests use GPUI 0.2.2 from crates.io. Read [references/framework.md](references/framework.md#gpui-backend) before changing preview dependencies.

**Complete when:** `.hblank/config.toml`, `.hblank/Cargo.toml`, `.hblank/src/main.rs`, `.hblank/generated/fixtures.rs`, and `.hblank/.gitignore` exist and parse, and the preview builds against GPUI 0.2.2.

### 3. Keep the production component boundary

Model fixture controls as presentation inputs. Keep filesystem access, watchers, processes, networking, and unrelated global mutation in application/container modules; pass values and callbacks into the component.

Do not flatten a legitimately stateful production component into fixture-only primitives. Preserve its production entity or handle, and expose a typed `Rendered<Handle, _>` only when tests need inspectable project state.

Props owned by a component must implement:

- `Clone`;
- `HblankProps`;
- `Send + 'static` through their field types.

Supported automatic controls:

| Rust type | Harness control |
|---|---|
| `bool` | toggle |
| `String` | direct single-line editor; `#[hblank(multiline)]` enables multiline |
| integer or float | direct editor and configured stepper; optional `min`, `max`, `step` |
| unit enum deriving `HblankEnum` | chips for small enums, list for larger enums |

Use `#[hblank(skip)]` for props fields that must not become controls. For a project domain newtype that maps cleanly to a built-in bool, string, number, or enum carrier, implement `HblankControlAdapter<T>` and annotate the field with `#[hblank(adapter = AdapterType)]`; do not duplicate the domain as fixture-only primitive state. Put user-facing explanations in field doc comments. For unsupported stateful/resource fields, use `skip` or a focused fixture props type. Do not weaken the production model for the harness.

Read [references/component-fixtures.md](references/component-fixtures.md) before creating a new component, fixture, or typed `DocPage`. Prefer generated Rustdoc/props/controls/source blocks; author a page only when block ordering, embedded variants, or callouts materially improve the catalog.

**Complete when:** changing each derived value produces a valid production render without fixture-only behavior leaking into production code.

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

**Complete when:** `hblank list` contains the expected component, nested variants, canonical ids, docs metadata, and controls.

### 5. Run the exact fixture

List canonical ids, then launch either the first variant in a source file or one exact variant:

```bash
hblank list --project .
hblank dev --fixture src/badge.hblank.rs
hblank dev --fixture-id 'src/badge.hblank.rs#badge_warning'
```

With a separate project root:

```bash
hblank dev --project crates/ui --fixture src/badge.hblank.rs
```

Relative fixture paths resolve from `--project`; absolute paths also work. Canonical ids are project-relative `path#function` values emitted by `hblank list`. `--fixture-id` is strict and fails before launch when no registration matches; do not hand-invent or alias ids.

Keep `hblank dev` running while editing. Verify System/Light/Dark from the toolbar when the component is theme-sensitive; when `theme_hook` is configured, confirm both harness chrome and the production component switch. A successful build replaces the preview automatically and preserves the latest selection, theme mode, and non-default control values for that command's session. Starting a new `hblank dev` intentionally returns controls to source-defined fixture defaults. A failed build leaves the last successful preview open; fix the compiler error rather than restarting repeatedly.

**Complete when:** the actual GPUI window opens with the requested component and variant selected, using the canonical id emitted by `hblank list`.

### 6. Iterate through observable states

In the running harness:

1. exercise every boolean, text, numeric, and enum control;
2. verify the isolated component visibly changes after each control action;
3. open Docs and verify the generated Rustdoc page or every explicitly declared `DocPage` block, including source context where requested;
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

Run `hblank test --project PATH` when fixture files contain explicit inline `#[test]` or `#[gpui::test]` modules; use `--filter` for one test name. Hblank regenerates the preview target and delegates execution/output to Cargo—it does not auto-test every fixture. For stateful behavior, prefer `Rendered<Handle, _>`, `render_handle!`, and `hblank::testing::draw_with_handle` so assertions retain typed project state rather than relying on coordinates or invented selectors. Also run the narrow command that builds the changed preview and exercise the fixture in the GPUI harness. Keep generated files, host tests, docs, and fixture registration consistent.

**Complete when:** command gates pass and direct harness evidence covers discovery, controls, docs, navigation, and reload behavior.

## Hard rules

- Never hand-edit generated imports or preview target output.
- Never silently omit an unsupported prop; choose an explicit `skip`, domain control adapter, or focused fixture props type.
- Never introduce or relocate production state merely to make a fixture easier.
- Never use unsafe dynamic-library loading for reload; Hblank uses supervised Rust rebuilds.
- Never claim hot reload from compilation alone; observe the updated GPUI render.
- Never replace persisted selection on every reload; `--fixture` controls initial launch only.
- Preserve existing project fixture file patterns and terminology.
- Never mix the removed pre-component fixture interface with component-first registrations; migrate cleanly.

## Reference routing

- Read [references/framework.md](references/framework.md) when changing configuration, discovery, GPUI backend selection, macros, registration, preview startup, themes, tests, or hot reload.
- Read [references/component-fixtures.md](references/component-fixtures.md) when creating or migrating props, enums, components, fixture adapters, variants, documentation, or typed handles.
- Read [references/troubleshooting.md](references/troubleshooting.md) when discovery, compilation, controls, docs, themes, direct fixture launch, tests, or reload fails.


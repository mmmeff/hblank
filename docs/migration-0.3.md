# Migrating to 0.3

[Documentation home](README.md) | [Project README](../README.md)

Hblank 0.3 replaces renderer-style fixtures with first-class components and zero-argument fixture variants.

The migration changes authoring code, canonical IDs, and older private preview manifests. Migrate the whole project in one pass. Keeping old and new fixture forms together leaves two incompatible models and makes direct-launch IDs unpredictable.

## What changed

Before 0.3, a fixture carried its own ID, title, group, renderer, and props:

```rust
#[hblank::fixture(
    id = "badge-warning",
    title = "Badge warning",
    group = "Components"
)]
fn badge_warning(
    props: &BadgeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    badge(props, window, cx)
}
```

In 0.3, the component owns rendering and shared metadata. Fixtures only return named starting props:

```rust
#[hblank::component(title = "Badge", group = "Components")]
fn badge_component(
    props: &BadgeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    badge(props, window, cx)
}

#[hblank::fixture(component = badge_component, title = "Warning")]
fn badge_warning() -> BadgeProps {
    BadgeProps {
        tone: Tone::Warning,
        ..BadgeProps::default()
    }
}
```

The catalog now shows one **Badge** component with a **Warning** variant beneath it.

## Step 1: move rendering to a component

For each old fixture renderer:

1. Add `#[hblank::component(title, group)]`.
2. Keep the `&Props`, `&mut Window`, and `&mut App` function signature.
3. Keep production rendering unchanged.
4. Move shared component Rustdoc to this function.

If several old fixtures rendered the same production component, create one Hblank component renderer and reuse it for every variant.

## Step 2: turn states into fixture factories

Create one zero-argument function per named state:

```rust
#[hblank::fixture(component = badge_component, title = "Default")]
fn badge_default() -> BadgeProps {
    BadgeProps::default()
}

#[hblank::fixture(component = badge_component, title = "Long label")]
fn badge_long_label() -> BadgeProps {
    BadgeProps {
        label: "A label long enough to exercise wrapping".to_owned(),
        ..BadgeProps::default()
    }
}
```

A fixture return type must match the component props type. The macro checks this at compile time.

Put notes about one state on its fixture function. Hblank adds those notes to the component docs when that variant is selected.

## Step 3: remove explicit IDs

Delete every old `id = "..."` attribute.

Hblank 0.3 derives IDs from the project-relative source path and Rust function name:

```text
src/badge.hblank.rs#badge_component
src/badge.hblank.rs#badge_default
src/badge.hblank.rs#badge_long_label
```

Run:

```bash
hblank list
```

Update any scripts, docs, editor tasks, or CI commands that stored an old fixture ID.

Use fixture records with `--fixture-id`. Component IDs identify catalog nodes but do not identify one renderable state.

## Step 4: check direct launch commands

A source-path launch still opens the first fixture in that file:

```bash
hblank dev --fixture src/badge.hblank.rs
```

For a stable exact state, use the new canonical ID:

```bash
hblank dev --fixture-id 'src/badge.hblank.rs#badge_long_label'
```

Do not rely on an old hand-written ID or add an alias layer.

## Step 5: update documentation

Generated docs now belong to the component.

- Component Rustdoc describes the component.
- Fixture Rustdoc describes one variant.
- Props field Rustdoc explains controls.

If the project needs a custom order or embedded examples, add `docs = function` to the component and return a typed `DocPage`.

```rust
#[hblank::component(
    title = "Badge",
    group = "Components",
    docs = badge_docs
)]
fn badge_component(...) -> impl IntoElement {
    // production render
}
```

Use `fixture_ref!` inside a `DocPage` instead of a string ID:

```rust
DocBlock::fixture(hblank::fixture_ref!(badge_long_label))
```

A fixture rename then fails at compile time until the docs reference is updated.

## Step 6: update the private preview

`hblank init` refuses to overwrite an existing `.hblank/` directory, so update old preview files by hand.

A crates.io GPUI preview needs both test-support features:

```toml
[dependencies]
gpui = { version = "0.2.2", features = ["test-support"] }
hblank = { path = "/path/to/hblank/crates/hblank", features = ["test-support"] }
hblank_project = { package = "your-package", path = ".." }
```

Use the same dependency source for `hblank` that the host package uses.

The preview entry point must re-export GPUI:

```rust
pub use hblank::gpui;
```

Keep the generated fixture module and main call:

```rust
mod fixtures {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/fixtures.rs"));
}

fn main() {
    hblank::run_harness();
}
```

The `.hblank/.gitignore` should contain:

```gitignore
target/
generated/
state.toml
```

## Step 7: move tests to the generated target

Tests inside discovered fixture files run through:

```bash
hblank test
```

Hblank 0.3 does not generate a test for each fixture. Keep explicit `#[test]` and `#[gpui::test]` functions for behavior the project cares about.

For stateful components, use a component `handle = Type`, return `Rendered<Handle, _>`, and render through `hblank::render_handle!` in a GPUI test. See [Testing components](testing.md).

## Migration checklist

- [ ] Every old renderer moved to `#[hblank::component]`.
- [ ] Every named state is a zero-argument `#[hblank::fixture]` factory.
- [ ] Old explicit IDs are gone.
- [ ] `hblank list` shows the expected component and fixture records.
- [ ] Direct-launch commands use the new fixture IDs.
- [ ] Component, fixture, and field Rustdoc sit on the right items.
- [ ] `.hblank/Cargo.toml` has current GPUI and Hblank test features.
- [ ] `.hblank/src/main.rs` re-exports `hblank::gpui`.
- [ ] `hblank test` passes explicit fixture tests.
- [ ] `hblank dev` opens the migrated fixture and reloads a visible source edit.

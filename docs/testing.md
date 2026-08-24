# Testing components

[Documentation home](README.md) | [Project README](../README.md)

Hblank runs tests inside the same private preview crate that compiles fixture files. You can test fixture data with ordinary Rust tests or render through GPUI's deterministic test context.

Hblank does not generate smoke tests for every fixture. It runs only the tests you write.

## Run fixture tests

Run every inline test in discovered fixture files:

```bash
hblank test
```

For another package:

```bash
hblank test --project crates/ui
```

Filter by Cargo test name:

```bash
hblank test --filter fixture_card_default_and_docs_are_explicit
```

The command regenerates fixture imports, then calls `cargo test` with `.hblank/Cargo.toml` and the private target directory.

## Test fixture data without GPUI

Put the test in the matched `*.hblank.rs` file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_fixture_has_actionable_copy() {
        let props = badge_warning();

        assert_eq!(props.tone, Tone::Warning);
        assert_eq!(props.label, "Payment failed");
    }
}
```

This is the cheapest useful test. It catches accidental fixture defaults and documentation assumptions without opening a window.

Typed docs are ordinary Rust values too:

```rust
#[test]
fn badge_docs_include_live_example_and_controls() {
    let page = badge_docs();

    assert_eq!(page.blocks().len(), 6);
}
```

Prefer assertions about behavior or required content. Do not pin a block count unless the exact page structure is part of the contract.

## Render a fixture in GPUI

Fresh `hblank init` output enables GPUI and Hblank `test-support` features in the private preview crate.

A GPUI test receives `hblank::testing::TestAppContext`:

```rust
#[gpui::test]
fn badge_draws(cx: &mut hblank::testing::TestAppContext) {
    let catalog = hblank::registered_catalog().expect("fixture registry is valid");
    let fixture = catalog
        .fixtures()
        .iter()
        .find(|fixture| {
            fixture.metadata().id == hblank::fixture_ref!(badge_warning)
        })
        .expect("warning fixture is registered");
    let cx = cx.add_empty_window();

    hblank::testing::draw_fixture(
        cx,
        fixture,
        size(px(480.0), px(320.0)),
    );
}
```

`registered_catalog` resolves the fixture. `draw_fixture` renders the supplied definition through its component adapter and asks GPUI to draw the window.

Use this when the test only needs to prove that real rendering succeeds.

## Expose a typed handle

A render can return a typed test handle alongside its GPUI element:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
struct FixtureCardHandle {
    label: String,
}

#[hblank::component(
    title = "Fixture card",
    group = "Dogfood",
    handle = FixtureCardHandle
)]
fn fixture_card_component(
    props: &FixtureCardProps,
    window: &mut Window,
    cx: &mut App,
) -> hblank::Rendered<FixtureCardHandle, impl IntoElement> {
    hblank::Rendered::new(
        fixture_card(props, window, cx),
        FixtureCardHandle {
            label: props.label.clone(),
        },
    )
}
```

`Rendered<Handle, Content>` still implements `IntoElement`, so production and catalog rendering use the content normally. The component macro also generates a typed render helper for tests.

Use `render_handle!` with `draw_with_handle`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn fixture_card_exposes_its_label(
        cx: &mut hblank::testing::TestAppContext,
    ) {
        let props = fixture_card_default();
        let cx = cx.add_empty_window();
        let handle = hblank::testing::draw_with_handle(
            cx,
            size(px(480.0), px(320.0)),
            |window, app| {
                hblank::render_handle!(
                    fixture_card_component,
                    &props,
                    window,
                    app
                )
            },
        );

        assert_eq!(handle.label, "Hot reload verified");
    }
}
```

A handle can contain a real project entity when the test needs state. Keep that entity as the assertion target instead of creating a second state model just for Hblank.

## Click known bounds

`hblank::testing::click_bounds` sends a primary click to known GPUI bounds. Use it when the component already exposes the bounds through a typed handle or test fixture.

```rust
hblank::testing::click_bounds(cx, button_bounds);
```

Hblank does not have a DOM, accessibility selector engine, or assertion language. GPUI currently does not expose the semantics tree needed for a reliable text or role selector API.

## Preview manifest requirements

A current `.hblank/Cargo.toml` needs:

- `test-support` on the direct `gpui` dependency
- `test-support` on `hblank`
- a direct dependency named `gpui`
- the same GPUI package identity used by the host and Hblank runtime

`.hblank/src/main.rs` must expose:

```rust
pub use hblank::gpui;
```

The `#[gpui::test]` macro resolves the crate by that name. Read [Crates and GPUI backends](crates.md) before changing GPUI sources or feature flags.

## What to test

Good fixture tests defend facts a future edit could break:

- a named variant has the intended defaults
- a component accepts long, empty, disabled, or error-state data
- a typed docs page includes a required example or warning
- a stateful render exposes the expected entity state
- a click changes project state through the real callback path

A test that only proves a macro expanded or a helper returned a value adds little. The compiler already checks that work.

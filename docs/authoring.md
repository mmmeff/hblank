# Components and fixtures

[Documentation home](README.md) | [Project README](../README.md)

Hblank has two authoring concepts.

A **component** registers one renderer, one props type, and its catalog metadata. A **fixture** returns a named starting value for that component's props.

The catalog shows a component once and nests its fixture variants underneath it.

## Props and generated controls

Derive `HblankProps` on a named-field struct:

```rust
use hblank::{HblankEnum, HblankProps};

#[derive(Clone, Copy, Default, HblankEnum)]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warning,
}

#[derive(Clone, HblankProps)]
pub struct BadgeProps {
    /// Uses the emphasized visual treatment.
    pub emphasized: bool,

    /// Text rendered inside the badge.
    #[hblank(multiline)]
    pub label: String,

    /// Number shown beside the label.
    #[hblank(min = 0, max = 10, step = 1)]
    pub count: u32,

    /// Semantic color treatment.
    pub tone: Tone,
}
```

Hblank uses field names as stable control IDs and field Rustdoc as help text.

| Rust type | Control |
|---|---|
| `bool` | Toggle |
| `String` | Single-line editor |
| `String` with `#[hblank(multiline)]` | Multiline editor |
| Integer or float | Direct editor and stepper |
| Unit enum deriving `HblankEnum` | Option chips for small enums, compact list for larger enums |

Numeric fields accept `min`, `max`, and positive `step` values. Hblank rejects non-finite numbers, fractions assigned to integers, values outside the Rust type, and values outside configured limits.

A unit enum can override its generated label:

```rust
#[derive(Clone, Copy, HblankEnum)]
pub enum Density {
    Compact,
    #[hblank(label = "High contrast")]
    HighContrast,
}
```

### Skipping a field

Use `#[hblank(skip)]` when the field must remain in the props type but should not appear in the inspector:

```rust
#[derive(Clone, HblankProps)]
pub struct AvatarProps {
    pub label: String,
    #[hblank(skip)]
    pub image_cache: ImageCache,
}
```

Skipping is explicit. Hblank does not silently drop unsupported fields.

### Adapting a domain type

A project newtype can use one of Hblank's built-in control values without changing the production model:

```rust
use hblank::{HblankControlAdapter, HblankProps};

#[derive(Clone, Default)]
pub struct Percentage(pub u8);

pub struct PercentageControl;

impl HblankControlAdapter<Percentage> for PercentageControl {
    type Value = u8;

    fn to_control(value: &Percentage) -> u8 {
        value.0
    }

    fn apply_control(value: &mut Percentage, control: u8) {
        value.0 = control;
    }
}

#[derive(Clone, Default, HblankProps)]
pub struct ProgressProps {
    #[hblank(adapter = PercentageControl, min = 0, max = 100, step = 5)]
    pub progress: Percentage,
}
```

The adapter only converts values. Hblank still owns the editor, validation, reset behavior, and saved values during the current development session.

### Using separate fixture props

Production props often contain callbacks, entities, caches, or resource handles that do not make useful controls. Keep a small fixture-only props type in the `*.hblank.rs` file and translate it when rendering:

```rust
use hblank::HblankProps;
use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{AccountCardProps, account_card};

#[derive(Clone, Default, HblankProps)]
struct AccountCardFixtureProps {
    /// Name displayed in the card.
    name: String,
    /// Shows the verified treatment.
    verified: bool,
}

#[hblank::component(title = "Account card", group = "Components")]
fn account_card_component(
    props: &AccountCardFixtureProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    account_card(
        &AccountCardProps {
            name: props.name.clone(),
            verified: props.verified,
            on_open: None,
        },
        window,
        cx,
    )
}
```

This adapter belongs to the fixture, not the production module. It should expose only values that affect the component's presentation.

## Registering a component

A component function receives its props, `Window`, and `App`. It returns any GPUI type that implements `IntoElement`:

```rust
#[hblank::component(
    title = "Badge",
    group = "Components",
    docs = badge_docs
)]
/// A compact semantic status badge.
fn badge_component(
    props: &BadgeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    badge(props, window, cx)
}
```

Supported component attributes are:

| Attribute | Purpose |
|---|---|
| `title` | Component name shown in the catalog |
| `group` | Top-level catalog group |
| `docs` | Optional function that returns a `DocPage` |
| `handle` | Optional typed handle exposed to component tests |

The canonical component ID is the project-relative source path plus the component function name.

## Registering variants

Each fixture takes no arguments and returns the component's props type:

```rust
#[hblank::fixture(component = badge_component, title = "Default")]
fn badge_default() -> BadgeProps {
    BadgeProps::default()
}

#[hblank::fixture(component = badge_component, title = "Warning")]
/// Warning state used when the user can take action.
fn badge_warning() -> BadgeProps {
    BadgeProps {
        tone: Tone::Warning,
        ..BadgeProps::default()
    }
}
```

The fixture macro checks the props type against the component at compile time. A fixture can only point at a registered component with the same props type.

Canonical fixture IDs also use source path plus function name:

```text
src/badge.hblank.rs#badge_default
src/badge.hblank.rs#badge_warning
```

Run `hblank list` instead of building IDs by hand.

A file can register several components and any number of variants. Hblank sorts the catalog by group, component title, variant title, and canonical ID.

## Generated documentation

If the component has no `docs` attribute, Hblank builds a Docs page with:

1. component Rustdoc
2. fixture-specific Rustdoc
3. generated props
4. live controls
5. captured component and fixture source

Props field Rustdoc becomes control help. Keep long production guidance on the production type or function. Use fixture Rustdoc for facts about that one state.

Hblank captures declaration tokens at compile time. The source block includes project-relative paths and normalized component and fixture declarations. It keeps doc comments but not original whitespace or ordinary comments.

## Typed documentation pages

Use `docs = function_name` when you need a different order, embedded variants, or a callout:

```rust
use hblank::{CalloutTone, DocBlock, DocPage};

fn badge_docs() -> DocPage {
    DocPage::new([
        DocBlock::heading(1, "Badge"),
        DocBlock::prose("Compact status for dense interfaces."),
        DocBlock::fixture(hblank::fixture_ref!(badge_warning)),
        DocBlock::props(),
        DocBlock::controls(),
        DocBlock::callout(
            CalloutTone::Note,
            "Usage",
            "Use warning tone only when the user can take action.",
        ),
        DocBlock::source(),
    ])
}
```

`fixture_ref!` resolves the fixture at compile time. Renaming the fixture breaks the build instead of leaving a stale string.

An explicit `DocPage` is the whole page. Add every block you want to display.

## Custom native documentation blocks

Use a custom block for a project-specific visualization that the built-in blocks cannot express:

```rust
use hblank::gpui::{IntoElement as _, div};

#[hblank::doc_block]
fn token_sample(
    context: &hblank::DocContext<'_>,
    payload: &str,
) -> hblank::gpui::AnyElement {
    div()
        .child(format!("{}: {payload}", context.component_title))
        .into_any_element()
}

fn button_docs() -> hblank::DocPage {
    hblank::DocPage::new([
        hblank::custom_doc!(token_sample, "spacing/medium"),
    ])
}
```

`DocContext` contains read-only component, fixture, selected theme, and resolved appearance data. A custom block cannot change controls, selection, navigation, or saved state.

## Project theme switching

Hblank's toolbar has System, Light, and Dark modes. System follows GPUI window appearance changes. Without a project hook, these modes change Hblank's UI but not the component under test.

Add a project hook when the component reads theme state from the host app:

```rust
#[hblank::theme_hook]
pub fn apply_hblank_theme(
    mode: hblank::ThemeMode,
    resolved: hblank::ResolvedTheme,
    cx: &mut hblank::gpui::App,
) {
    cx.set_global(ProjectTheme::from_hblank(mode, resolved));
}
```

Name the fully qualified function in `.hblank/config.toml`:

```toml
theme_hook = "my_app::apply_hblank_theme"
```

Hblank runs the hook when the preview starts, when the toolbar mode changes, and when System mode sees a new OS appearance.

## File organization

The default pattern is `src/**/*.hblank.rs`. Keep a fixture close to the component unless the project already has another convention.

Inside fixture files:

- Import production code through `hblank_project`.
- Import GPUI through `hblank::gpui` so the selected backend stays consistent.
- Use "component" and "fixture" in names and docs. Hblank does not use Storybook's "story" term.
- Never edit `.hblank/generated/fixtures.rs`. Hblank rewrites it from discovery.

See [Crates and GPUI backends](crates.md) if imports resolve to incompatible GPUI types.

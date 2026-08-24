# Component and fixture patterns

## Direct production props

Use this when production props consist only of supported presentation fields.

```rust
use hblank::{HblankEnum, HblankProps};

#[derive(Clone, Copy, Debug, Default, HblankEnum)]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warning,
}

#[derive(Clone, Debug, HblankProps)]
pub struct BadgeProps {
    /// Enables the emphasized visual treatment.
    pub emphasized: bool,
    /// Text displayed in the badge.
    #[hblank(multiline)]
    pub label: String,
    /// Number displayed beside the label.
    #[hblank(min = 0, max = 10, step = 1)]
    pub count: u32,
    /// Semantic color treatment.
    pub tone: Tone,
}

impl Default for BadgeProps {
    fn default() -> Self {
        Self {
            emphasized: true,
            label: "Ready".to_owned(),
            count: 3,
            tone: Tone::Neutral,
        }
    }
}
```

A state-free GPUI component:

```rust
use gpui::{App, Div, Window, div, prelude::*, px, rgb};

pub fn badge(props: &BadgeProps, _window: &mut Window, _cx: &mut App) -> Div {
    let (accent, background) = match props.tone {
        Tone::Neutral => (rgb(0x6f6f77), rgb(0xf1f1ee)),
        Tone::Success => (rgb(0x258b63), rgb(0xe8f6ef)),
        Tone::Warning => (rgb(0xc65d3b), rgb(0xffeee8)),
    };

    div()
        .flex()
        .items_center()
        .gap_2()
        .h(px(30.0))
        .px_3()
        .rounded_full()
        .border_1()
        .border_color(accent)
        .text_color(accent)
        .when(props.emphasized, |badge| badge.bg(background))
        .child(props.label.clone())
        .child(props.count.to_string())
}
```

The matched fixture file:

```rust
use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{BadgeProps, badge};

#[hblank::component(title = "Badge", group = "Components")]
/// A compact semantic status badge shown in isolation.
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

## Fixture adapter props

Use an adapter when production props contain callbacks, entities, resource handles, collections, optional domain objects, or other unsupported fields. Keep the adapter inside the `*.hblank.rs` file.

```rust
use hblank::HblankProps;
use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{AccountCardProps, account_card};

#[derive(Clone, Debug, HblankProps)]
struct AccountCardFixtureProps {
    /// Name shown in the card.
    name: String,
    /// Displays the verified treatment.
    verified: bool,
}

impl Default for AccountCardFixtureProps {
    fn default() -> Self {
        Self {
            name: "Ada Lovelace".to_owned(),
            verified: true,
        }
    }
}

#[hblank::component(title = "Account card", group = "Components")]
/// Account identity presentation without application state or network data.
fn account_card_fixture(
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

#[hblank::fixture(component = account_card_fixture, title = "Default")]
fn account_card_default() -> AccountCardFixtureProps {
    AccountCardFixtureProps::default()
}
```

The adapter is not a second production model. It is a small fixture interface that selects presentation-relevant values and supplies inert callback/resource values explicitly.

## Domain control adapters

Map a project newtype to one built-in control carrier. Hblank retains the editor, validation, reset, and session-state implementation:

```rust
use hblank::{HblankControlAdapter, HblankProps};

#[derive(Clone, Default)]
struct Percentage(u8);

struct PercentageControl;

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
struct ProgressProps {
    #[hblank(adapter = PercentageControl, min = 0, max = 100, step = 5)]
    progress: Percentage,
}
```

Adapters convert values only. They do not render custom GPUI editors or bypass the built-in control validation contract.

## Typed component documentation

Attach a Rust function returning `DocPage` through the component attribute:

```rust
#[hblank::component(title = "Badge", group = "Components", docs = badge_docs)]
fn badge_component(
    props: &BadgeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    badge(props, window, cx)
}

fn badge_docs() -> hblank::DocPage {
    hblank::DocPage::new([
        hblank::DocBlock::heading(1, "Badge"),
        hblank::DocBlock::prose("Compact semantic status."),
        hblank::DocBlock::fixture(hblank::fixture_ref!(badge_default)),
        hblank::DocBlock::props(),
        hblank::DocBlock::controls(),
        hblank::DocBlock::source(),
    ])
}
```

`fixture_ref!` resolves through the generated fixture registration, so a renamed fixture fails at compile time instead of leaving a stale string. Components without a custom page receive generated Rustdoc, props, live controls, and source blocks. Source blocks contain normalized tokens captured by the component/fixture attribute macros plus project-relative path and line; doc comments survive tokenization, while original formatting and ordinary comments do not.

## Custom native doc blocks

Use a custom block only when the built-in heading, prose, fixture, props, controls, callout, and source blocks cannot express a project-specific visualization:

```rust
#[hblank::doc_block]
fn token_sample(context: &hblank::DocContext<'_>, payload: &str) -> hblank::gpui::AnyElement {
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

`DocContext` is read-only catalog metadata: component/fixture ids and titles, selected theme mode, and resolved appearance. The renderer returns one native GPUI element and receives no harness entity, control mutation interface, navigation callback, or persistence access.

## Multiple variants in one file

Register one `#[hblank::component]` renderer, then add any number of zero-argument `#[hblank::fixture(component = renderer, title = "…")]` factories returning the same props type. Canonical component and fixture ids derive from source path plus function symbol. The harness groups by component and nests variants in title/id order; source-path launch selects the first variant in that hierarchy.

## Typed render handles in tests

Add `handle = Type` only when tests need inspectable project state that cannot be asserted from fixture props. The renderer returns `Rendered<Handle, impl IntoElement>`; production catalog rendering erases the handle, while `render_handle!` calls the generated typed helper in inline tests.

Use `hblank::testing::draw_with_handle` with GPUI `TestAppContext::add_empty_window`. `draw_fixture` paints a registry fixture and `click_bounds` dispatches a primary click to known bounds. Keep typed entities/state as the assertion surface; do not invent text/role selectors over GPUI's missing semantics tree.

## Rustdoc placement

- Component function doc comments become the generated component catalog description.
- Fixture factory doc comments add state-specific notes for that variant.
- Props field doc comments explain controls.
- Production component docs remain on the production function/type.
- Do not duplicate long prose in Hblank-specific files when the production docs already answer the user’s question; fixture docs should explain the showcased state.







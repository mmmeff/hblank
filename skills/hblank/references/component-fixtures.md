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

## Multiple variants in one file

Register one `#[hblank::component]` renderer, then add any number of zero-argument `#[hblank::fixture(component = renderer, title = "…")]` factories returning the same props type. Canonical component and fixture ids derive from source path plus function symbol. The harness groups by component and nests variants in title/id order; source-path launch selects the first variant in that hierarchy.

## Rustdoc placement

- Component function doc comments become the generated component catalog description.
- Fixture factory doc comments add state-specific notes for that variant.
- Props field doc comments explain controls.
- Production component docs remain on the production function/type.
- Do not duplicate long prose in Hblank-specific files when the production docs already answer the user’s question; fixture docs should explain the showcased state.



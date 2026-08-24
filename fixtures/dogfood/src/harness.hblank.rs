#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::rc::Rc;

use hblank::gpui::{App, IntoElement, Window, div, prelude::*, px, rgb};
use hblank::harness::{
    CanvasProps, ControlsPanelProps, DocsPanelProps, EmptyStateProps, HeaderProps, InspectorTab,
    NavigationItem, NavigationProps, SearchProps, ToolbarProps, UiHandler, canvas, controls_panel,
    docs_panel, empty_state, header, navigation, search, toolbar,
};
use hblank::{HblankEnum, HblankProps};

fn noop<T: 'static>() -> UiHandler<T> {
    Rc::new(|_, _, _| {})
}

#[derive(Clone, Debug, HblankProps)]
struct HeaderFixtureProps {
    /// Project label shown next to the Hblank mark.
    project: String,
    /// Number of discovered fixtures.
    fixture_count: u32,
    /// Build and reload status.
    status: String,
}

impl Default for HeaderFixtureProps {
    fn default() -> Self {
        Self {
            project: "hblank-dogfood · Hblank".to_owned(),
            fixture_count: 9,
            status: "Ready".to_owned(),
        }
    }
}

#[hblank::fixture(id = "hblank.header", title = "Header", group = "Hblank UI")]
/// Hblank's compact project identity, fixture count, and live build status surface.
fn header_fixture(
    props: &HeaderFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    header(HeaderProps {
        project: props.project.clone().into(),
        fixture_count: props.fixture_count as usize,
        status: props.status.clone().into(),
    })
}

#[derive(Clone, Debug, HblankProps)]
struct SearchFixtureProps {
    /// Visible search query.
    query: String,
    /// Whether keyboard input is currently routed to search.
    active: bool,
}

impl Default for SearchFixtureProps {
    fn default() -> Self {
        Self {
            query: "button".to_owned(),
            active: true,
        }
    }
}

#[hblank::fixture(id = "hblank.search", title = "Search", group = "Hblank UI")]
/// The keyboard-focused fixture filter used above the navigation tree.
fn search_fixture(
    props: &SearchFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    search(
        SearchProps {
            query: props.query.clone().into(),
            active: props.active,
        },
        noop(),
    )
}

#[derive(Clone, Debug, HblankProps)]
struct NavigationFixtureProps {
    /// Query applied to groups and fixture titles.
    query: String,
}

impl Default for NavigationFixtureProps {
    fn default() -> Self {
        Self {
            query: String::new(),
        }
    }
}

#[hblank::fixture(id = "hblank.navigation", title = "Navigation", group = "Hblank UI")]
/// Grouped, filterable navigation for jumping between isolated component fixtures.
fn navigation_fixture(
    props: &NavigationFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    static ITEMS: &[NavigationItem] = &[
        NavigationItem {
            id: "components.button",
            title: "Button",
            group: "Components",
        },
        NavigationItem {
            id: "components.card",
            title: "Card",
            group: "Components",
        },
        NavigationItem {
            id: "patterns.empty-state",
            title: "Empty state",
            group: "Patterns",
        },
    ];
    let handler = noop();
    navigation(
        NavigationProps {
            items: ITEMS,
            selected: Some("components.button"),
            query: &props.query,
        },
        &handler,
    )
}

#[derive(Clone, Copy, Debug, Default, HblankEnum)]
enum FixtureTab {
    #[default]
    Controls,
    Docs,
}

#[derive(Clone, Debug, HblankProps)]
struct ToolbarFixtureProps {
    /// Selected fixture title.
    title: String,
    /// Inspector tab shown as active.
    tab: FixtureTab,
}

impl Default for ToolbarFixtureProps {
    fn default() -> Self {
        Self {
            title: "Fixture card".to_owned(),
            tab: FixtureTab::Controls,
        }
    }
}

#[hblank::fixture(id = "hblank.toolbar", title = "Toolbar", group = "Hblank UI")]
/// Selected-fixture context and inspector tab switcher.
fn toolbar_fixture(
    props: &ToolbarFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    toolbar(
        ToolbarProps {
            title: props.title.clone().into(),
            source: "src/fixture_card.hblank.rs:8".into(),
            active_tab: match props.tab {
                FixtureTab::Controls => InspectorTab::Controls,
                FixtureTab::Docs => InspectorTab::Docs,
            },
        },
        noop(),
    )
}

#[derive(Clone, Debug, HblankProps)]
struct CanvasFixtureProps {
    /// Small context label above the isolated surface.
    label: String,
}

impl Default for CanvasFixtureProps {
    fn default() -> Self {
        Self {
            label: "ISOLATED PREVIEW".to_owned(),
        }
    }
}

#[hblank::fixture(id = "hblank.canvas", title = "Canvas", group = "Hblank UI")]
/// The centered, scrollable surface that contains exactly one rendered fixture.
fn canvas_fixture(
    props: &CanvasFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    canvas(
        CanvasProps {
            label: props.label.clone().into(),
        },
        div()
            .w(px(180.0))
            .h(px(96.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .bg(rgb(0x7357d8))
            .text_color(rgb(0xffffff))
            .child("Live fixture")
            .into_any_element(),
    )
}

#[derive(Clone, Copy, Debug, Default, HblankEnum)]
enum ControlTone {
    #[default]
    Violet,
    Jade,
}

#[derive(Clone, Debug, HblankProps)]
struct ControlsFixtureProps {
    /// Boolean control fixture.
    active: bool,
    /// Text control fixture.
    label: String,
    /// Numeric control fixture.
    count: u32,
    /// Enum control fixture.
    tone: ControlTone,
}

impl Default for ControlsFixtureProps {
    fn default() -> Self {
        Self {
            active: true,
            label: "Editable".to_owned(),
            count: 4,
            tone: ControlTone::Violet,
        }
    }
}

#[hblank::fixture(
    id = "hblank.controls-panel",
    title = "Controls panel",
    group = "Hblank UI"
)]
/// Automatically generated property controls, including field-level Rustdoc.
fn controls_fixture(
    props: &ControlsFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    controls_panel(
        ControlsPanelProps {
            definitions: props.definitions(),
            props,
            editing_text: Some("label"),
        },
        noop(),
    )
}

#[derive(Clone, Debug, HblankProps)]
struct DocsFixtureProps {
    /// Documentation heading.
    title: String,
    /// Rustdoc body rendered by the inspector.
    docs: String,
}

impl Default for DocsFixtureProps {
    fn default() -> Self {
        Self {
            title: "Fixture card".to_owned(),
            docs: "A presentational GPUI card documented directly from its Rust doc comment."
                .to_owned(),
        }
    }
}

#[hblank::fixture(id = "hblank.docs-panel", title = "Docs panel", group = "Hblank UI")]
/// Rustdoc rendered beside the isolated component without duplicate documentation files.
fn docs_fixture(props: &DocsFixtureProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    docs_panel(DocsPanelProps {
        title: props.title.clone().into(),
        docs: props.docs.clone().into(),
        source: "src/fixture_card.hblank.rs:8".into(),
    })
}

#[derive(Clone, Debug, HblankProps)]
struct EmptyFixtureProps {
    /// Empty-state heading.
    title: String,
    /// Recovery instruction.
    body: String,
}

impl Default for EmptyFixtureProps {
    fn default() -> Self {
        Self {
            title: "No fixtures discovered".to_owned(),
            body: "Add a matching .hblank.rs file and save; Hblank will discover it automatically."
                .to_owned(),
        }
    }
}

#[hblank::fixture(id = "hblank.empty-state", title = "Empty state", group = "Hblank UI")]
/// Actionable first-run guidance shown when no configured fixtures are linked.
fn empty_state_fixture(
    props: &EmptyFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    empty_state(EmptyStateProps {
        title: props.title.clone().into(),
        body: props.body.clone().into(),
    })
}

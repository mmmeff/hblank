#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::rc::Rc;

use hblank::gpui::{App, IntoElement, Window, div, prelude::*, px, rgb};
use hblank::harness::{
    CanvasProps, ControlsPanelProps, DocsPanelProps, EmptyStateProps, HeaderProps, InspectorTab,
    NavigationComponent, NavigationProps, NavigationVariant, SearchProps, ToolbarProps, UiHandler, canvas, controls_panel,
    doc_prose, doc_source, docs_panel, empty_state, header, navigation, search, toolbar,
};
use hblank::{HblankEnum, HblankProps, ThemeMode};

fn noop<T: 'static>() -> UiHandler<T> {
    Rc::new(|_, _, _| {})
}

#[derive(Clone, Debug, HblankProps)]
struct HeaderFixtureProps {
    /// Project label shown next to the Hblank mark.
    project: String,
    /// Number of discovered components.
    component_count: u32,
    /// Number of discovered fixtures.
    fixture_count: u32,
    /// Build and reload status.
    status: String,
}

impl Default for HeaderFixtureProps {
    fn default() -> Self {
        Self {
            project: "hblank-dogfood · Hblank".to_owned(),
            component_count: 8,
            fixture_count: 10,
            status: "Ready".to_owned(),
        }
    }
}

#[hblank::component(title = "Header", group = "Hblank UI")]
/// Hblank's compact project identity, fixture count, and live build status surface.
fn header_fixture(
    props: &HeaderFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    header(HeaderProps {
        project: props.project.clone().into(),
        component_count: props.component_count as usize,
        fixture_count: props.fixture_count as usize,
        status: props.status.clone().into(),
    })
}

#[hblank::fixture(component = header_fixture, title = "Default")]
fn header_default() -> HeaderFixtureProps {
    HeaderFixtureProps::default()
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

#[hblank::component(title = "Search", group = "Hblank UI")]
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

#[hblank::fixture(component = search_fixture, title = "Default")]
fn search_default() -> SearchFixtureProps {
    SearchFixtureProps::default()
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

#[hblank::component(title = "Navigation", group = "Hblank UI")]
/// Grouped, filterable navigation for jumping between isolated component fixtures.
fn navigation_fixture(
    props: &NavigationFixtureProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let components = [
        NavigationComponent {
            id: "components.button".into(),
            title: "Button",
            group: "Components",
            variants: vec![
                NavigationVariant {
                    id: "components.button#default".into(),
                    title: "Default",
                },
                NavigationVariant {
                    id: "components.button#loading".into(),
                    title: "Loading",
                },
            ],
        },
        NavigationComponent {
            id: "components.card".into(),
            title: "Card",
            group: "Components",
            variants: vec![NavigationVariant {
                id: "components.card#default".into(),
                title: "Default",
            }],
        },
    ];
    let handler = noop();
    navigation(
        NavigationProps {
            components: &components,
            selected: Some("components.button#default"),
            query: &props.query,
        },
        &handler,
    )
}

#[hblank::fixture(component = navigation_fixture, title = "Default")]
fn navigation_default() -> NavigationFixtureProps {
    NavigationFixtureProps::default()
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

#[hblank::component(title = "Toolbar", group = "Hblank UI")]
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
            theme_mode: ThemeMode::System,
        },
        noop(),
    )
}

#[hblank::fixture(component = toolbar_fixture, title = "Default")]
fn toolbar_default() -> ToolbarFixtureProps {
    ToolbarFixtureProps::default()
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

#[hblank::component(title = "Canvas", group = "Hblank UI")]
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

#[hblank::fixture(component = canvas_fixture, title = "Default")]
fn canvas_default() -> CanvasFixtureProps {
    CanvasFixtureProps::default()
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

#[hblank::component(title = "Controls panel", group = "Hblank UI")]
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
            editing_number: None,
        },
        noop(),
    )
}

#[hblank::fixture(component = controls_fixture, title = "Default")]
fn controls_default() -> ControlsFixtureProps {
    ControlsFixtureProps::default()
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

#[hblank::component(title = "Docs panel", group = "Hblank UI")]
/// Rustdoc rendered beside the isolated component without duplicate documentation files.
fn docs_fixture(props: &DocsFixtureProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    docs_panel(DocsPanelProps {
        title: props.title.clone().into(),
        blocks: vec![
            doc_prose(props.docs.clone()),
            doc_source("src/fixture_card.hblank.rs:8"),
        ],
    })
}

#[hblank::fixture(component = docs_fixture, title = "Default")]
fn docs_default() -> DocsFixtureProps {
    DocsFixtureProps::default()
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

#[hblank::component(title = "Empty state", group = "Hblank UI")]
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

#[hblank::fixture(component = empty_state_fixture, title = "Default")]
fn empty_state_default() -> EmptyFixtureProps {
    EmptyFixtureProps::default()
}




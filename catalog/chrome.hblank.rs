use std::{collections::BTreeSet, rc::Rc};

use hblank::gpui::{
    App, Context, IntoElement, Render, SharedString, Window, div, prelude::*, rems,
};
use hblank::harness::{
    CanvasProps, EmptyStateProps, HeaderProps, InspectorTab, NavigationAction, NavigationComponent,
    NavigationProps, NavigationVariant, ToolbarAction, ToolbarProps, UiHandler, canvas, doc_prose,
    empty_state, header, navigation, toolbar,
};
use hblank::{HblankEnum, HblankProps, ThemeMode};

#[derive(Clone, Debug, HblankProps)]
struct HeaderFixtureProps {
    /// Project label beside the Hblank mark.
    project: String,
    /// Sample component count shown in the header.
    component_count: u32,
    /// Sample fixture count shown in the header.
    fixture_count: u32,
    /// Current build status.
    status: String,
}

impl Default for HeaderFixtureProps {
    fn default() -> Self {
        Self {
            project: "Hblank component catalog".to_owned(),
            component_count: 16,
            fixture_count: 19,
            status: "Ready".to_owned(),
        }
    }
}

#[hblank::component(title = "Header", group = "Layout")]
/// Project identity, component and fixture counts, and build status.
fn header_fixture(props: &HeaderFixtureProps, _: &mut Window, _: &mut App) -> impl IntoElement {
    div().w(rems(52.0)).child(header(HeaderProps {
        project: props.project.clone().into(),
        component_count: props.component_count as usize,
        fixture_count: props.fixture_count as usize,
        status: props.status.clone().into(),
    }))
}

#[hblank::fixture(component = header_fixture, title = "Default")]
fn header_default() -> HeaderFixtureProps {
    HeaderFixtureProps::default()
}

#[derive(Clone, Debug, Default, HblankProps)]
struct NavigationFixtureProps {
    /// Filter applied to group, component, and variant titles.
    query: String,
}

#[hblank::component(title = "Navigation", group = "Layout")]
/// Collapsible component groups and selectable fixture variants. Filter from the properties panel.
fn navigation_fixture(
    props: &NavigationFixtureProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let state = window.use_keyed_state("catalog-navigation", cx, |_, _| NavigationPreview {
        query: props.query.clone(),
        selected: "inputs.text-input#single-line".into(),
        collapsed_groups: BTreeSet::new(),
        components: vec![
            NavigationComponent {
                id: "layout.header".into(),
                title: "Header",
                group: "Layout",
                variants: vec![NavigationVariant {
                    id: "layout.header#default".into(),
                    title: "Default",
                }],
            },
            NavigationComponent {
                id: "inputs.text-input".into(),
                title: "Text input",
                group: "Inputs",
                variants: vec![
                    NavigationVariant {
                        id: "inputs.text-input#single-line".into(),
                        title: "Single line",
                    },
                    NavigationVariant {
                        id: "inputs.text-input#multiline".into(),
                        title: "Multiline",
                    },
                ],
            },
            NavigationComponent {
                id: "documentation.prose".into(),
                title: "Prose",
                group: "Documentation",
                variants: vec![NavigationVariant {
                    id: "documentation.prose#default".into(),
                    title: "Default",
                }],
            },
        ],
    });
    state.update(cx, |state, cx| {
        if state.query != props.query {
            state.query.clone_from(&props.query);
            cx.notify();
        }
    });
    state
}

#[hblank::fixture(component = navigation_fixture, title = "Default")]
fn navigation_default() -> NavigationFixtureProps {
    NavigationFixtureProps::default()
}

struct NavigationPreview {
    query: String,
    selected: SharedString,
    collapsed_groups: BTreeSet<String>,
    components: Vec<NavigationComponent>,
}

impl NavigationPreview {
    fn on_navigation(&mut self, action: &NavigationAction, _: &mut Window, cx: &mut Context<Self>) {
        match action {
            NavigationAction::Select { id } => self.selected = id.clone(),
            NavigationAction::ToggleGroup { group } => {
                if !self.collapsed_groups.remove(*group) {
                    self.collapsed_groups.insert((*group).to_owned());
                }
            }
        }
        cx.notify();
    }
}

impl Render for NavigationPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handler: UiHandler<NavigationAction> = Rc::new(cx.listener(Self::on_navigation));
        div()
            .w(rems(18.0))
            .h(rems(30.0))
            .flex()
            .flex_col()
            .child(navigation(
                NavigationProps {
                    components: &self.components,
                    selected: Some(self.selected.as_ref()),
                    query: &self.query,
                    collapsed_groups: &self.collapsed_groups,
                },
                &handler,
            ))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, HblankEnum)]
enum FixtureTab {
    #[default]
    Controls,
    Docs,
}

impl FixtureTab {
    fn inspector(self) -> InspectorTab {
        match self {
            Self::Controls => InspectorTab::Controls,
            Self::Docs => InspectorTab::Docs,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, HblankEnum)]
enum FixtureTheme {
    #[default]
    System,
    Light,
    Dark,
}

impl FixtureTheme {
    fn mode(self) -> ThemeMode {
        match self {
            Self::System => ThemeMode::System,
            Self::Light => ThemeMode::Light,
            Self::Dark => ThemeMode::Dark,
        }
    }
}

#[derive(Clone, Debug, HblankProps)]
struct ToolbarFixtureProps {
    /// Selected component and variant title.
    title: String,
    /// Source location displayed below the title.
    source: String,
    /// Selected inspector tab. Clicking a tab changes the local selection.
    tab: FixtureTab,
    /// Selected theme preference. The preview retains local theme-button selections.
    theme: FixtureTheme,
}

impl Default for ToolbarFixtureProps {
    fn default() -> Self {
        Self {
            title: "Text input · Single line".to_owned(),
            source: "catalog/inputs.hblank.rs".to_owned(),
            tab: FixtureTab::Controls,
            theme: FixtureTheme::System,
        }
    }
}

#[hblank::component(title = "Toolbar", group = "Layout")]
/// Selected-fixture context with interactive inspector tabs and theme preference buttons.
fn toolbar_fixture(
    props: &ToolbarFixtureProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let state = window.use_keyed_state("catalog-toolbar", cx, |_, _| ToolbarPreview {
        external: props.clone(),
        active_tab: props.tab.inspector(),
        theme_mode: props.theme.mode(),
    });
    state.update(cx, |state, cx| {
        let changed = state.external.title != props.title
            || state.external.source != props.source
            || state.external.tab != props.tab
            || state.external.theme != props.theme;
        if state.external.tab != props.tab {
            state.active_tab = props.tab.inspector();
        }
        if state.external.theme != props.theme {
            state.theme_mode = props.theme.mode();
        }
        if changed {
            state.external.clone_from(props);
            cx.notify();
        }
    });
    state
}

#[hblank::fixture(component = toolbar_fixture, title = "Default")]
fn toolbar_default() -> ToolbarFixtureProps {
    ToolbarFixtureProps::default()
}

struct ToolbarPreview {
    external: ToolbarFixtureProps,
    active_tab: InspectorTab,
    theme_mode: ThemeMode,
}

impl ToolbarPreview {
    fn on_toolbar(&mut self, action: &ToolbarAction, _: &mut Window, cx: &mut Context<Self>) {
        match action {
            ToolbarAction::ShowControls => self.active_tab = InspectorTab::Controls,
            ToolbarAction::ShowDocs => self.active_tab = InspectorTab::Docs,
            ToolbarAction::SetTheme(mode) => self.theme_mode = *mode,
        }
        cx.notify();
    }
}

impl Render for ToolbarPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(rems(52.0)).child(toolbar(
            ToolbarProps {
                title: self.external.title.clone().into(),
                source: self.external.source.clone().into(),
                active_tab: self.active_tab,
                theme_mode: self.theme_mode,
            },
            Rc::new(cx.listener(Self::on_toolbar)),
        ))
    }
}

#[derive(Clone, Debug, HblankProps)]
struct CanvasFixtureProps {
    /// Context label above the isolated surface.
    label: String,
    /// Prose rendered inside the canvas.
    text: String,
}

impl Default for CanvasFixtureProps {
    fn default() -> Self {
        Self {
            label: "ISOLATED PREVIEW".to_owned(),
            text: "Hblank centers one component inside a scrollable preview surface.".to_owned(),
        }
    }
}

#[hblank::component(title = "Canvas", group = "Layout")]
/// A labeled, scrollable canvas that centers its component preview.
fn canvas_fixture(props: &CanvasFixtureProps, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
        .w(rems(38.0))
        .h(rems(25.0))
        .flex()
        .flex_col()
        .child(canvas(
            CanvasProps {
                label: props.label.clone().into(),
            },
            div()
                .w(rems(18.0))
                .child(doc_prose(props.text.clone()))
                .into_any_element(),
        ))
}

#[hblank::fixture(component = canvas_fixture, title = "Default")]
fn canvas_default() -> CanvasFixtureProps {
    CanvasFixtureProps::default()
}

#[derive(Clone, Debug, HblankProps)]
struct EmptyFixtureProps {
    /// Empty-state heading.
    title: String,
    /// Instruction explaining how to populate the catalog.
    body: String,
}

impl Default for EmptyFixtureProps {
    fn default() -> Self {
        Self {
            title: "No fixtures configured".to_owned(),
            body: "Add a component and a fixture variant in a .hblank.rs file included by your project configuration.".to_owned(),
        }
    }
}

#[hblank::component(title = "Empty state", group = "Layout")]
/// First-run guidance for a catalog without configured fixtures.
fn empty_state_fixture(props: &EmptyFixtureProps, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
        .w(rems(36.0))
        .h(rems(25.0))
        .flex()
        .child(empty_state(EmptyStateProps {
            title: props.title.clone().into(),
            body: props.body.clone().into(),
        }))
}

#[hblank::fixture(component = empty_state_fixture, title = "Default")]
fn empty_state_default() -> EmptyFixtureProps {
    EmptyFixtureProps::default()
}

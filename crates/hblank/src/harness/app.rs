#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::gpui;
#[cfg(feature = "crates-io-gpui")]
use crate::gpui::Application;
use crate::gpui::{
    App, Bounds, Context, FocusHandle, IntoElement, KeyDownEvent, Modifiers, Render, SharedString,
    Subscription, Window, WindowAppearance, WindowBounds, WindowOptions, div, prelude::*, px, rems,
    rgb, size,
};
use serde::{Deserialize, Serialize};

use crate::{
    ControlKind, ControlValue, FixtureDefinition, ResolvedTheme, TextMode, ThemeHook, ThemeMode,
    registered_catalog, registered_theme_hook, render_fixture,
};

use super::components::{
    CanvasProps, ControlAction, ControlsPanelProps, DocsPanelProps, EmptyStateProps, HeaderProps,
    InspectorTab, NavigationAction, NavigationItem, NavigationProps, SearchAction, SearchProps,
    ToolbarAction, ToolbarProps, UiHandler, canvas, controls_panel, docs_panel, empty_state,
    header, navigation, search, theme, toolbar,
};

const DEFAULT_WIDTH: f32 = 1440.0;
const DEFAULT_HEIGHT: f32 = 900.0;
const BASE_REM_SIZE: f32 = 16.0;
const DEFAULT_UI_SCALE: f32 = 1.0;
const UI_SCALE_STEP: f32 = 0.1;
const MIN_UI_SCALE: f32 = 0.5;
const MAX_UI_SCALE: f32 = 2.0;

fn focus(handle: &FocusHandle, window: &mut Window, cx: &mut App) {
    #[cfg(feature = "zed-gpui")]
    handle.focus(window, cx);
    #[cfg(feature = "crates-io-gpui")]
    {
        let _ = cx;
        handle.focus(window);
    }
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct PersistedState {
    session: String,
    selected: Option<String>,
    filter: String,
    theme: ThemeMode,
    controls: BTreeMap<String, BTreeMap<String, ControlValue>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditingTarget {
    Search,
    TextControl(&'static str),
    NumberControl(&'static str),
}

struct HarnessApp {
    fixtures: Vec<FixtureDefinition>,
    navigation: Vec<NavigationItem>,
    selected: Option<usize>,
    filter: String,
    inspector: InspectorTab,
    editing: EditingTarget,
    number_draft: String,
    ui_scale: f32,
    focus_handle: FocusHandle,
    project: SharedString,
    status: SharedString,
    session: String,
    theme_mode: ThemeMode,
    theme_hook: Option<ThemeHook>,
    appearance_subscription: Option<Subscription>,
    state_path: PathBuf,
}

fn initial_selection<'a>(
    entries: impl Iterator<Item = (&'a str, &'a str)>,
    persisted_id: Option<&str>,
    fixture_source: Option<&str>,
) -> (Option<usize>, bool) {
    let mut first = None;
    let mut persisted = None;
    for (index, (id, source)) in entries.enumerate() {
        first.get_or_insert(index);
        if fixture_source == Some(source) {
            return (Some(index), true);
        }
        if persisted_id == Some(id) {
            persisted = Some(index);
        }
    }
    (persisted.or(first), false)
}

fn resolved_theme(mode: ThemeMode, appearance: WindowAppearance) -> ResolvedTheme {
    match mode {
        ThemeMode::Light => ResolvedTheme::Light,
        ThemeMode::Dark => ResolvedTheme::Dark,
        ThemeMode::System => match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => ResolvedTheme::Light,
            WindowAppearance::Dark | WindowAppearance::VibrantDark => ResolvedTheme::Dark,
        },
    }
}

fn restore_control_overrides(
    fixtures: &mut [FixtureDefinition],
    controls: &BTreeMap<String, BTreeMap<String, ControlValue>>,
) {
    for fixture in fixtures {
        let Some(values) = controls.get(fixture.metadata().id.as_str()) else {
            continue;
        };
        fixture.apply_control_values(values.iter().map(|(id, value)| (id.as_str(), value)));
    }
}

fn ui_scale_delta(key: &str, modifiers: Modifiers) -> Option<f32> {
    if !modifiers.platform || modifiers.control || modifiers.alt || modifiers.function {
        return None;
    }
    match key {
        "=" | "+" => Some(UI_SCALE_STEP),
        "-" => Some(-UI_SCALE_STEP),
        _ => None,
    }
}

fn bounded_ui_scale(current: f32, delta: f32) -> f32 {
    (current + delta).clamp(MIN_UI_SCALE, MAX_UI_SCALE)
}

impl HarnessApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state_path = state_path();
        let mut persisted = load_state(&state_path);
        let session = env::var("HBLANK_SESSION_ID")
            .unwrap_or_else(|_| format!("process-{}", std::process::id()));
        let (mut fixtures, mut status) = match registered_catalog() {
            Ok(catalog) => (catalog.into_parts().1, SharedString::from("Ready")),
            Err(error) => (Vec::new(), SharedString::from(error.to_string())),
        };
        let theme_mode = if persisted.session == session {
            restore_control_overrides(&mut fixtures, &persisted.controls);
            persisted.theme
        } else {
            persisted.controls.clear();
            ThemeMode::System
        };
        let theme_hook_name = env::var("HBLANK_THEME_HOOK").ok();
        let theme_hook = theme_hook_name.as_deref().and_then(registered_theme_hook);
        if let Some(name) = &theme_hook_name
            && theme_hook.is_none()
        {
            status = format!("Configured theme hook '{name}' is not registered").into();
        }
        let requested_fixture = env::var("HBLANK_INITIAL_FIXTURE").ok();
        let (selected, matched_fixture) = initial_selection(
            fixtures.iter().map(|fixture| {
                let metadata = fixture.metadata();
                (metadata.id.as_str(), metadata.source)
            }),
            persisted.selected.as_deref(),
            requested_fixture.as_deref(),
        );
        if requested_fixture.is_some() && !matched_fixture {
            status = "Requested fixture contains no registered fixtures".into();
        }
        let navigation = fixtures
            .iter()
            .map(|fixture| {
                let metadata = fixture.metadata();
                NavigationItem {
                    id: metadata.id.clone().into(),
                    title: metadata.title,
                    group: metadata.group,
                }
            })
            .collect::<Vec<_>>();
        let focus_handle = cx.focus_handle();
        focus(&focus_handle, window, cx);
        println!("Hblank harness ready: {} fixtures", fixtures.len());

        let mut app = Self {
            fixtures,
            navigation,
            selected,
            filter: if matched_fixture {
                String::new()
            } else {
                persisted.filter
            },
            inspector: InspectorTab::Controls,
            editing: EditingTarget::Search,
            number_draft: String::new(),
            ui_scale: DEFAULT_UI_SCALE,
            focus_handle,
            project: env::var("HBLANK_WINDOW_TITLE")
                .unwrap_or_else(|_| "GPUI project · Hblank".to_owned())
                .into(),
            status,
            session,
            theme_mode,
            theme_hook,
            appearance_subscription: None,
            state_path,
        };
        app.apply_theme(window, cx);
        app.appearance_subscription =
            Some(cx.observe_window_appearance(window, HarnessApp::apply_theme));
        if matched_fixture {
            app.persist();
        }
        app
    }

    fn selected_fixture(&self) -> Option<&FixtureDefinition> {
        self.selected.and_then(|index| self.fixtures.get(index))
    }

    fn selected_fixture_mut(&mut self) -> Option<&mut FixtureDefinition> {
        self.selected.and_then(|index| self.fixtures.get_mut(index))
    }

    fn selected_id(&self) -> Option<&str> {
        self.selected_fixture()
            .map(|fixture| fixture.metadata().id.as_str())
    }

    fn select(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(index) = self
            .fixtures
            .iter()
            .position(|fixture| fixture.metadata().id == id)
        {
            self.selected = Some(index);
            self.editing = EditingTarget::Search;
            self.number_draft.clear();
            self.status = SharedString::from("Ready");
            self.persist();
            cx.notify();
        }
    }

    fn navigate_filtered(&mut self, delta: isize, cx: &mut Context<Self>) {
        let query = self.filter.to_ascii_lowercase();
        let visible = self
            .navigation
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                query.is_empty()
                    || item.title.to_ascii_lowercase().contains(&query)
                    || item.group.to_ascii_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if visible.is_empty() {
            return;
        }
        let current = self
            .selected
            .and_then(|selected| visible.iter().position(|index| *index == selected))
            .unwrap_or(0);
        let next = if delta.is_negative() {
            current
                .checked_sub(delta.unsigned_abs())
                .unwrap_or(visible.len() - 1)
        } else {
            (current + delta.unsigned_abs()) % visible.len()
        };
        self.selected = Some(visible[next]);
        self.persist();
        cx.notify();
    }

    fn on_navigation(
        &mut self,
        action: &NavigationAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select(action.id.as_ref(), cx);
    }
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn on_search_focus(&mut self, _: &SearchAction, window: &mut Window, cx: &mut Context<Self>) {
        self.editing = EditingTarget::Search;
        self.number_draft.clear();
        focus(&self.focus_handle, window, cx);
        cx.notify();
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn on_toolbar(&mut self, action: &ToolbarAction, window: &mut Window, cx: &mut Context<Self>) {
        match action {
            ToolbarAction::ShowControls => self.inspector = InspectorTab::Controls,
            ToolbarAction::ShowDocs => self.inspector = InspectorTab::Docs,
            ToolbarAction::SetTheme(mode) => {
                self.theme_mode = *mode;
                self.apply_theme(window, cx);
                self.persist();
                return;
            }
        }
        cx.notify();
    }

    fn apply_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let resolved = resolved_theme(self.theme_mode, window.appearance());
        theme::set_dark(resolved == ResolvedTheme::Dark);
        if let Some(hook) = self.theme_hook {
            hook(self.theme_mode, resolved, cx);
        }
        cx.notify();
    }

    fn on_control(&mut self, action: &ControlAction, window: &mut Window, cx: &mut Context<Self>) {
        match action {
            ControlAction::Set { id, value } => {
                let result = self
                    .selected_fixture_mut()
                    .expect("a rendered control always has a selected fixture")
                    .set_control(id, value.clone());
                self.status = match result {
                    Ok(()) => SharedString::from("Ready"),
                    Err(error) => SharedString::from(error.to_string()),
                };
            }
            ControlAction::EditText { id } => {
                self.editing = EditingTarget::TextControl(id);
                self.number_draft.clear();
                focus(&self.focus_handle, window, cx);
            }
            ControlAction::EditNumber { id } => {
                self.editing = EditingTarget::NumberControl(id);
                self.number_draft.clear();
                self.status = "Type a number".into();
                focus(&self.focus_handle, window, cx);
            }
            ControlAction::Reset => {
                if let Some(fixture) = self.selected_fixture_mut() {
                    fixture.reset();
                }
                self.editing = EditingTarget::Search;
                self.number_draft.clear();
                self.status = SharedString::from("Ready");
            }
        }
        self.persist();
        cx.notify();
    }

    fn adjust_ui_scale(&mut self, delta: f32, cx: &mut Context<Self>) {
        let next = bounded_ui_scale(self.ui_scale, delta);
        if (next - self.ui_scale).abs() < f32::EPSILON {
            return;
        }
        self.ui_scale = next;
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(delta) = ui_scale_delta(event.keystroke.key.as_str(), event.keystroke.modifiers)
        {
            self.adjust_ui_scale(delta, cx);
            return;
        }

        match event.keystroke.key.as_str() {
            "up" if self.editing == EditingTarget::Search => {
                self.navigate_filtered(-1, cx);
                return;
            }
            "down" if self.editing == EditingTarget::Search => {
                self.navigate_filtered(1, cx);
                return;
            }
            "escape" => {
                if self.editing == EditingTarget::Search {
                    self.filter.clear();
                }
                self.editing = EditingTarget::Search;
                self.number_draft.clear();
                self.status = "Ready".into();
                self.persist();
                cx.notify();
                return;
            }
            "enter" => {
                if let EditingTarget::TextControl(id) = self.editing
                    && self.text_is_multiline(id)
                {
                    self.insert_text("\n");
                } else {
                    self.editing = EditingTarget::Search;
                    self.number_draft.clear();
                }
                self.persist();
                cx.notify();
                return;
            }
            "backspace" => {
                self.remove_character();
                self.persist();
                cx.notify();
                return;
            }
            _ => {}
        }

        if let Some(text) = event.keystroke.key_char.as_deref()
            && !text.chars().any(char::is_control)
        {
            self.insert_text(text);
            self.persist();
            cx.notify();
        }
    }

    fn remove_character(&mut self) {
        match self.editing {
            EditingTarget::Search => {
                self.filter.pop();
            }
            EditingTarget::TextControl(id) => {
                let Some(fixture) = self.selected_fixture_mut() else {
                    return;
                };
                let Some(ControlValue::Text(mut value)) = fixture.props().control_value(id) else {
                    return;
                };
                value.pop();
                let _ = fixture.set_control(id, ControlValue::Text(value));
            }
            EditingTarget::NumberControl(id) => {
                self.number_draft.pop();
                self.apply_number_draft(id);
            }
        }
    }

    fn insert_text(&mut self, text: &str) {
        match self.editing {
            EditingTarget::Search => self.filter.push_str(text),
            EditingTarget::TextControl(id) => {
                let Some(fixture) = self.selected_fixture_mut() else {
                    return;
                };
                let Some(ControlValue::Text(mut value)) = fixture.props().control_value(id) else {
                    return;
                };
                value.push_str(text);
                let _ = fixture.set_control(id, ControlValue::Text(value));
            }
            EditingTarget::NumberControl(id) => {
                if text
                    .chars()
                    .all(|character| character.is_ascii_digit() || ".-+eE".contains(character))
                {
                    self.number_draft.push_str(text);
                    self.apply_number_draft(id);
                }
            }
        }
    }

    fn apply_number_draft(&mut self, id: &'static str) {
        let Ok(value) = self.number_draft.parse::<f64>() else {
            self.status = "Enter a valid number".into();
            return;
        };
        let result = self
            .selected_fixture_mut()
            .expect("an edited control always has a selected fixture")
            .set_control(id, ControlValue::Number(value));
        self.status = match result {
            Ok(()) => "Ready".into(),
            Err(error) => error.to_string().into(),
        };
    }

    fn text_is_multiline(&self, id: &str) -> bool {
        self.selected_fixture().is_some_and(|fixture| {
            fixture.props().definitions().iter().any(|definition| {
                definition.id == id
                    && matches!(
                        definition.kind,
                        ControlKind::Text {
                            mode: TextMode::Multiline
                        }
                    )
            })
        })
    }

    fn persist(&self) {
        let controls = self
            .fixtures
            .iter()
            .filter_map(|fixture| {
                let values = fixture
                    .props()
                    .definitions()
                    .iter()
                    .filter_map(|definition| {
                        let current = fixture.props().control_value(definition.id)?;
                        (fixture.default_control_value(definition.id) != Some(current.clone()))
                            .then(|| (definition.id.to_owned(), current))
                    })
                    .collect::<BTreeMap<_, _>>();
                (!values.is_empty()).then(|| (fixture.metadata().id.clone(), values))
            })
            .collect();
        let state = PersistedState {
            session: self.session.clone(),
            selected: self.selected_id().map(str::to_owned),
            filter: self.filter.clone(),
            theme: self.theme_mode,
            controls,
        };
        let Ok(source) = toml::to_string(&state) else {
            return;
        };
        if let Some(parent) = self.state_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&self.state_path, source);
    }
    fn render_body(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        toolbar_handler: UiHandler<ToolbarAction>,
        control_handler: UiHandler<ControlAction>,
    ) -> gpui::AnyElement {
        let Some(index) = self.selected else {
            return empty_state(EmptyStateProps {
                title: "No fixtures discovered".into(),
                body: "Add a discovered file, define a #[hblank::component] renderer and a #[hblank::fixture] variant, then save. They will appear here automatically."
                    .into(),
            })
            .into_any_element();
        };
        let metadata = self.fixtures[index].metadata();
        let preview = render_fixture(&self.fixtures[index], window, cx);
        let toolbar_surface = toolbar(
            ToolbarProps {
                title: metadata.title.into(),
                source: source_label(metadata.source, metadata.line),
                active_tab: self.inspector,
                theme_mode: self.theme_mode,
            },
            toolbar_handler,
        );
        let inspector = match self.inspector {
            InspectorTab::Controls => controls_panel(
                ControlsPanelProps {
                    definitions: self.fixtures[index].props().definitions(),
                    props: self.fixtures[index].props(),
                    editing_text: match self.editing {
                        EditingTarget::TextControl(id) => Some(id),
                        EditingTarget::Search | EditingTarget::NumberControl(_) => None,
                    },
                    editing_number: match self.editing {
                        EditingTarget::NumberControl(id) => Some((id, self.number_draft.as_str())),
                        EditingTarget::Search | EditingTarget::TextControl(_) => None,
                    },
                },
                control_handler,
            )
            .into_any_element(),
            InspectorTab::Docs => docs_panel(DocsPanelProps {
                title: metadata.title.into(),
                docs: metadata.docs.into(),
                source: source_label(metadata.source, metadata.line),
            })
            .into_any_element(),
        };
        div()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .child(toolbar_surface)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(canvas(
                        CanvasProps {
                            label: "ISOLATED PREVIEW".into(),
                        },
                        preview,
                    ))
                    .child(inspector),
            )
            .into_any_element()
    }
}

impl Render for HarnessApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(BASE_REM_SIZE * self.ui_scale));
        let navigation_handler: UiHandler<NavigationAction> =
            Rc::new(cx.listener(Self::on_navigation));
        let search_handler: UiHandler<SearchAction> = Rc::new(cx.listener(Self::on_search_focus));
        let toolbar_handler: UiHandler<ToolbarAction> = Rc::new(cx.listener(Self::on_toolbar));
        let control_handler: UiHandler<ControlAction> = Rc::new(cx.listener(Self::on_control));
        let selected_id = self.selected_id();
        let search_surface = search(
            SearchProps {
                query: self.filter.clone().into(),
                active: self.editing == EditingTarget::Search,
            },
            search_handler,
        );
        let navigation_surface = navigation(
            NavigationProps {
                items: &self.navigation,
                selected: selected_id,
                query: &self.filter,
            },
            &navigation_handler,
        );

        let body = self.render_body(window, cx, toolbar_handler, control_handler);

        div()
            .id("hblank-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(theme::canvas()))
            .text_color(rgb(theme::text()))
            .child(header(HeaderProps {
                project: self.project.clone(),
                fixture_count: self.fixtures.len(),
                status: self.status.clone(),
            }))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(
                        div()
                            .w(rems(17.0))
                            .h_full()
                            .flex_none()
                            .flex()
                            .flex_col()
                            .bg(rgb(theme::sidebar()))
                            .child(search_surface)
                            .child(navigation_surface),
                    )
                    .child(body),
            )
    }
}

pub fn run_harness() {
    #[cfg(feature = "crates-io-gpui")]
    let application = Application::new();
    #[cfg(feature = "zed-gpui")]
    let application = gpui_platform_zed::application();

    application.run(|cx: &mut App| {
        #[cfg(feature = "crates-io-gpui")]
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        #[cfg(feature = "zed-gpui")]
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let width = env_dimension("HBLANK_WINDOW_WIDTH", DEFAULT_WIDTH);
        let height = env_dimension("HBLANK_WINDOW_HEIGHT", DEFAULT_HEIGHT);
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.0), px(600.0))),
                app_id: Some("hblank".to_owned()),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| HarnessApp::new(window, cx)),
        );
        if let Err(error) = result {
            eprintln!("Could not open the Hblank window: {error}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}

fn source_label(source: &str, line: u32) -> SharedString {
    let path = Path::new(source);
    let project_root = env::var_os("HBLANK_PROJECT_ROOT").map(PathBuf::from);
    let path = project_root
        .as_deref()
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path);
    format!("{}:{line}", path.display()).into()
}

fn state_path() -> PathBuf {
    env::var_os("HBLANK_PROJECT_ROOT").map_or_else(
        || PathBuf::from(".hblank/state.toml"),
        |root| PathBuf::from(root).join(".hblank/state.toml"),
    )
}

fn load_state(path: &PathBuf) -> PersistedState {
    fs::read_to_string(path)
        .ok()
        .and_then(|source| toml::from_str(&source).ok())
        .unwrap_or_default()
}

fn env_dimension(name: &str, fallback: f32) -> f32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite() && *value >= 1.0)
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        ControlValue, ResolvedTheme, ThemeMode,
        gpui::{Modifiers, WindowAppearance},
    };

    use super::{
        MAX_UI_SCALE, MIN_UI_SCALE, PersistedState, UI_SCALE_STEP, bounded_ui_scale, env_dimension,
        initial_selection, resolved_theme, ui_scale_delta,
    };

    #[test]
    fn persisted_state_round_trips() {
        let state = PersistedState {
            session: "dev-session".to_owned(),
            selected: Some("components::button".to_owned()),
            filter: "button".to_owned(),
            theme: ThemeMode::Dark,
            controls: BTreeMap::from([(
                "src/button.hblank.rs#default".to_owned(),
                BTreeMap::from([("label".to_owned(), ControlValue::Text("Saved".to_owned()))]),
            )]),
        };
        let source = toml::to_string(&state).expect("state should serialize");
        let restored: PersistedState = toml::from_str(&source).expect("state should parse");
        assert_eq!(restored.session, state.session);
        assert_eq!(restored.selected, state.selected);
        assert_eq!(restored.filter, state.filter);
        assert_eq!(restored.theme, state.theme);
        assert_eq!(restored.controls, state.controls);
    }

    #[test]
    fn theme_mode_resolves_system_and_overrides() {
        assert_eq!(
            resolved_theme(ThemeMode::System, WindowAppearance::Dark),
            ResolvedTheme::Dark
        );
        assert_eq!(
            resolved_theme(ThemeMode::System, WindowAppearance::VibrantLight),
            ResolvedTheme::Light
        );
        assert_eq!(
            resolved_theme(ThemeMode::Light, WindowAppearance::Dark),
            ResolvedTheme::Light
        );
        assert_eq!(
            resolved_theme(ThemeMode::Dark, WindowAppearance::Light),
            ResolvedTheme::Dark
        );
    }

    #[test]
    fn invalid_environment_dimension_uses_fallback() {
        // An unrepresentable environment key cannot exist and exercises the fallback deterministically.
        let value = env_dimension("HBLANK_TEST_DIMENSION_THAT_IS_NOT_SET", 900.0);
        assert!((value - 900.0).abs() < f32::EPSILON);
    }

    #[test]
    fn requested_fixture_overrides_persisted_selection() {
        let entries = [
            ("first", "/project/src/first.hblank.rs"),
            ("requested-a", "/project/src/requested.hblank.rs"),
            ("requested-b", "/project/src/requested.hblank.rs"),
        ];

        let selection = initial_selection(
            entries.into_iter(),
            Some("first"),
            Some("/project/src/requested.hblank.rs"),
        );

        assert_eq!(selection, (Some(1), true));
    }

    #[test]
    fn persisted_selection_remains_fallback_without_fixture_match() {
        let entries = [
            ("first", "/project/src/first.hblank.rs"),
            ("persisted", "/project/src/persisted.hblank.rs"),
        ];

        let selection = initial_selection(
            entries.into_iter(),
            Some("persisted"),
            Some("/project/src/missing.hblank.rs"),
        );

        assert_eq!(selection, (Some(1), false));
    }

    #[test]
    fn platform_zoom_shortcuts_map_plus_equals_and_minus() {
        let command = Modifiers::command();
        let shifted_command = Modifiers {
            shift: true,
            ..command
        };

        assert_eq!(ui_scale_delta("=", command), Some(UI_SCALE_STEP));
        assert_eq!(ui_scale_delta("+", shifted_command), Some(UI_SCALE_STEP));
        assert_eq!(ui_scale_delta("-", command), Some(-UI_SCALE_STEP));
        assert_eq!(ui_scale_delta("=", Modifiers::default()), None);
        assert_eq!(
            ui_scale_delta(
                "=",
                Modifiers {
                    alt: true,
                    ..command
                }
            ),
            None
        );
    }

    #[test]
    fn ui_scale_stays_within_supported_bounds() {
        let mut scale = 1.0;
        for _ in 0..20 {
            scale = bounded_ui_scale(scale, UI_SCALE_STEP);
        }
        assert!((scale - MAX_UI_SCALE).abs() < f32::EPSILON);

        for _ in 0..30 {
            scale = bounded_ui_scale(scale, -UI_SCALE_STEP);
        }
        assert!((scale - MIN_UI_SCALE).abs() < f32::EPSILON);
    }
}

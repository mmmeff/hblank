#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::{
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use gpui::{
    App, Application, Bounds, Context, FocusHandle, IntoElement, KeyDownEvent, Render,
    SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use serde::{Deserialize, Serialize};

use crate::{ControlValue, ExampleDefinition, registered_examples};

use super::components::{
    CanvasProps, ControlAction, ControlsPanelProps, DocsPanelProps, EmptyStateProps, HeaderProps,
    InspectorTab, NavigationAction, NavigationItem, NavigationProps, SearchAction, SearchProps,
    ToolbarAction, ToolbarProps, UiHandler, canvas, controls_panel, docs_panel, empty_state,
    header, navigation, search, theme, toolbar,
};

const DEFAULT_WIDTH: f32 = 1440.0;
const DEFAULT_HEIGHT: f32 = 900.0;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct PersistedState {
    selected: Option<String>,
    filter: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditingTarget {
    Search,
    TextControl(&'static str),
}

struct HarnessApp {
    examples: Vec<ExampleDefinition>,
    navigation: Vec<NavigationItem>,
    selected: Option<usize>,
    filter: String,
    inspector: InspectorTab,
    editing: EditingTarget,
    focus_handle: FocusHandle,
    project: SharedString,
    status: SharedString,
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

impl HarnessApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state_path = state_path();
        let persisted = load_state(&state_path);
        let (examples, mut status) = match registered_examples() {
            Ok(examples) => (examples, SharedString::from("Ready")),
            Err(error) => (Vec::new(), SharedString::from(error.to_string())),
        };
        let requested_fixture = env::var("HBLANK_INITIAL_FIXTURE").ok();
        let (selected, matched_fixture) = initial_selection(
            examples.iter().map(|example| {
                let metadata = example.metadata();
                (metadata.id, metadata.source)
            }),
            persisted.selected.as_deref(),
            requested_fixture.as_deref(),
        );
        if requested_fixture.is_some() && !matched_fixture {
            status = "Requested fixture contains no registered examples".into();
        }
        let navigation = examples
            .iter()
            .map(|example| {
                let metadata = example.metadata();
                NavigationItem {
                    id: metadata.id,
                    title: metadata.title,
                    group: metadata.group,
                }
            })
            .collect::<Vec<_>>();
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);
        println!("Hblank harness ready: {} examples", examples.len());

        let app = Self {
            examples,
            navigation,
            selected,
            filter: if matched_fixture {
                String::new()
            } else {
                persisted.filter
            },
            inspector: InspectorTab::Controls,
            editing: EditingTarget::Search,
            focus_handle,
            project: env::var("HBLANK_WINDOW_TITLE")
                .unwrap_or_else(|_| "GPUI project · Hblank".to_owned())
                .into(),
            status,
            state_path,
        };
        if matched_fixture {
            app.persist();
        }
        app
    }

    fn selected_example(&self) -> Option<&ExampleDefinition> {
        self.selected.and_then(|index| self.examples.get(index))
    }

    fn selected_example_mut(&mut self) -> Option<&mut ExampleDefinition> {
        self.selected.and_then(|index| self.examples.get_mut(index))
    }

    fn selected_id(&self) -> Option<&'static str> {
        self.selected_example().map(|example| example.metadata().id)
    }

    fn select(&mut self, id: &'static str, cx: &mut Context<Self>) {
        if let Some(index) = self
            .examples
            .iter()
            .position(|example| example.metadata().id == id)
        {
            self.selected = Some(index);
            self.editing = EditingTarget::Search;
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
        self.select(action.id, cx);
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn on_search_focus(&mut self, _: &SearchAction, window: &mut Window, cx: &mut Context<Self>) {
        self.editing = EditingTarget::Search;
        self.focus_handle.focus(window);
        cx.notify();
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn on_toolbar(&mut self, action: &ToolbarAction, _window: &mut Window, cx: &mut Context<Self>) {
        self.inspector = match action {
            ToolbarAction::ShowControls => InspectorTab::Controls,
            ToolbarAction::ShowDocs => InspectorTab::Docs,
        };
        cx.notify();
    }

    fn on_control(&mut self, action: &ControlAction, window: &mut Window, cx: &mut Context<Self>) {
        match action {
            ControlAction::Set { id, value } => {
                let result = self
                    .selected_example_mut()
                    .expect("a rendered control always has a selected example")
                    .set_control(id, value.clone());
                self.status = match result {
                    Ok(()) => SharedString::from("Ready"),
                    Err(error) => SharedString::from(error.to_string()),
                };
            }
            ControlAction::EditText { id } => {
                self.editing = EditingTarget::TextControl(id);
                self.focus_handle.focus(window);
            }
            ControlAction::Reset => {
                if let Some(example) = self.selected_example_mut() {
                    example.reset();
                }
                self.editing = EditingTarget::Search;
                self.status = SharedString::from("Ready");
            }
        }
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "up" => {
                self.navigate_filtered(-1, cx);
                return;
            }
            "down" => {
                self.navigate_filtered(1, cx);
                return;
            }
            "escape" => {
                self.filter.clear();
                self.editing = EditingTarget::Search;
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

        if let Some(text) = event.keystroke.key_char.as_deref() {
            if !text.chars().any(char::is_control) {
                self.insert_text(text);
                self.persist();
                cx.notify();
            }
        }
    }

    fn remove_character(&mut self) {
        match self.editing {
            EditingTarget::Search => {
                self.filter.pop();
            }
            EditingTarget::TextControl(id) => {
                let Some(example) = self.selected_example_mut() else {
                    return;
                };
                let Some(ControlValue::Text(mut value)) = example.props().control_value(id) else {
                    return;
                };
                value.pop();
                let _ = example.set_control(id, ControlValue::Text(value));
            }
        }
    }

    fn insert_text(&mut self, text: &str) {
        match self.editing {
            EditingTarget::Search => self.filter.push_str(text),
            EditingTarget::TextControl(id) => {
                let Some(example) = self.selected_example_mut() else {
                    return;
                };
                let Some(ControlValue::Text(mut value)) = example.props().control_value(id) else {
                    return;
                };
                value.push_str(text);
                let _ = example.set_control(id, ControlValue::Text(value));
            }
        }
    }

    fn persist(&self) {
        let state = PersistedState {
            selected: self.selected_id().map(str::to_owned),
            filter: self.filter.clone(),
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
                title: "No examples discovered".into(),
                body: "Add a file matching .hblank/config.toml, annotate a render function with #[hblank::example], and save. It will appear here automatically."
                    .into(),
            })
            .into_any_element();
        };
        let metadata = *self.examples[index].metadata();
        let preview = self.examples[index].render(window, cx);
        let toolbar_surface = toolbar(
            ToolbarProps {
                title: metadata.title.into(),
                source: source_label(metadata.source, metadata.line),
                active_tab: self.inspector,
            },
            toolbar_handler,
        );
        let inspector = match self.inspector {
            InspectorTab::Controls => controls_panel(
                ControlsPanelProps {
                    definitions: self.examples[index].props().definitions(),
                    props: self.examples[index].props(),
                    editing_text: match self.editing {
                        EditingTarget::TextControl(id) => Some(id),
                        EditingTarget::Search => None,
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
            .bg(rgb(theme::CANVAS))
            .text_color(rgb(theme::TEXT))
            .child(header(HeaderProps {
                project: self.project.clone(),
                example_count: self.examples.len(),
                status: self.status.clone(),
            }))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(
                        div()
                            .w(px(272.0))
                            .h_full()
                            .flex_none()
                            .flex()
                            .flex_col()
                            .bg(rgb(theme::SIDEBAR))
                            .child(search_surface)
                            .child(navigation_surface),
                    )
                    .child(body),
            )
    }
}

pub fn run_harness() {
    Application::new().run(|cx: &mut App| {
        cx.on_window_closed(|cx| {
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
    use super::{PersistedState, env_dimension, initial_selection};

    #[test]
    fn persisted_state_round_trips() {
        let state = PersistedState {
            selected: Some("components::button".to_owned()),
            filter: "button".to_owned(),
        };
        let source = toml::to_string(&state).expect("state should serialize");
        let restored: PersistedState = toml::from_str(&source).expect("state should parse");
        assert_eq!(restored.selected, state.selected);
        assert_eq!(restored.filter, state.filter);
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
}

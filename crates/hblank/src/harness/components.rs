#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::rc::Rc;

use gpui::{AnyElement, App, Div, FontWeight, SharedString, Window, div, prelude::*, px, rgb};

use crate::{ControlDefinition, ControlKind, ControlValue, HblankProps};

pub type UiHandler<T> = Rc<dyn Fn(&T, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorTab {
    Controls,
    Docs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderProps {
    pub project: SharedString,
    pub example_count: usize,
    pub status: SharedString,
}

#[must_use]
pub fn header(props: HeaderProps) -> Div {
    div()
        .h(px(54.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .bg(rgb(0x17171a))
        .border_b_1()
        .border_color(rgb(0x303036))
        .text_color(rgb(0xf8f8f6))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .w(px(30.0))
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .bg(rgb(0x7357d8))
                        .font_weight(FontWeight::BOLD)
                        .child("H"),
                )
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("hblank"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xa9a9b2))
                                .child(props.project),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_xs()
                .child(
                    div()
                        .rounded_full()
                        .bg(rgb(0x29292f))
                        .px_3()
                        .py_1()
                        .text_color(rgb(0xc8c8d0))
                        .child(format!("{} examples", props.example_count)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_color(rgb(0x9fe4b2))
                        .child(div().size_2().rounded_full().bg(rgb(0x53c878)))
                        .child(props.status),
                ),
        )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchProps {
    pub query: SharedString,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchAction;

#[must_use]
pub fn search(props: SearchProps, on_focus: UiHandler<SearchAction>) -> impl IntoElement {
    let empty = props.query.is_empty();
    div()
        .id("hblank-search")
        .mx_3()
        .mt_3()
        .mb_2()
        .h(px(36.0))
        .flex_none()
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(if props.active {
            rgb(0x876ff0)
        } else {
            rgb(0x3a3a42)
        })
        .bg(rgb(0x232329))
        .text_sm()
        .text_color(if empty { rgb(0x85858f) } else { rgb(0xf0f0ee) })
        .cursor_pointer()
        .hover(|this| this.border_color(rgb(0x686875)))
        .on_click(move |_, window, cx| on_focus(&SearchAction, window, cx))
        .child(if empty {
            SharedString::from("Filter examples…")
        } else {
            props.query
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationItem {
    pub id: &'static str,
    pub title: &'static str,
    pub group: &'static str,
}

#[derive(Clone, Copy)]
pub struct NavigationProps<'a> {
    pub items: &'a [NavigationItem],
    pub selected: Option<&'static str>,
    pub query: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationAction {
    pub id: &'static str,
}

#[must_use]
pub fn navigation(props: NavigationProps<'_>, on_select: &UiHandler<NavigationAction>) -> Div {
    let on_select = Rc::clone(on_select);
    let query = props.query.to_ascii_lowercase();
    let visible = props.items.iter().filter(|item| {
        query.is_empty()
            || item.title.to_ascii_lowercase().contains(&query)
            || item.group.to_ascii_lowercase().contains(&query)
    });
    let mut previous_group = None;
    let mut children = Vec::new();
    for (index, item) in visible.enumerate() {
        if previous_group != Some(item.group) {
            previous_group = Some(item.group);
            children.push(
                div()
                    .mt_3()
                    .mb_1()
                    .px_3()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0x878791))
                    .child(item.group)
                    .into_any_element(),
            );
        }
        let action = NavigationAction { id: item.id };
        let handler = on_select.clone();
        let selected = props.selected == Some(item.id);
        children.push(
            div()
                .id(("hblank-nav", index))
                .mx_2()
                .h(px(34.0))
                .flex()
                .items_center()
                .px_3()
                .rounded_md()
                .text_sm()
                .cursor_pointer()
                .bg(if selected {
                    rgb(0x332e4d)
                } else {
                    rgb(0x1d1d22)
                })
                .text_color(if selected {
                    rgb(0xded7ff)
                } else {
                    rgb(0xc4c4cb)
                })
                .hover(|this| this.bg(rgb(0x292930)).text_color(rgb(0xffffff)))
                .on_click(move |_, window, cx| handler(&action, window, cx))
                .child(item.title)
                .into_any_element(),
        );
    }

    div()
        .w(px(272.0))
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(0x1d1d22))
        .border_r_1()
        .border_color(rgb(0x303036))
        .child(
            div()
                .id("hblank-navigation-scroll")
                .flex_1()
                .min_h_0()
                .overflow_scroll()
                .children(children),
        )
        .child(
            div()
                .flex_none()
                .px_3()
                .py_3()
                .border_t_1()
                .border_color(rgb(0x303036))
                .text_xs()
                .text_color(rgb(0x777781))
                .child("Type to filter · ↑↓ to navigate"),
        )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarProps {
    pub title: SharedString,
    pub source: SharedString,
    pub active_tab: InspectorTab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarAction {
    ShowControls,
    ShowDocs,
}

#[must_use]
pub fn toolbar(props: ToolbarProps, on_action: UiHandler<ToolbarAction>) -> Div {
    let controls_handler = on_action.clone();
    let docs_handler = on_action;
    div()
        .h(px(58.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .bg(rgb(0xffffff))
        .border_b_1()
        .border_color(rgb(0xe4e4e0))
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x202024))
                        .child(props.title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x77777f))
                        .child(props.source),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .rounded_md()
                .bg(rgb(0xf0f0ed))
                .p_1()
                .child(tab_button(
                    "Controls",
                    props.active_tab == InspectorTab::Controls,
                    move |window, cx| {
                        controls_handler(&ToolbarAction::ShowControls, window, cx);
                    },
                ))
                .child(tab_button(
                    "Docs",
                    props.active_tab == InspectorTab::Docs,
                    move |window, cx| {
                        docs_handler(&ToolbarAction::ShowDocs, window, cx);
                    },
                )),
        )
}

fn tab_button(
    label: &'static str,
    selected: bool,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(label)
        .h(px(30.0))
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .bg(if selected {
            rgb(0xffffff)
        } else {
            rgb(0xf0f0ed)
        })
        .text_color(if selected {
            rgb(0x312765)
        } else {
            rgb(0x6f6f76)
        })
        .hover(|this| this.text_color(rgb(0x312765)))
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(label)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasProps {
    pub label: SharedString,
}

#[must_use]
pub fn canvas(props: CanvasProps, preview: AnyElement) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(0xf5f5f1))
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(rgb(0x898990))
                .child(props.label),
        )
        .child(
            div()
                .id("hblank-canvas-scroll")
                .flex_1()
                .min_h_0()
                .overflow_scroll()
                .p_8()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .min_w(px(280.0))
                        .min_h(px(180.0))
                        .p_6()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_lg()
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xe1e1dc))
                        .shadow_md()
                        .child(preview),
                ),
        )
}

#[derive(Clone, Copy)]
pub struct ControlsPanelProps<'a> {
    pub definitions: &'static [ControlDefinition],
    pub props: &'a dyn HblankProps,
    pub editing_text: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlAction {
    Set {
        id: &'static str,
        value: ControlValue,
    },
    EditText {
        id: &'static str,
    },
    Reset,
}

#[must_use]
pub fn controls_panel(props: ControlsPanelProps<'_>, on_action: UiHandler<ControlAction>) -> Div {
    let row_handler = on_action.clone();
    let rows = props
        .definitions
        .iter()
        .enumerate()
        .map(move |(index, definition)| {
            let value = props.props.control_value(definition.id);
            control_row(
                index,
                definition,
                value,
                props.editing_text == Some(definition.id),
                row_handler.clone(),
            )
        });
    let reset_handler = on_action;

    div()
        .w(px(320.0))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(0xffffff))
        .border_l_1()
        .border_color(rgb(0xe4e4e0))
        .child(
            div()
                .h(px(44.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .border_b_1()
                .border_color(rgb(0xe9e9e5))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x333338))
                        .child("PROPERTIES"),
                )
                .child(
                    div()
                        .id("hblank-reset")
                        .text_xs()
                        .text_color(rgb(0x6f56c9))
                        .cursor_pointer()
                        .hover(|this| this.text_color(rgb(0x4c369f)))
                        .on_click(move |_, window, cx| {
                            reset_handler(&ControlAction::Reset, window, cx);
                        })
                        .child("Reset"),
                ),
        )
        .child(
            div()
                .id("hblank-controls-scroll")
                .flex_1()
                .min_h_0()
                .overflow_scroll()
                .children(rows),
        )
}

fn control_row(
    index: usize,
    definition: &'static ControlDefinition,
    value: Option<ControlValue>,
    editing: bool,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let control = control_input(index, definition, value, editing, handler);
    div()
        .px_4()
        .py_4()
        .border_b_1()
        .border_color(rgb(0xeeeeea))
        .child(
            div()
                .mb_2()
                .flex()
                .items_baseline()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x303034))
                        .child(definition.label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x919198))
                        .child(definition.kind.name()),
                ),
        )
        .when(!definition.docs.is_empty(), |this| {
            this.child(
                div()
                    .mb_3()
                    .text_xs()
                    .text_color(rgb(0x74747c))
                    .child(definition.docs),
            )
        })
        .child(control)
        .into_any_element()
}

fn control_input(
    index: usize,
    definition: &'static ControlDefinition,
    value: Option<ControlValue>,
    editing: bool,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    match (definition.kind, value) {
        (ControlKind::Boolean, Some(ControlValue::Boolean(value))) => {
            boolean_control(index, definition.id, value, handler)
        }
        (ControlKind::Text, Some(ControlValue::Text(value))) => {
            text_control(index, definition.id, value, editing, handler)
        }
        (ControlKind::Number, Some(ControlValue::Number(value))) => {
            number_control(index, definition.id, value, handler)
        }
        (ControlKind::Enum { options }, Some(ControlValue::Enum(selected))) => {
            enum_control(index, definition.id, options, &selected, &handler)
        }
        _ => div()
            .text_xs()
            .text_color(rgb(0xb64141))
            .child("Control value unavailable")
            .into_any_element(),
    }
}

fn boolean_control(
    index: usize,
    id: &'static str,
    value: bool,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let action = ControlAction::Set {
        id,
        value: ControlValue::Boolean(!value),
    };
    div()
        .id(("hblank-bool", index))
        .w(px(42.0))
        .h(px(24.0))
        .p_1()
        .flex()
        .items_center()
        .justify_end()
        .rounded_full()
        .cursor_pointer()
        .bg(if value { rgb(0x7357d8) } else { rgb(0xd7d7d2) })
        .when(!value, gpui::Styled::justify_start)
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(div().size_4().rounded_full().bg(rgb(0xffffff)))
        .into_any_element()
}

fn text_control(
    index: usize,
    id: &'static str,
    value: String,
    editing: bool,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let action = ControlAction::EditText { id };
    div()
        .id(("hblank-text", index))
        .min_h(px(34.0))
        .w_full()
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(if editing {
            rgb(0x7357d8)
        } else {
            rgb(0xdededa)
        })
        .bg(rgb(0xfafaf8))
        .text_sm()
        .text_color(rgb(0x343438))
        .cursor_pointer()
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(if value.is_empty() {
            SharedString::from("Type a value…")
        } else {
            SharedString::from(value)
        })
        .into_any_element()
}

fn number_control(
    index: usize,
    id: &'static str,
    value: f64,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let decrement = handler.clone();
    let increment = handler;
    let decrement_action = ControlAction::Set {
        id,
        value: ControlValue::Number(value - 1.0),
    };
    let increment_action = ControlAction::Set {
        id,
        value: ControlValue::Number(value + 1.0),
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(step_button(index * 2, "−", move |window, cx| {
            decrement(&decrement_action, window, cx);
        }))
        .child(
            div()
                .w(px(72.0))
                .h(px(32.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xdededa))
                .bg(rgb(0xfafaf8))
                .text_sm()
                .text_color(rgb(0x343438))
                .child(format_number(value)),
        )
        .child(step_button(index * 2 + 1, "+", move |window, cx| {
            increment(&increment_action, window, cx);
        }))
        .into_any_element()
}

fn enum_control(
    index: usize,
    id: &'static str,
    options: &'static [&'static str],
    selected: &str,
    handler: &UiHandler<ControlAction>,
) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .gap_1()
        .children(options.iter().enumerate().map(|(option_index, option)| {
            let option_handler = Rc::clone(handler);
            let action = ControlAction::Set {
                id,
                value: ControlValue::Enum((*option).to_owned()),
            };
            let is_selected = selected == *option;
            div()
                .id(("hblank-enum", index * 100 + option_index))
                .h(px(30.0))
                .flex()
                .items_center()
                .px_3()
                .rounded_md()
                .border_1()
                .border_color(if is_selected {
                    rgb(0x7357d8)
                } else {
                    rgb(0xdededa)
                })
                .bg(if is_selected {
                    rgb(0xf0ecff)
                } else {
                    rgb(0xffffff)
                })
                .text_xs()
                .text_color(rgb(0x3b3459))
                .cursor_pointer()
                .on_click(move |_, window, cx| option_handler(&action, window, cx))
                .child(*option)
        }))
        .into_any_element()
}

fn step_button(
    id: usize,
    label: &'static str,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(("hblank-step", id))
        .size_8()
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .border_1()
        .border_color(rgb(0xdededa))
        .bg(rgb(0xffffff))
        .text_color(rgb(0x514c67))
        .cursor_pointer()
        .hover(|this| this.bg(rgb(0xf2efff)).border_color(rgb(0xa695e7)))
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(label)
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsPanelProps {
    pub title: SharedString,
    pub docs: SharedString,
    pub source: SharedString,
}

#[must_use]
pub fn docs_panel(props: DocsPanelProps) -> Div {
    let has_docs = !props.docs.is_empty();
    div()
        .w(px(320.0))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(0xffffff))
        .border_l_1()
        .border_color(rgb(0xe4e4e0))
        .child(
            div()
                .h(px(44.0))
                .flex_none()
                .flex()
                .items_center()
                .px_4()
                .border_b_1()
                .border_color(rgb(0xe9e9e5))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x333338))
                .child("DOCUMENTATION"),
        )
        .child(
            div()
                .id("hblank-docs-scroll")
                .flex_1()
                .min_h_0()
                .overflow_scroll()
                .p_5()
                .child(
                    div()
                        .mb_4()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x26262a))
                        .child(props.title),
                )
                .child(
                    div()
                        .mb_5()
                        .text_sm()
                        .text_color(if has_docs {
                            rgb(0x55555d)
                        } else {
                            rgb(0x898991)
                        })
                        .child(if has_docs {
                            props.docs
                        } else {
                            SharedString::from("Add Rust doc comments above the #[hblank::example] function to document this example.")
                        }),
                )
                .child(
                    div()
                        .pt_4()
                        .border_t_1()
                        .border_color(rgb(0xe9e9e5))
                        .text_xs()
                        .text_color(rgb(0x85858d))
                        .child(props.source),
                ),
        )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmptyStateProps {
    pub title: SharedString,
    pub body: SharedString,
}

#[must_use]
pub fn empty_state(props: EmptyStateProps) -> Div {
    div()
        .flex_1()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xf5f5f1))
        .child(
            div()
                .w(px(420.0))
                .p_8()
                .rounded_lg()
                .border_1()
                .border_color(rgb(0xe1e1dc))
                .bg(rgb(0xffffff))
                .text_center()
                .child(
                    div()
                        .mb_2()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x27272b))
                        .child(props.title),
                )
                .child(div().text_sm().text_color(rgb(0x6f6f77)).child(props.body)),
        )
}

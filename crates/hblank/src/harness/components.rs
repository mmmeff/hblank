#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, FontWeight, SharedString, Window, div, prelude::*, px, rems, rgb,
};

use crate::{ControlDefinition, ControlKind, ControlValue, HblankProps};

pub(super) mod theme {
    pub const CHROME: u32 = 0x17171c;
    pub const CHROME_RAISED: u32 = 0x24242c;
    pub const CHROME_BORDER: u32 = 0x32323c;
    pub const SIDEBAR: u32 = 0x1d1d24;
    pub const SIDEBAR_HOVER: u32 = 0x292932;
    pub const SIDEBAR_SELECTED: u32 = 0x39325a;
    pub const SIDEBAR_SELECTED_HOVER: u32 = 0x44396c;
    pub const CHROME_TEXT: u32 = 0xf8f8f6;
    pub const CHROME_TEXT_MUTED: u32 = 0xb0b0bb;
    pub const SIDEBAR_TEXT: u32 = 0xd1d1d9;
    pub const SIDEBAR_TEXT_MUTED: u32 = 0x9999a5;
    pub const PAPER: u32 = 0xffffff;
    pub const CANVAS: u32 = 0xf4f4f0;
    pub const SURFACE_SUBTLE: u32 = 0xf1f1ed;
    pub const LINE: u32 = 0xe5e5df;
    pub const LINE_STRONG: u32 = 0xd8d8d2;
    pub const TEXT: u32 = 0x29292e;
    pub const TEXT_MUTED: u32 = 0x66666f;
    pub const TEXT_SUBTLE: u32 = 0x74747e;
    pub const ACCENT: u32 = 0x7559e8;
    pub const ACCENT_HOVER: u32 = 0x6347d4;
    pub const ACCENT_WASH: u32 = 0xeeeaff;
    pub const ACCENT_INK: u32 = 0x3e2b86;
    pub const SUCCESS: u32 = 0x53cf82;
    pub const SUCCESS_TEXT: u32 = 0xa2e8bb;
    pub const ERROR: u32 = 0xf18175;
    pub const ERROR_INK: u32 = 0x9e3f37;
    pub const ERROR_TEXT: u32 = 0xffb4ab;
}

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
    let ready = props.status.as_ref() == "Ready";
    div()
        .h(rems(3.625))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .px_5()
        .bg(rgb(theme::CHROME))
        .border_b_1()
        .border_color(rgb(theme::CHROME_BORDER))
        .text_color(rgb(theme::CHROME_TEXT))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .size_8()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_lg()
                        .bg(rgb(theme::ACCENT))
                        .font_weight(FontWeight::BOLD)
                        .child("H"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("hblank"),
                        )
                        .child(div().w(px(1.0)).h_4().bg(rgb(theme::CHROME_BORDER)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::CHROME_TEXT_MUTED))
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
                        .border_1()
                        .border_color(rgb(theme::CHROME_BORDER))
                        .rounded_full()
                        .bg(rgb(theme::CHROME_RAISED))
                        .px_3()
                        .py_1()
                        .text_color(rgb(theme::SIDEBAR_TEXT))
                        .child(format!(
                            "{} example{}",
                            props.example_count,
                            if props.example_count == 1 { "" } else { "s" }
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_color(rgb(if ready {
                            theme::SUCCESS_TEXT
                        } else {
                            theme::ERROR_TEXT
                        }))
                        .child(div().size_2().rounded_full().bg(rgb(if ready {
                            theme::SUCCESS
                        } else {
                            theme::ERROR
                        })))
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
    let active = props.active;
    div()
        .id("hblank-search")
        .mx_3()
        .mt_4()
        .mb_3()
        .h(rems(2.5))
        .flex_none()
        .flex()
        .items_center()
        .px_3()
        .rounded_lg()
        .border_1()
        .border_color(if active {
            rgb(theme::ACCENT)
        } else {
            rgb(theme::CHROME_BORDER)
        })
        .bg(rgb(theme::CHROME_RAISED))
        .text_sm()
        .text_color(if empty {
            rgb(theme::SIDEBAR_TEXT_MUTED)
        } else {
            rgb(theme::CHROME_TEXT)
        })
        .cursor_pointer()
        .hover(move |this| {
            this.border_color(rgb(if active {
                theme::ACCENT
            } else {
                theme::CHROME_TEXT_MUTED
            }))
        })
        .active(|this| this.bg(rgb(theme::SIDEBAR_HOVER)))
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
    let query = props.query.to_ascii_lowercase();
    let visible = props.items.iter().filter(|item| {
        query.is_empty()
            || item.title.to_ascii_lowercase().contains(&query)
            || item.group.to_ascii_lowercase().contains(&query)
    });
    let mut previous_group = None;
    let mut visible_count = 0;
    let mut children = Vec::new();
    for (index, item) in visible.enumerate() {
        visible_count += 1;
        if previous_group != Some(item.group) {
            previous_group = Some(item.group);
            children.push(
                div()
                    .mt_3()
                    .mb_1()
                    .px_4()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(theme::SIDEBAR_TEXT_MUTED))
                    .child(item.group)
                    .into_any_element(),
            );
        }
        children.push(navigation_row(
            index,
            item,
            props.selected == Some(item.id),
            on_select,
        ));
    }
    if visible_count == 0 {
        children.push(
            div()
                .mx_3()
                .mt_5()
                .p_4()
                .rounded_lg()
                .bg(rgb(theme::CHROME_RAISED))
                .text_sm()
                .text_color(rgb(theme::SIDEBAR_TEXT_MUTED))
                .child("No matching examples")
                .into_any_element(),
        );
    }

    div()
        .w(rems(17.0))
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(theme::SIDEBAR))
        .border_r_1()
        .border_color(rgb(theme::CHROME_BORDER))
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
                .border_color(rgb(theme::CHROME_BORDER))
                .text_xs()
                .text_color(rgb(theme::SIDEBAR_TEXT_MUTED))
                .child("Type to filter · Arrow keys to navigate"),
        )
}

fn navigation_row(
    index: usize,
    item: &NavigationItem,
    selected: bool,
    on_select: &UiHandler<NavigationAction>,
) -> AnyElement {
    let action = NavigationAction { id: item.id };
    let handler = Rc::clone(on_select);
    div()
        .id(("hblank-nav", index))
        .mx_2()
        .h(rems(2.0))
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(if selected {
            theme::ACCENT
        } else {
            theme::SIDEBAR
        }))
        .text_sm()
        .cursor_pointer()
        .bg(if selected {
            rgb(theme::SIDEBAR_SELECTED)
        } else {
            rgb(theme::SIDEBAR)
        })
        .text_color(if selected {
            rgb(theme::CHROME_TEXT)
        } else {
            rgb(theme::SIDEBAR_TEXT)
        })
        .hover(move |this| {
            if selected {
                this.bg(rgb(theme::SIDEBAR_SELECTED_HOVER))
                    .border_color(rgb(theme::ACCENT))
            } else {
                this.bg(rgb(theme::SIDEBAR_HOVER))
                    .border_color(rgb(theme::CHROME_BORDER))
            }
            .text_color(rgb(theme::CHROME_TEXT))
        })
        .active(move |this| {
            this.bg(rgb(if selected {
                theme::SIDEBAR_SELECTED_HOVER
            } else {
                theme::CHROME_RAISED
            }))
        })
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(item.title)
        .into_any_element()
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
        .h(rems(4.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .px_5()
        .bg(rgb(theme::PAPER))
        .border_b_1()
        .border_color(rgb(theme::LINE))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .mr_4()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::TEXT))
                        .child(props.title),
                )
                .child(
                    div()
                        .min_w_0()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_SUBTLE))
                        .child(props.source),
                ),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap_1()
                .rounded_lg()
                .border_1()
                .border_color(rgb(theme::LINE))
                .bg(rgb(theme::SURFACE_SUBTLE))
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
        .h(rems(2.0))
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .bg(if selected {
            rgb(theme::PAPER)
        } else {
            rgb(theme::SURFACE_SUBTLE)
        })
        .text_color(if selected {
            rgb(theme::ACCENT_INK)
        } else {
            rgb(theme::TEXT_MUTED)
        })
        .when(selected, gpui::Styled::shadow_sm)
        .hover(move |this| {
            if selected {
                this.text_color(rgb(theme::ACCENT_INK))
            } else {
                this.bg(rgb(theme::ACCENT_WASH))
                    .text_color(rgb(theme::ACCENT_INK))
            }
        })
        .active(|this| this.bg(rgb(theme::ACCENT_WASH)))
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
        .bg(rgb(theme::CANVAS))
        .child(
            div()
                .h(rems(2.375))
                .flex_none()
                .flex()
                .items_center()
                .px_5()
                .border_b_1()
                .border_color(rgb(theme::LINE))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme::TEXT_SUBTLE))
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
                        .min_w(rems(17.5))
                        .min_h(rems(11.25))
                        .p_8()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_xl()
                        .bg(rgb(theme::PAPER))
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
        .w(rems(20.0))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(theme::PAPER))
        .border_l_1()
        .border_color(rgb(theme::LINE))
        .child(
            div()
                .h(rems(3.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .border_b_1()
                .border_color(rgb(theme::LINE))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::TEXT))
                        .child("PROPERTIES"),
                )
                .child(
                    div()
                        .id("hblank-reset")
                        .h(rems(2.0))
                        .flex()
                        .items_center()
                        .px_3()
                        .rounded_md()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::ACCENT_HOVER))
                        .cursor_pointer()
                        .hover(|this| {
                            this.bg(rgb(theme::ACCENT_WASH))
                                .text_color(rgb(theme::ACCENT_INK))
                        })
                        .active(|this| this.bg(rgb(theme::LINE)))
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
        .border_color(rgb(theme::LINE))
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
                        .text_color(rgb(theme::TEXT))
                        .child(definition.label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_SUBTLE))
                        .child(definition.kind.name()),
                ),
        )
        .when(!definition.docs.is_empty(), |this| {
            this.child(
                div()
                    .mb_3()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_MUTED))
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
            .text_color(rgb(theme::ERROR_INK))
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
        .w(rems(2.75))
        .h(rems(1.5))
        .p_1()
        .flex()
        .items_center()
        .justify_end()
        .rounded_full()
        .cursor_pointer()
        .bg(if value {
            rgb(theme::ACCENT)
        } else {
            rgb(theme::LINE_STRONG)
        })
        .when(!value, gpui::Styled::justify_start)
        .hover(move |this| {
            this.bg(rgb(if value {
                theme::ACCENT_HOVER
            } else {
                theme::CHROME_TEXT_MUTED
            }))
        })
        .active(|this| this.bg(rgb(theme::ACCENT_HOVER)))
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(
            div()
                .size_4()
                .rounded_full()
                .bg(rgb(theme::PAPER))
                .shadow_sm(),
        )
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
    let empty = value.is_empty();
    div()
        .id(("hblank-text", index))
        .min_h(rems(2.25))
        .w_full()
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(if editing {
            rgb(theme::ACCENT)
        } else {
            rgb(theme::LINE_STRONG)
        })
        .bg(rgb(theme::PAPER))
        .text_sm()
        .text_color(if empty {
            rgb(theme::TEXT_SUBTLE)
        } else {
            rgb(theme::TEXT)
        })
        .cursor_pointer()
        .hover(move |this| {
            this.border_color(rgb(if editing {
                theme::ACCENT
            } else {
                theme::CHROME_TEXT_MUTED
            }))
        })
        .active(|this| this.bg(rgb(theme::SURFACE_SUBTLE)))
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(if empty {
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
                .w(rems(4.5))
                .h(rems(2.125))
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .border_1()
                .border_color(rgb(theme::LINE_STRONG))
                .bg(rgb(theme::SURFACE_SUBTLE))
                .text_sm()
                .text_color(rgb(theme::TEXT))
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
                .h(rems(2.0))
                .flex()
                .items_center()
                .px_3()
                .rounded_md()
                .border_1()
                .border_color(if is_selected {
                    rgb(theme::ACCENT)
                } else {
                    rgb(theme::LINE_STRONG)
                })
                .bg(if is_selected {
                    rgb(theme::ACCENT)
                } else {
                    rgb(theme::PAPER)
                })
                .text_xs()
                .text_color(if is_selected {
                    rgb(theme::CHROME_TEXT)
                } else {
                    rgb(theme::ACCENT_INK)
                })
                .when(is_selected, |this| this.font_weight(FontWeight::SEMIBOLD))
                .cursor_pointer()
                .hover(move |this| {
                    if is_selected {
                        this.bg(rgb(theme::ACCENT_HOVER))
                            .border_color(rgb(theme::ACCENT_HOVER))
                    } else {
                        this.bg(rgb(theme::ACCENT_WASH))
                            .border_color(rgb(theme::ACCENT))
                    }
                })
                .active(|this| this.bg(rgb(theme::ACCENT_HOVER)))
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
        .border_color(rgb(theme::LINE_STRONG))
        .bg(rgb(theme::PAPER))
        .text_color(rgb(theme::ACCENT_INK))
        .cursor_pointer()
        .hover(|this| {
            this.bg(rgb(theme::ACCENT_WASH))
                .border_color(rgb(theme::ACCENT))
        })
        .active(|this| this.bg(rgb(theme::LINE)))
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
        .w(rems(20.0))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(theme::PAPER))
        .border_l_1()
        .border_color(rgb(theme::LINE))
        .child(
            div()
                .h(rems(3.0))
                .flex_none()
                .flex()
                .items_center()
                .px_4()
                .border_b_1()
                .border_color(rgb(theme::LINE))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme::TEXT))
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
                        .mb_3()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::TEXT))
                        .child(props.title),
                )
                .child(
                    div()
                        .mb_5()
                        .text_sm()
                        .text_color(if has_docs {
                            rgb(theme::TEXT_MUTED)
                        } else {
                            rgb(theme::TEXT_SUBTLE)
                        })
                        .child(if has_docs {
                            props.docs
                        } else {
                            SharedString::from("Add Rust doc comments above the #[hblank::example] function to document this example.")
                        }),
                )
                .child(
                    div()
                        .p_3()
                        .rounded_lg()
                        .bg(rgb(theme::SURFACE_SUBTLE))
                        .text_xs()
                        .text_color(rgb(theme::TEXT_SUBTLE))
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
        .bg(rgb(theme::CANVAS))
        .child(
            div()
                .w(rems(26.25))
                .p_8()
                .flex()
                .flex_col()
                .items_center()
                .rounded_xl()
                .bg(rgb(theme::PAPER))
                .shadow_md()
                .text_center()
                .child(
                    div()
                        .mb_4()
                        .size_10()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_lg()
                        .bg(rgb(theme::ACCENT))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(theme::CHROME_TEXT))
                        .child("H"),
                )
                .child(
                    div()
                        .mb_2()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::TEXT))
                        .child(props.title),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child(props.body),
                ),
        )
}

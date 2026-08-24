#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

use std::rc::Rc;

use crate::gpui;
use crate::gpui::{
    AnyElement, App, Div, FontWeight, SharedString, Window, div, prelude::*, px, rems, rgb,
};

use crate::{
    CalloutTone, ControlDefinition, ControlKind, ControlValue, HblankProps, NumberConstraints,
    TextMode, ThemeMode,
};

pub(super) mod theme {
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Clone, Copy)]
    struct Palette {
        chrome: u32,
        chrome_raised: u32,
        chrome_border: u32,
        sidebar: u32,
        sidebar_hover: u32,
        sidebar_selected: u32,
        sidebar_selected_hover: u32,
        chrome_text: u32,
        chrome_text_muted: u32,
        sidebar_text: u32,
        sidebar_text_muted: u32,
        paper: u32,
        canvas: u32,
        surface_subtle: u32,
        line: u32,
        line_strong: u32,
        text: u32,
        text_muted: u32,
        text_subtle: u32,
        accent: u32,
        accent_hover: u32,
        accent_wash: u32,
        accent_ink: u32,
        success: u32,
        success_text: u32,
        error: u32,
        error_ink: u32,
        error_text: u32,
    }

    const LIGHT: Palette = Palette {
        chrome: 0xf7f7f4,
        chrome_raised: 0xffffff,
        chrome_border: 0xdededa,
        sidebar: 0xefefec,
        sidebar_hover: 0xe5e5e0,
        sidebar_selected: 0xe4ddff,
        sidebar_selected_hover: 0xd8ceff,
        chrome_text: 0x29292e,
        chrome_text_muted: 0x66666f,
        sidebar_text: 0x3f3f47,
        sidebar_text_muted: 0x74747e,
        paper: 0xffffff,
        canvas: 0xf4f4f0,
        surface_subtle: 0xf1f1ed,
        line: 0xe5e5df,
        line_strong: 0xd8d8d2,
        text: 0x29292e,
        text_muted: 0x66666f,
        text_subtle: 0x74747e,
        accent: 0x7559e8,
        accent_hover: 0x6347d4,
        accent_wash: 0xeeeaff,
        accent_ink: 0x3e2b86,
        success: 0x258b63,
        success_text: 0x216e51,
        error: 0xc95b50,
        error_ink: 0x9e3f37,
        error_text: 0x9e3f37,
    };

    const DARK: Palette = Palette {
        chrome: 0x17171c,
        chrome_raised: 0x24242c,
        chrome_border: 0x393944,
        sidebar: 0x1d1d24,
        sidebar_hover: 0x292932,
        sidebar_selected: 0x39325a,
        sidebar_selected_hover: 0x44396c,
        chrome_text: 0xf8f8f6,
        chrome_text_muted: 0xb0b0bb,
        sidebar_text: 0xd1d1d9,
        sidebar_text_muted: 0x9999a5,
        paper: 0x24242c,
        canvas: 0x101015,
        surface_subtle: 0x2c2c35,
        line: 0x393944,
        line_strong: 0x4a4a57,
        text: 0xf8f8f6,
        text_muted: 0xb0b0bb,
        text_subtle: 0x9999a5,
        accent: 0x8b75f6,
        accent_hover: 0x7559e8,
        accent_wash: 0x39325a,
        accent_ink: 0xddd5ff,
        success: 0x53cf82,
        success_text: 0xa2e8bb,
        error: 0xf18175,
        error_ink: 0xffb4ab,
        error_text: 0xffb4ab,
    };

    static DARK_MODE: AtomicBool = AtomicBool::new(false);

    pub fn set_dark(dark: bool) {
        DARK_MODE.store(dark, Ordering::Relaxed);
    }

    fn palette() -> &'static Palette {
        if DARK_MODE.load(Ordering::Relaxed) {
            &DARK
        } else {
            &LIGHT
        }
    }

    macro_rules! colors {
        ($($name:ident),+ $(,)?) => {
            $(pub fn $name() -> u32 { palette().$name })+
        };
    }

    colors!(
        chrome,
        chrome_raised,
        chrome_border,
        sidebar,
        sidebar_hover,
        sidebar_selected,
        sidebar_selected_hover,
        chrome_text,
        chrome_text_muted,
        sidebar_text,
        sidebar_text_muted,
        paper,
        canvas,
        surface_subtle,
        line,
        line_strong,
        text,
        text_muted,
        text_subtle,
        accent,
        accent_hover,
        accent_wash,
        accent_ink,
        success,
        success_text,
        error,
        error_ink,
        error_text,
    );
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
    pub component_count: usize,
    pub fixture_count: usize,
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
        .bg(rgb(theme::chrome()))
        .border_b_1()
        .border_color(rgb(theme::chrome_border()))
        .text_color(rgb(theme::chrome_text()))
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
                        .bg(rgb(theme::accent()))
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
                        .child(div().w(px(1.0)).h_4().bg(rgb(theme::chrome_border())))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::chrome_text_muted()))
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
                        .border_color(rgb(theme::chrome_border()))
                        .rounded_full()
                        .bg(rgb(theme::chrome_raised()))
                        .px_3()
                        .py_1()
                        .text_color(rgb(theme::sidebar_text()))
                        .child(format!(
                            "{} component{} · {} fixture{}",
                            props.component_count,
                            if props.component_count == 1 { "" } else { "s" },
                            props.fixture_count,
                            if props.fixture_count == 1 { "" } else { "s" },
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_color(rgb(if ready {
                            theme::success_text()
                        } else {
                            theme::error_text()
                        }))
                        .child(div().size_2().rounded_full().bg(rgb(if ready {
                            theme::success()
                        } else {
                            theme::error()
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
            rgb(theme::accent())
        } else {
            rgb(theme::chrome_border())
        })
        .bg(rgb(theme::chrome_raised()))
        .text_sm()
        .text_color(if empty {
            rgb(theme::sidebar_text_muted())
        } else {
            rgb(theme::chrome_text())
        })
        .cursor_pointer()
        .hover(move |this| {
            this.border_color(rgb(if active {
                theme::accent()
            } else {
                theme::chrome_text_muted()
            }))
        })
        .active(|this| this.bg(rgb(theme::sidebar_hover())))
        .on_click(move |_, window, cx| on_focus(&SearchAction, window, cx))
        .child(if empty {
            SharedString::from("Filter fixtures…")
        } else {
            props.query
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationVariant {
    pub id: SharedString,
    pub title: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationComponent {
    pub id: SharedString,
    pub title: &'static str,
    pub group: &'static str,
    pub variants: Vec<NavigationVariant>,
}

#[derive(Clone, Copy)]
pub struct NavigationProps<'a> {
    pub components: &'a [NavigationComponent],
    pub selected: Option<&'a str>,
    pub query: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationAction {
    pub id: SharedString,
}

fn navigation_matches(
    component: &NavigationComponent,
    variant: &NavigationVariant,
    query: &str,
) -> bool {
    query.is_empty()
        || component.title.to_ascii_lowercase().contains(query)
        || component.group.to_ascii_lowercase().contains(query)
        || variant.title.to_ascii_lowercase().contains(query)
}

#[must_use]
pub fn navigation(props: NavigationProps<'_>, on_select: &UiHandler<NavigationAction>) -> Div {
    let query = props.query.to_ascii_lowercase();
    let mut previous_group = None;
    let mut visible_count = 0;
    let mut row_index = 0;
    let mut children = Vec::new();
    for component in props.components {
        let visible = component
            .variants
            .iter()
            .filter(|variant| navigation_matches(component, variant, &query))
            .collect::<Vec<_>>();
        if visible.is_empty() {
            continue;
        }
        visible_count += visible.len();
        if previous_group != Some(component.group) {
            previous_group = Some(component.group);
            children.push(group_heading(component.group));
        }
        let selected_component = component
            .variants
            .iter()
            .any(|variant| props.selected == Some(variant.id.as_ref()));
        children.push(component_row(
            row_index,
            component,
            visible[0],
            selected_component,
            on_select,
        ));
        row_index += 1;
        for variant in visible {
            children.push(variant_row(
                row_index,
                variant,
                props.selected == Some(variant.id.as_ref()),
                on_select,
            ));
            row_index += 1;
        }
    }
    if visible_count == 0 {
        children.push(
            div()
                .mx_3()
                .mt_5()
                .p_4()
                .rounded_lg()
                .bg(rgb(theme::chrome_raised()))
                .text_sm()
                .text_color(rgb(theme::sidebar_text_muted()))
                .child("No matching components")
                .into_any_element(),
        );
    }

    div()
        .w(rems(18.0))
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(theme::sidebar()))
        .border_r_1()
        .border_color(rgb(theme::chrome_border()))
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
                .border_color(rgb(theme::chrome_border()))
                .text_xs()
                .text_color(rgb(theme::sidebar_text_muted()))
                .child("Filter components and variants · Arrow keys navigate"),
        )
}

fn group_heading(group: &'static str) -> AnyElement {
    div()
        .mt_3()
        .mb_1()
        .px_4()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(theme::sidebar_text_muted()))
        .child(group)
        .into_any_element()
}

fn component_row(
    index: usize,
    component: &NavigationComponent,
    first_variant: &NavigationVariant,
    selected: bool,
    on_select: &UiHandler<NavigationAction>,
) -> AnyElement {
    let action = NavigationAction {
        id: first_variant.id.clone(),
    };
    let handler = Rc::clone(on_select);
    div()
        .id(("hblank-component", index))
        .mx_2()
        .h(rems(2.25))
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .rounded_md()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .bg(rgb(if selected {
            theme::sidebar_hover()
        } else {
            theme::sidebar()
        }))
        .text_color(rgb(theme::chrome_text()))
        .hover(|this| this.bg(rgb(theme::sidebar_hover())))
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(component.title)
        .child(
            div()
                .text_xs()
                .text_color(rgb(theme::sidebar_text_muted()))
                .child(component.variants.len().to_string()),
        )
        .into_any_element()
}

fn variant_row(
    index: usize,
    variant: &NavigationVariant,
    selected: bool,
    on_select: &UiHandler<NavigationAction>,
) -> AnyElement {
    let action = NavigationAction {
        id: variant.id.clone(),
    };
    let handler = Rc::clone(on_select);
    div()
        .id(("hblank-variant", index))
        .ml_5()
        .mr_2()
        .h(rems(1.875))
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(if selected {
            theme::accent()
        } else {
            theme::sidebar()
        }))
        .text_xs()
        .cursor_pointer()
        .bg(rgb(if selected {
            theme::sidebar_selected()
        } else {
            theme::sidebar()
        }))
        .text_color(rgb(if selected {
            theme::chrome_text()
        } else {
            theme::sidebar_text()
        }))
        .hover(move |this| {
            this.bg(rgb(if selected {
                theme::sidebar_selected_hover()
            } else {
                theme::sidebar_hover()
            }))
        })
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(variant.title)
        .into_any_element()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarProps {
    pub title: SharedString,
    pub source: SharedString,
    pub active_tab: InspectorTab,
    pub theme_mode: ThemeMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarAction {
    ShowControls,
    ShowDocs,
    SetTheme(ThemeMode),
}

#[must_use]
pub fn toolbar(props: ToolbarProps, on_action: UiHandler<ToolbarAction>) -> Div {
    let ToolbarProps {
        title,
        source,
        active_tab,
        theme_mode,
    } = props;
    div()
        .h(rems(4.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .px_5()
        .bg(rgb(theme::paper()))
        .border_b_1()
        .border_color(rgb(theme::line()))
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
                        .text_color(rgb(theme::text()))
                        .child(title),
                )
                .child(
                    div()
                        .min_w_0()
                        .text_xs()
                        .text_color(rgb(theme::text_subtle()))
                        .child(source),
                ),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap_2()
                .child(theme_buttons(theme_mode, on_action.clone()))
                .child(inspector_buttons(active_tab, on_action)),
        )
}

fn theme_buttons(mode: ThemeMode, on_action: UiHandler<ToolbarAction>) -> Div {
    let system_handler = on_action.clone();
    let light_handler = on_action.clone();
    let dark_handler = on_action;
    div()
        .flex()
        .items_center()
        .gap_1()
        .rounded_lg()
        .border_1()
        .border_color(rgb(theme::line()))
        .bg(rgb(theme::surface_subtle()))
        .p_1()
        .child(tab_button(
            "System",
            mode == ThemeMode::System,
            move |window, cx| {
                system_handler(&ToolbarAction::SetTheme(ThemeMode::System), window, cx);
            },
        ))
        .child(tab_button(
            "Light",
            mode == ThemeMode::Light,
            move |window, cx| {
                light_handler(&ToolbarAction::SetTheme(ThemeMode::Light), window, cx);
            },
        ))
        .child(tab_button(
            "Dark",
            mode == ThemeMode::Dark,
            move |window, cx| {
                dark_handler(&ToolbarAction::SetTheme(ThemeMode::Dark), window, cx);
            },
        ))
}

fn inspector_buttons(active: InspectorTab, on_action: UiHandler<ToolbarAction>) -> Div {
    let controls_handler = on_action.clone();
    let docs_handler = on_action;
    div()
        .flex()
        .items_center()
        .gap_1()
        .rounded_lg()
        .border_1()
        .border_color(rgb(theme::line()))
        .bg(rgb(theme::surface_subtle()))
        .p_1()
        .child(tab_button(
            "Controls",
            active == InspectorTab::Controls,
            move |window, cx| {
                controls_handler(&ToolbarAction::ShowControls, window, cx);
            },
        ))
        .child(tab_button(
            "Docs",
            active == InspectorTab::Docs,
            move |window, cx| {
                docs_handler(&ToolbarAction::ShowDocs, window, cx);
            },
        ))
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
            rgb(theme::paper())
        } else {
            rgb(theme::surface_subtle())
        })
        .text_color(if selected {
            rgb(theme::accent_ink())
        } else {
            rgb(theme::text_muted())
        })
        .when(selected, gpui::Styled::shadow_sm)
        .hover(move |this| {
            if selected {
                this.text_color(rgb(theme::accent_ink()))
            } else {
                this.bg(rgb(theme::accent_wash()))
                    .text_color(rgb(theme::accent_ink()))
            }
        })
        .active(|this| this.bg(rgb(theme::accent_wash())))
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
        .bg(rgb(theme::canvas()))
        .child(
            div()
                .h(rems(2.375))
                .flex_none()
                .flex()
                .items_center()
                .px_5()
                .border_b_1()
                .border_color(rgb(theme::line()))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme::text_subtle()))
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
                        .bg(rgb(theme::paper()))
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
    pub editing_number: Option<(&'static str, &'a str)>,
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
    EditNumber {
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
            let number_draft = props
                .editing_number
                .and_then(|(id, draft)| (id == definition.id).then_some(draft));
            control_row(
                index,
                definition,
                value,
                props.editing_text == Some(definition.id),
                number_draft,
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
        .bg(rgb(theme::paper()))
        .border_l_1()
        .border_color(rgb(theme::line()))
        .child(
            div()
                .h(rems(3.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .border_b_1()
                .border_color(rgb(theme::line()))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::text()))
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
                        .text_color(rgb(theme::accent_hover()))
                        .cursor_pointer()
                        .hover(|this| {
                            this.bg(rgb(theme::accent_wash()))
                                .text_color(rgb(theme::accent_ink()))
                        })
                        .active(|this| this.bg(rgb(theme::line())))
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
    editing_text: bool,
    number_draft: Option<&str>,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let control = control_input(
        index,
        definition,
        value,
        editing_text,
        number_draft,
        handler,
    );
    div()
        .px_4()
        .py_4()
        .border_b_1()
        .border_color(rgb(theme::line()))
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
                        .text_color(rgb(theme::text()))
                        .child(definition.label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme::text_subtle()))
                        .child(definition.kind.name()),
                ),
        )
        .when(!definition.docs.is_empty(), |this| {
            this.child(
                div()
                    .mb_3()
                    .text_xs()
                    .text_color(rgb(theme::text_muted()))
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
    editing_text: bool,
    number_draft: Option<&str>,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    match (definition.kind, value) {
        (ControlKind::Boolean, Some(ControlValue::Boolean(value))) => {
            boolean_control(index, definition.id, value, handler)
        }
        (ControlKind::Text { mode }, Some(ControlValue::Text(value))) => {
            text_control(index, definition.id, value, mode, editing_text, handler)
        }
        (ControlKind::Number { constraints }, Some(ControlValue::Number(value))) => number_control(
            index,
            definition.id,
            value,
            constraints,
            number_draft,
            handler,
        ),
        (ControlKind::Enum { options }, Some(ControlValue::Enum(selected))) => {
            enum_control(index, definition.id, options, &selected, &handler)
        }
        _ => div()
            .text_xs()
            .text_color(rgb(theme::error_ink()))
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
        .bg(rgb(if value {
            theme::accent()
        } else {
            theme::line_strong()
        }))
        .when(!value, gpui::Styled::justify_start)
        .hover(move |this| {
            this.bg(rgb(if value {
                theme::accent_hover()
            } else {
                theme::chrome_text_muted()
            }))
        })
        .active(|this| this.bg(rgb(theme::accent_hover())))
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(
            div()
                .size_4()
                .rounded_full()
                .bg(rgb(theme::paper()))
                .shadow_sm(),
        )
        .into_any_element()
}

fn text_control(
    index: usize,
    id: &'static str,
    value: String,
    mode: TextMode,
    editing: bool,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let action = ControlAction::EditText { id };
    let empty = value.is_empty();
    div()
        .id(("hblank-text", index))
        .min_h(rems(if mode == TextMode::Multiline {
            5.0
        } else {
            2.25
        }))
        .w_full()
        .flex()
        .items_center()
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(if editing {
            theme::accent()
        } else {
            theme::line_strong()
        }))
        .bg(rgb(theme::paper()))
        .text_sm()
        .text_color(rgb(if empty {
            theme::text_subtle()
        } else {
            theme::text()
        }))
        .cursor_pointer()
        .hover(move |this| {
            this.border_color(rgb(if editing {
                theme::accent()
            } else {
                theme::chrome_text_muted()
            }))
        })
        .active(|this| this.bg(rgb(theme::surface_subtle())))
        .on_click(move |_, window, cx| handler(&action, window, cx))
        .child(if empty {
            SharedString::from(if mode == TextMode::Multiline {
                "Type multiple lines…"
            } else {
                "Type a value…"
            })
        } else {
            SharedString::from(value)
        })
        .into_any_element()
}

fn number_control(
    index: usize,
    id: &'static str,
    value: f64,
    constraints: NumberConstraints,
    draft: Option<&str>,
    handler: UiHandler<ControlAction>,
) -> AnyElement {
    let decrement = handler.clone();
    let increment = handler.clone();
    let edit = handler;
    let decrement_action = ControlAction::Set {
        id,
        value: ControlValue::Number(stepped_number(value, -constraints.step, constraints)),
    };
    let increment_action = ControlAction::Set {
        id,
        value: ControlValue::Number(stepped_number(value, constraints.step, constraints)),
    };
    let edit_action = ControlAction::EditNumber { id };
    let constraint_label = number_constraint_label(constraints);
    let display = draft.map_or_else(|| format_number(value), str::to_owned);
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(step_button(index * 2, "−", move |window, cx| {
                    decrement(&decrement_action, window, cx);
                }))
                .child(
                    div()
                        .id(("hblank-number", index))
                        .min_w(rems(5.5))
                        .h(rems(2.125))
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_2()
                        .rounded_md()
                        .border_1()
                        .border_color(rgb(if draft.is_some() {
                            theme::accent()
                        } else {
                            theme::line_strong()
                        }))
                        .bg(rgb(theme::surface_subtle()))
                        .text_sm()
                        .text_color(rgb(if display.is_empty() {
                            theme::text_subtle()
                        } else {
                            theme::text()
                        }))
                        .cursor_pointer()
                        .on_click(move |_, window, cx| edit(&edit_action, window, cx))
                        .child(if display.is_empty() {
                            "Type a number…".to_owned()
                        } else {
                            display
                        }),
                )
                .child(step_button(index * 2 + 1, "+", move |window, cx| {
                    increment(&increment_action, window, cx);
                })),
        )
        .when(!constraint_label.is_empty(), |this| {
            this.child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::text_subtle()))
                    .child(constraint_label),
            )
        })
        .into_any_element()
}

fn enum_control(
    index: usize,
    id: &'static str,
    options: &'static [&'static str],
    selected: &str,
    handler: &UiHandler<ControlAction>,
) -> AnyElement {
    let list = options.len() > 4;
    div()
        .flex()
        .gap_1()
        .when(list, |this| this.w_full().flex_col())
        .when(!list, gpui::Styled::flex_wrap)
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
                .when(list, gpui::Styled::w_full)
                .flex()
                .items_center()
                .px_3()
                .rounded_md()
                .border_1()
                .border_color(rgb(if is_selected {
                    theme::accent()
                } else {
                    theme::line_strong()
                }))
                .bg(rgb(if is_selected {
                    theme::accent()
                } else {
                    theme::paper()
                }))
                .text_xs()
                .text_color(rgb(if is_selected {
                    theme::chrome_text()
                } else {
                    theme::accent_ink()
                }))
                .when(is_selected, |this| this.font_weight(FontWeight::SEMIBOLD))
                .cursor_pointer()
                .hover(move |this| {
                    if is_selected {
                        this.bg(rgb(theme::accent_hover()))
                            .border_color(rgb(theme::accent_hover()))
                    } else {
                        this.bg(rgb(theme::accent_wash()))
                            .border_color(rgb(theme::accent()))
                    }
                })
                .active(|this| this.bg(rgb(theme::accent_hover())))
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
        .border_color(rgb(theme::line_strong()))
        .bg(rgb(theme::paper()))
        .text_color(rgb(theme::accent_ink()))
        .cursor_pointer()
        .hover(|this| {
            this.bg(rgb(theme::accent_wash()))
                .border_color(rgb(theme::accent()))
        })
        .active(|this| this.bg(rgb(theme::line())))
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(label)
}

fn stepped_number(value: f64, delta: f64, constraints: NumberConstraints) -> f64 {
    let mut next = value + delta;
    if let Some(min) = constraints.min {
        next = next.max(min);
    }
    if let Some(max) = constraints.max {
        next = next.min(max);
    }
    next
}

fn number_constraint_label(constraints: NumberConstraints) -> String {
    let mut parts = Vec::with_capacity(3);
    if let Some(min) = constraints.min {
        parts.push(format!("min {}", format_number(min)));
    }
    if let Some(max) = constraints.max {
        parts.push(format!("max {}", format_number(max)));
    }
    parts.push(format!("step {}", format_number(constraints.step)));
    parts.join(" · ")
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.4}").trim_end_matches('0').to_owned()
    }
}

pub struct DocsPanelProps {
    pub title: SharedString,
    pub blocks: Vec<AnyElement>,
}

#[must_use]
pub fn docs_panel(props: DocsPanelProps) -> Div {
    div()
        .w(rems(28.0))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(theme::paper()))
        .border_l_1()
        .border_color(rgb(theme::line()))
        .child(
            div()
                .h(rems(3.0))
                .flex_none()
                .flex()
                .items_center()
                .px_4()
                .border_b_1()
                .border_color(rgb(theme::line()))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme::text()))
                .child("COMPONENT DOCUMENTATION"),
        )
        .child(
            div()
                .id("hblank-docs-scroll")
                .flex_1()
                .min_h_0()
                .overflow_scroll()
                .p_5()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .mb_1()
                        .text_xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::text()))
                        .child(props.title),
                )
                .children(props.blocks),
        )
}

#[must_use]
pub fn doc_heading(level: u8, text: impl Into<SharedString>) -> AnyElement {
    let heading = div()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(theme::text()))
        .child(text.into());
    match level {
        1 => heading.text_xl(),
        2 => heading.text_lg(),
        _ => heading.text_sm(),
    }
    .into_any_element()
}

#[must_use]
pub fn doc_prose(text: impl Into<SharedString>) -> AnyElement {
    div()
        .text_sm()
        .text_color(rgb(theme::text_muted()))
        .child(text.into())
        .into_any_element()
}

#[must_use]
pub fn doc_callout(
    tone: CalloutTone,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
) -> AnyElement {
    let (border, background) = match tone {
        CalloutTone::Note => (theme::accent(), theme::accent_wash()),
        CalloutTone::Success => (theme::success(), theme::surface_subtle()),
        CalloutTone::Warning => (theme::error(), theme::surface_subtle()),
    };
    div()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(background))
        .child(
            div()
                .mb_2()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme::text()))
                .child(title.into()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(theme::text_muted()))
                .child(body.into()),
        )
        .into_any_element()
}

#[must_use]
pub fn doc_fixture(label: impl Into<SharedString>, preview: AnyElement) -> AnyElement {
    div()
        .rounded_lg()
        .border_1()
        .border_color(rgb(theme::line()))
        .bg(rgb(theme::canvas()))
        .child(
            div()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(rgb(theme::line()))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme::text_subtle()))
                .child(label.into()),
        )
        .child(
            div()
                .min_h(rems(8.0))
                .p_4()
                .flex()
                .items_center()
                .justify_center()
                .child(preview),
        )
        .into_any_element()
}

#[must_use]
pub fn doc_props(definitions: &'static [ControlDefinition]) -> AnyElement {
    let rows = definitions.iter().map(|definition| {
        div()
            .py_2()
            .border_b_1()
            .border_color(rgb(theme::line()))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(theme::text()))
                            .child(definition.label),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme::text_subtle()))
                            .child(definition.kind.name()),
                    ),
            )
            .when(!definition.docs.is_empty(), |this| {
                this.child(
                    div()
                        .mt_1()
                        .text_xs()
                        .text_color(rgb(theme::text_muted()))
                        .child(definition.docs),
                )
            })
    });
    div()
        .child(doc_heading(2, "Properties"))
        .children(rows)
        .into_any_element()
}

#[must_use]
pub fn doc_controls(
    props: ControlsPanelProps<'_>,
    on_action: &UiHandler<ControlAction>,
) -> AnyElement {
    let rows = props
        .definitions
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            let value = props.props.control_value(definition.id);
            let number_draft = props
                .editing_number
                .and_then(|(id, draft)| (id == definition.id).then_some(draft));
            control_row(
                index,
                definition,
                value,
                props.editing_text == Some(definition.id),
                number_draft,
                on_action.clone(),
            )
        });
    div()
        .child(doc_heading(2, "Live controls"))
        .rounded_lg()
        .border_1()
        .border_color(rgb(theme::line()))
        .children(rows)
        .into_any_element()
}

#[must_use]
pub fn doc_source(source: impl Into<SharedString>) -> AnyElement {
    div()
        .p_3()
        .rounded_lg()
        .bg(rgb(theme::surface_subtle()))
        .text_xs()
        .text_color(rgb(theme::text_subtle()))
        .child(source.into())
        .into_any_element()
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
        .bg(rgb(theme::canvas()))
        .child(
            div()
                .w(rems(26.25))
                .p_8()
                .flex()
                .flex_col()
                .items_center()
                .rounded_xl()
                .bg(rgb(theme::paper()))
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
                        .bg(rgb(theme::accent()))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(theme::chrome_text()))
                        .child("H"),
                )
                .child(
                    div()
                        .mb_2()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::text()))
                        .child(props.title),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme::text_muted()))
                        .child(props.body),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::{NavigationComponent, NavigationVariant, navigation_matches};

    fn component() -> NavigationComponent {
        NavigationComponent {
            id: "src/button.rs#button".into(),
            title: "Button",
            group: "Inputs",
            variants: vec![NavigationVariant {
                id: "src/button.rs#loading".into(),
                title: "Loading",
            }],
        }
    }

    #[test]
    fn nested_navigation_filters_group_component_and_variant() {
        let component = component();
        let variant = &component.variants[0];

        assert!(navigation_matches(&component, variant, "inputs"));
        assert!(navigation_matches(&component, variant, "button"));
        assert!(navigation_matches(&component, variant, "loading"));
        assert!(!navigation_matches(&component, variant, "card"));
    }
}

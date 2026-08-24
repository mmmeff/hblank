#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

//! Dogfooding fixtures for Hblank's own presentational components.

use gpui::{App, Div, FontWeight, Window, div, prelude::*, px, rgb};
use hblank::{HblankEnum, HblankProps};

#[derive(Clone, Copy, Debug, Default, HblankEnum)]
pub enum Accent {
    #[default]
    Violet,
    Jade,
    Ember,
}

#[derive(Clone, Debug, HblankProps)]
pub struct FixtureCardProps {
    /// Controls whether the card appears active.
    pub active: bool,
    /// Main label rendered by the card.
    #[hblank(multiline)]
    pub label: String,
    /// Number shown in the card's presentation data.
    #[hblank(min = 0, max = 10, step = 1)]
    pub count: u32,
    /// Accent treatment used by the card.
    pub accent: Accent,
}

impl Default for FixtureCardProps {
    fn default() -> Self {
        Self {
            active: true,
            label: "Hot reload verified".to_owned(),
            count: 3,
            accent: Accent::Violet,
        }
    }
}

/// A state-free GPUI card used to exercise every automatic Hblank control kind.
#[must_use]
pub fn fixture_card(props: &FixtureCardProps, _window: &mut Window, _cx: &mut App) -> Div {
    let accent = match props.accent {
        Accent::Violet => rgb(0x7357d8),
        Accent::Jade => rgb(0x258b63),
        Accent::Ember => rgb(0xc65d3b),
    };
    div()
        .w(px(360.0))
        .p_6()
        .rounded_lg()
        .border_1()
        .border_color(if props.active { accent } else { rgb(0xd8d8d3) })
        .bg(rgb(0xffffff))
        .text_color(rgb(0x29292d))
        .child(
            div()
                .mb_2()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child(props.label.clone()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .text_sm()
                .text_color(rgb(0x6e6e76))
                .child(if props.active { "Active" } else { "Inactive" })
                .child(format!("Count {}", props.count)),
        )
}

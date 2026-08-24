#![allow(clippy::unreadable_literal)] // Six-digit RGB values remain recognizable as design tokens.

//! Dogfooding fixtures for Hblank's own presentational components.

use gpui::{App, Div, FontWeight, Global, Window, div, prelude::*, px, rgb};
use hblank::{HblankEnum, HblankProps, ResolvedTheme, ThemeMode};

#[derive(Clone, Copy)]
pub struct DogfoodTheme(pub ResolvedTheme);

impl Global for DogfoodTheme {}

#[hblank::theme_hook]
pub fn apply_hblank_theme(_mode: ThemeMode, resolved: ResolvedTheme, cx: &mut App) {
    cx.set_global(DogfoodTheme(resolved));
}

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
pub fn fixture_card(props: &FixtureCardProps, _window: &mut Window, cx: &mut App) -> Div {
    let dark = cx
        .try_global::<DogfoodTheme>()
        .is_some_and(|theme| theme.0 == ResolvedTheme::Dark);
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
        .bg(rgb(if dark { 0x24242c } else { 0xffffff }))
        .text_color(rgb(if dark { 0xf8f8f6 } else { 0x29292d }))
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
                .text_color(rgb(if dark { 0xb0b0bb } else { 0x6e6e76 }))
                .child(if props.active { "Active" } else { "Inactive" })
                .child(format!("Count {}", props.count)),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn registers_configured_theme_hook_path() {
        assert!(hblank::registered_theme_hook("hblank_dogfood::apply_hblank_theme").is_some());
    }
}

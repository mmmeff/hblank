use crate::{ResolvedTheme, ThemeMode, gpui::App};

pub type ThemeHook = fn(ThemeMode, ResolvedTheme, &mut App);

#[doc(hidden)]
pub struct ThemeHookRegistration {
    pub id: &'static str,
    pub apply: ThemeHook,
}

inventory::collect!(ThemeHookRegistration);

#[must_use]
pub fn registered_theme_hook(id: &str) -> Option<ThemeHook> {
    inventory::iter::<ThemeHookRegistration>
        .into_iter()
        .find(|registration| registration.id == id)
        .map(|registration| registration.apply)
}

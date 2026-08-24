use crate::{ResolvedTheme, ThemeMode, gpui::AnyElement};

#[derive(Clone, Copy, Debug)]
pub struct DocContext<'a> {
    pub component_id: &'a str,
    pub component_title: &'a str,
    pub fixture_id: &'a str,
    pub fixture_title: &'a str,
    pub theme_mode: ThemeMode,
    pub resolved_theme: ResolvedTheme,
}

pub type CustomDocRenderer = for<'a> fn(&DocContext<'a>, &str) -> AnyElement;

#[doc(hidden)]
pub struct CustomDocBlockRegistration {
    pub id: &'static str,
    pub render: CustomDocRenderer,
}

inventory::collect!(CustomDocBlockRegistration);

#[must_use]
pub fn registered_doc_block(id: &str) -> Option<CustomDocRenderer> {
    inventory::iter::<CustomDocBlockRegistration>
        .into_iter()
        .find(|registration| registration.id == id)
        .map(|registration| registration.render)
}

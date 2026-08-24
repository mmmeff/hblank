use crate::gpui::{AnyElement, IntoElement};

pub struct Rendered<Handle, Content> {
    content: Content,
    handle: Handle,
}

impl<Handle, Content> Rendered<Handle, Content> {
    #[must_use]
    pub const fn new(content: Content, handle: Handle) -> Self {
        Self { content, handle }
    }

    #[must_use]
    pub fn into_parts(self) -> (Content, Handle) {
        (self.content, self.handle)
    }
}

impl<Handle, Content: IntoElement> Rendered<Handle, Content> {
    #[must_use]
    pub fn into_erased_parts(self) -> (AnyElement, Handle) {
        (self.content.into_any_element(), self.handle)
    }
}

impl<Handle, Content: IntoElement> IntoElement for Rendered<Handle, Content> {
    type Element = Content::Element;

    fn into_element(self) -> Self::Element {
        self.content.into_element()
    }
}

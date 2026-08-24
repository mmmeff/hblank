#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalloutTone {
    Note,
    Success,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocBlock {
    Heading {
        level: u8,
        text: String,
    },
    Prose(String),
    Fixture {
        id: String,
    },
    Props,
    Controls,
    Source,
    Callout {
        tone: CalloutTone,
        title: String,
        body: String,
    },
    Custom {
        id: String,
        payload: String,
    },
}

impl DocBlock {
    #[must_use]
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        Self::Heading {
            level: level.clamp(1, 3),
            text: text.into(),
        }
    }

    #[must_use]
    pub fn prose(text: impl Into<String>) -> Self {
        Self::Prose(text.into())
    }

    #[must_use]
    pub fn fixture(id: impl Into<String>) -> Self {
        Self::Fixture { id: id.into() }
    }

    #[must_use]
    pub const fn props() -> Self {
        Self::Props
    }

    #[must_use]
    pub const fn controls() -> Self {
        Self::Controls
    }

    #[must_use]
    pub const fn source() -> Self {
        Self::Source
    }

    #[must_use]
    pub fn custom(id: impl Into<String>, payload: impl Into<String>) -> Self {
        Self::Custom {
            id: id.into(),
            payload: payload.into(),
        }
    }

    #[must_use]
    pub fn callout(tone: CalloutTone, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::Callout {
            tone,
            title: title.into(),
            body: body.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocPage {
    blocks: Vec<DocBlock>,
}

impl DocPage {
    #[must_use]
    pub fn new(blocks: impl IntoIterator<Item = DocBlock>) -> Self {
        Self {
            blocks: blocks.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn blocks(&self) -> &[DocBlock] {
        &self.blocks
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_typed_document_blocks_in_order() {
        let page = DocPage::new([
            DocBlock::heading(1, "Button"),
            DocBlock::prose("Use for primary actions."),
            DocBlock::props(),
            DocBlock::controls(),
            DocBlock::source(),
            DocBlock::custom("project::tokens", "spacing"),
        ]);

        assert_eq!(page.blocks().len(), 6);
        assert_eq!(
            page.blocks()[0],
            DocBlock::Heading {
                level: 1,
                text: "Button".to_owned(),
            }
        );
    }
}

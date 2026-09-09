use std::{collections::BTreeMap, rc::Rc};

use hblank::gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*, rems};
use hblank::harness::input::{InputEvent, TextInput};
use hblank::harness::{ControlAction, ControlsPanelProps, UiHandler, controls_panel, doc_controls};
use hblank::{ControlValue, HblankEnum, HblankProps};

#[derive(Clone, Copy, Debug, Default, HblankEnum)]
enum ControlTone {
    #[default]
    Violet,
    Jade,
    Amber,
}

#[derive(Clone, Debug, HblankProps)]
pub(crate) struct ControlProps {
    /// Whether the sample is active.
    active: bool,
    /// Text shown by the sample.
    label: String,
    /// Number of visible markers.
    #[hblank(min = 0, max = 12, step = 1)]
    count: u32,
    /// Color treatment used by the sample.
    tone: ControlTone,
}

impl Default for ControlProps {
    fn default() -> Self {
        Self {
            active: true,
            label: "Editable properties".to_owned(),
            count: 4,
            tone: ControlTone::Violet,
        }
    }
}

pub(crate) struct ControlsPreview {
    external: ControlProps,
    props: ControlProps,
    inputs: BTreeMap<&'static str, Entity<TextInput>>,
    inline: bool,
    error: Option<String>,
}

impl ControlsPreview {
    pub(crate) fn new(props: ControlProps, inline: bool, cx: &mut Context<Self>) -> Self {
        let mut inputs = BTreeMap::new();
        for (id, text, placeholder) in [
            ("label", props.label.clone(), "Type a value…"),
            ("count", props.count.to_string(), "Type a number…"),
        ] {
            let input = cx.new(|cx| TextInput::new(text, placeholder, false, cx));
            cx.subscribe(&input, move |this, input, event, cx| {
                if !matches!(event, InputEvent::Changed) {
                    return;
                }
                let text = input.read(cx).text().to_owned();
                let value = if id == "count" {
                    let Ok(number) = text.parse::<f64>() else {
                        this.error = Some("Enter a valid number".to_owned());
                        cx.notify();
                        return;
                    };
                    ControlValue::Number(number)
                } else {
                    ControlValue::Text(text)
                };
                this.error = this
                    .props
                    .set_control(id, value)
                    .err()
                    .map(|error| error.to_string());
                cx.notify();
            })
            .detach();
            inputs.insert(id, input);
        }
        Self {
            external: props.clone(),
            props,
            inputs,
            inline,
            error: None,
        }
    }

    pub(crate) fn sync(&mut self, props: &ControlProps, cx: &mut Context<Self>) {
        let mut changed = false;
        for definition in props.definitions() {
            let value = props.control_value(definition.id);
            if self.external.control_value(definition.id) != value
                && let Some(value) = value
            {
                self.error = self
                    .props
                    .set_control(definition.id, value)
                    .err()
                    .map(|error| error.to_string());
                if self.error.is_none() {
                    self.sync_input(definition.id, cx);
                }
                changed = true;
            }
        }
        if changed {
            self.external.clone_from(props);
            cx.notify();
        }
    }

    fn sync_input(&self, id: &str, cx: &mut Context<Self>) {
        let Some(input) = self.inputs.get(id) else {
            return;
        };
        let text = match self.props.control_value(id) {
            Some(ControlValue::Text(text)) => text,
            Some(ControlValue::Number(value)) => value.to_string(),
            _ => return,
        };
        input.update(cx, |input, cx| input.set_text(text, cx));
    }

    fn on_control(&mut self, action: &ControlAction, _: &mut Window, cx: &mut Context<Self>) {
        match action {
            ControlAction::Set { id, value } => {
                self.error = self
                    .props
                    .set_control(id, value.clone())
                    .err()
                    .map(|error| error.to_string());
                if self.error.is_none() {
                    self.sync_input(id, cx);
                }
            }
            ControlAction::Reset => {
                self.props = self.external.clone();
                self.error = None;
                for id in self.inputs.keys() {
                    self.sync_input(id, cx);
                }
            }
        }
        cx.notify();
    }
}

impl Render for ControlsPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handler: UiHandler<ControlAction> = Rc::new(cx.listener(Self::on_control));
        let props = ControlsPanelProps {
            definitions: self.props.definitions(),
            props: &self.props,
            inputs: &self.inputs,
        };
        let panel = if self.inline {
            doc_controls(props, &handler)
        } else {
            controls_panel(props, handler).into_any_element()
        };
        div()
            .w_full()
            .when(!self.inline, |this| this.w(rems(20.0)).h(rems(30.0)))
            .child(panel)
            .when_some(self.error.clone(), |this, error| {
                this.child(div().text_xs().child(error))
            })
    }
}

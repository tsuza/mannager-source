use std::ops::RangeInclusive;

use iced::{Element, advanced::text, widget::container};

use crate::ui::components::{
    metered_progress_bar::MeteredProgressBar,
    tooltip::{Position, Tooltip},
};

pub mod metered_progress_bar;
pub mod modal;
pub mod notification;
pub mod selectable_text;
pub mod textinput_terminal;
pub mod tooltip;
pub mod typed_input;

pub fn metered_progress_bar<'a, Theme>(
    range: RangeInclusive<f32>,
    value: f32,
) -> MeteredProgressBar<'a, Theme>
where
    Theme: metered_progress_bar::Catalog + 'a,
{
    MeteredProgressBar::new(range, value)
}

pub fn tooltip<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    position: Position,
) -> Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: text::Renderer,
{
    Tooltip::new(content, tooltip, position)
}

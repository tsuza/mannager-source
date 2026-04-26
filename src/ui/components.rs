use std::ops::RangeInclusive;

use crate::ui::components::metered_progress_bar::MeteredProgressBar;

pub mod metered_progress_bar;
pub mod modal;
pub mod notification;
pub mod spinner;
pub mod textinput_terminal;
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

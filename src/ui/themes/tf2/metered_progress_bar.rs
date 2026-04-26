use crate::ui::components::metered_progress_bar::{Catalog, Style, StyleFn};
use iced::{border, color};

use super::super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(_theme: &Theme) -> Style {
    Style {
        background: iced::Background::Color(color!(0x272422)),
        bar: iced::Background::Color(color!(0xffffff)),
        border: border::width(2).color(color!(0x3a3630)),
    }
}

use iced::{Background, Color, border};
use iced_aw::menu::{Catalog, Style};
use iced_aw::style::{Status, StyleFn};

use crate::ui::themes::tf2::container;

use super::super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self, Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, _status: Status) -> Style {
    Style {
        bar_background: Background::Color(Color::TRANSPARENT),
        bar_border: border::rounded(0),
        menu_background: Background::Color(theme.colors().surface.surface_container.lowest),
        menu_border: container::outlined(theme).border.width(5),
        ..Default::default()
    }
}

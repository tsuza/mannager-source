use iced::{Background, Color, Vector, border, color};
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
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;

    Style {
        bar_background: Background::Color(Color::TRANSPARENT),
        bar_border: border::rounded(10),

        menu_background: Background::Color(surface.surface_container.base),
        menu_border: border::Border {
            color: outline.color,
            width: 1.0,
            radius: 10.into(),
        },

        menu_shadow: iced::Shadow {
            color: color!(0, 0, 0, 0.45),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 16.0,
        },

        ..Default::default()
    }
}

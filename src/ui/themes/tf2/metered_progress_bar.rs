use crate::ui::components::metered_progress_bar::{Catalog, Style, StyleFn};
use iced::border;

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

pub fn default(theme: &Theme) -> Style {
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;
    let primary = theme.colors().primary;

    Style {
        background: iced::Background::Color(surface.surface_container.lowest),
        bar: iced::Background::Color(primary.color),
        border: border::Border {
            color: outline.color,
            width: 1.5,
            radius: 10.into(),
        },
    }
}

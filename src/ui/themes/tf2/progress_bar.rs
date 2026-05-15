use iced::Border;
use iced::widget::progress_bar::{Catalog, Style, StyleFn};

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
    let primary = theme.colors().primary;
    let surface = theme.colors().surface;

    Style {
        background: surface.surface_container.base.into(),
        border: Border {
            color: theme.colors().outline.color,
            width: 1.0,
            radius: 10.into(),
        },
        bar: primary.color.into(),
    }
}

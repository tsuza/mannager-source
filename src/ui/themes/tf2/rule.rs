use iced::border::Radius;
use iced::widget::rule::{Catalog, FillMode, Style, StyleFn};

use super::super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(full_width)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn inset(theme: &Theme) -> Style {
    let outline = theme.colors().outline;

    Style {
        color: outline.variant,
        fill_mode: FillMode::Padded(10),
        radius: Radius::from(8),
        snap: true,
    }
}

pub fn full_width(theme: &Theme) -> Style {
    let outline = theme.colors().outline;

    Style {
        color: outline.variant,
        fill_mode: FillMode::Full,
        radius: Radius::from(0),
        snap: true,
    }
}

pub fn primary(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        color: primary.color,
        fill_mode: FillMode::Full,
        radius: Radius::from(0),
        snap: true,
    }
}

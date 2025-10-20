use super::super::{Theme, disabled_text};
use iced::{Background, Color};
use iced_aw::style::number_input::{Catalog, ExtendedCatalog, Style};
use iced_aw::style::{Status, StyleFn};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self, Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

impl ExtendedCatalog for Theme {
    fn style(&self, class: &<Self as self::Catalog>::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;

    let active = Style {
        button_background: Some(Background::Color(surface.surface_container.highest)),
        icon_color: surface.on_surface,
    };

    match status {
        Status::Disabled => Style {
            button_background: Some(Background::Color(Color::TRANSPARENT)),
            icon_color: disabled_text(surface.on_surface),
        },
        _ => active,
    }
}

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
    let primary = theme.colors().primary;
    let surface = theme.colors().surface;

    let active = Style {
        button_background: Some(surface.container.lowest.into()),
        icon_color: primary.text,
    };

    match status {
        Status::Disabled => Style {
            button_background: Some(Color::TRANSPARENT.into()),
            icon_color: disabled_text(surface.text),
        },
        _ => active,
    }
}

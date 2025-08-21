use iced::widget::text_input::{Catalog, Status, Style, StyleFn};
use iced::{Background, Border, Color};

use super::super::{Theme, disabled_container, disabled_text};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;
    let primary = theme.colors().primary;

    let active = Style {
        background: Background::Color(surface.surface_container.low),
        border: Border {
            color: theme.colors().outline.color,
            width: 1.0,
            radius: 3.into(),
        },
        icon: surface.on_surface_variant,
        placeholder: surface.on_surface_variant,
        value: surface.on_surface,
        selection: disabled_text(primary.color),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: active.border.color(surface.on_surface),
            ..active
        },
        Status::Disabled => Style {
            background: Color::TRANSPARENT.into(),
            border: Border {
                color: disabled_container(surface.on_surface),
                ..active.border
            },
            icon: disabled_text(surface.on_surface),
            placeholder: disabled_text(surface.on_surface),
            value: disabled_text(surface.on_surface),
            selection: disabled_text(surface.on_surface),
        },
        Status::Focused { .. } => Style {
            border: Border {
                color: primary.color,
                width: 3.0,
                ..active.border
            },
            ..active
        },
    }
}

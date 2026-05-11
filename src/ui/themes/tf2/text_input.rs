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
    let outline = theme.colors().outline;
    let primary = theme.colors().primary;

    let active_border = Border {
        color: outline.color,
        width: 1.0,
        radius: 8.into(),
    };

    let focused_border = active_border.color(primary.color);

    let active = Style {
        background: Background::Color(surface.surface_container.lowest),
        border: active_border,
        icon: surface.on_surface_variant,
        placeholder: surface.on_surface_variant,
        value: primary.on_primary,
        selection: primary.color.scale_alpha(0.35),
    };

    match status {
        Status::Active | Status::Hovered => active,

        Status::Focused { .. } => Style {
            border: focused_border,
            ..active
        },

        Status::Disabled => Style {
            background: Color::TRANSPARENT.into(),
            border: Border {
                color: disabled_container(surface.on_surface),
                width: 1.5,
                radius: 10.into(),
            },
            icon: disabled_text(surface.on_surface),
            placeholder: disabled_text(surface.on_surface),
            value: disabled_text(surface.on_surface),
            selection: disabled_text(surface.on_surface),
        },
    }
}

pub fn terminal(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;
    let primary = theme.colors().primary;

    let active_border = Border {
        color: outline.color,
        width: 1.0,
        radius: 8.into(),
    };

    let active = Style {
        background: Background::Color(surface.surface_container.base),
        border: active_border,
        icon: surface.on_surface_variant,
        placeholder: surface.on_surface_variant,
        value: surface.on_surface,
        selection: primary.color.scale_alpha(0.35),
    };

    match status {
        Status::Active | Status::Hovered | Status::Focused { .. } => active,

        Status::Disabled => Style {
            background: Color::TRANSPARENT.into(),
            border: Border {
                color: disabled_container(surface.on_surface),
                width: 1.5,
                radius: 10.into(),
            },
            icon: disabled_text(surface.on_surface),
            placeholder: disabled_text(surface.on_surface),
            value: disabled_text(surface.on_surface),
            selection: disabled_text(surface.on_surface),
        },
    }
}

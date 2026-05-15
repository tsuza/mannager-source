use iced::widget::checkbox::{Catalog, Status, Style, StyleFn};
use iced::{Background, Border};

use crate::ui::themes::Theme;

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
    let palette = theme.colors();
    let surface = palette.surface;
    let primary = palette.primary;
    let outline = palette.outline;
    let secondary = palette.secondary;

    match status {
        Status::Active { is_checked: false } => Style {
            background: Background::Color(surface.surface_container.lowest),
            icon_color: surface.surface_container.lowest, // invisible
            border: Border {
                color: outline.color,
                width: 1.5,
                radius: 4.0.into(),
            },
            text_color: Some(surface.on_surface_variant),
        },

        Status::Active { is_checked: true } => Style {
            background: Background::Color(primary.color),
            icon_color: primary.on_primary,
            border: Border {
                color: primary.color,
                width: 1.5,
                radius: 4.0.into(),
            },
            text_color: Some(surface.on_surface),
        },

        Status::Hovered { is_checked: false } => Style {
            background: Background::Color(surface.surface_container.base),
            icon_color: surface.surface_container.base, // invisible
            border: Border {
                color: outline.variant,
                width: 1.5,
                radius: 4.0.into(),
            },
            text_color: Some(surface.on_surface),
        },

        Status::Hovered { is_checked: true } => Style {
            background: Background::Color(primary.on_primary_container),
            icon_color: primary.on_primary,
            border: Border {
                color: primary.on_primary_container,
                width: 1.5,
                radius: 4.0.into(),
            },
            text_color: Some(surface.on_surface),
        },

        Status::Disabled { is_checked: false } => Style {
            background: Background::Color(surface.surface_container.lowest),
            icon_color: surface.surface_container.lowest, // invisible
            border: Border {
                color: surface.surface_container.base,
                width: 1.5,
                radius: 4.0.into(),
            },
            text_color: Some(secondary.color),
        },

        Status::Disabled { is_checked: true } => Style {
            background: Background::Color(primary.primary_container),
            icon_color: secondary.on_secondary_container,
            border: Border {
                color: primary.primary_container,
                width: 1.5,
                radius: 4.0.into(),
            },
            text_color: Some(secondary.color),
        },
    }
}

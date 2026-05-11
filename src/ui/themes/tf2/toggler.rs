use iced::{Background, Color};
use sweeten::widget::toggler::{Catalog, Status, Style, StyleFn};

use super::super::Theme;
use super::super::{HOVERED_LAYER_OPACITY, disabled_container, disabled_text, mix};

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
    let secondary = theme.colors().secondary;
    let outline = theme.colors().outline;

    match status {
        Status::Active { is_toggled } => {
            if is_toggled {
                Style {
                    background: Background::Color(primary.color),
                    foreground: Background::Color(primary.on_primary),
                    text_color: Some(primary.on_primary),
                    background_border_color: Color::TRANSPARENT,
                    foreground_border_color: Color::TRANSPARENT,
                    background_border_width: 0.0,
                    foreground_border_width: 0.0,
                    padding_ratio: 0.2,
                    border_radius: None,
                }
            } else {
                Style {
                    background: Background::Color(secondary.secondary_container),
                    foreground: Background::Color(surface.on_surface_variant),
                    text_color: Some(surface.on_surface),
                    background_border_color: outline.color,
                    foreground_border_color: Color::TRANSPARENT,
                    background_border_width: 1.0,
                    foreground_border_width: 0.0,
                    padding_ratio: 0.2,
                    border_radius: None,
                }
            }
        }

        Status::Hovered { is_toggled } => {
            if is_toggled {
                Style {
                    background: Background::Color(primary.primary_container),
                    foreground: Background::Color(primary.color),
                    text_color: Some(primary.on_primary),
                    ..default(theme, Status::Active { is_toggled })
                }
            } else {
                Style {
                    background: Background::Color(surface.surface_container.high),
                    foreground: Background::Color(surface.on_surface_variant),
                    text_color: Some(surface.on_surface),
                    ..default(theme, Status::Active { is_toggled })
                }
            }
        }

        Status::Disabled { is_toggled } => Style {
            background: Background::Color(surface.surface_container.lowest),
            foreground: Background::Color(surface.on_surface_variant),
            text_color: Some(surface.on_surface_variant),
            background_border_color: outline.variant,
            foreground_border_color: Color::TRANSPARENT,
            background_border_width: 1.0,
            foreground_border_width: 0.0,
            padding_ratio: 0.2,
            border_radius: None,
        },
    }
}

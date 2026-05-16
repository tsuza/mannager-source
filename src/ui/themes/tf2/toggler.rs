use iced::Color;
use sweeten::widget::toggler::{Catalog, Status, Style, StyleFn};

use super::super::Theme;

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
                    background: primary.color.into(),
                    foreground: primary.text.into(),
                    text_color: Some(primary.text),
                    background_border_color: Color::TRANSPARENT,
                    foreground_border_color: Color::TRANSPARENT,
                    background_border_width: 0.0,
                    foreground_border_width: 0.0,
                    padding_ratio: 0.2,
                    border_radius: None,
                }
            } else {
                Style {
                    background: secondary.container.into(),
                    foreground: surface.text_variant.into(),
                    text_color: Some(surface.text),
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
                    background: primary.container.into(),
                    foreground: primary.color.into(),
                    text_color: Some(primary.text),
                    ..default(theme, Status::Active { is_toggled })
                }
            } else {
                Style {
                    background: surface.container.high.into(),
                    foreground: surface.text_variant.into(),
                    text_color: Some(surface.text),
                    ..default(theme, Status::Active { is_toggled })
                }
            }
        }

        Status::Disabled { .. } => Style {
            background: surface.container.lowest.into(),
            foreground: surface.text_variant.into(),
            text_color: Some(surface.text_variant),
            background_border_color: outline.variant,
            foreground_border_color: Color::TRANSPARENT,
            background_border_width: 1.0,
            foreground_border_width: 0.0,
            padding_ratio: 0.2,
            border_radius: None,
        },
    }
}

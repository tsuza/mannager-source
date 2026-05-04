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

pub fn styled(
    background: Background,
    foreground: Background,
    text_color: Color,
    border: Option<Color>,
) -> Style {
    Style {
        background,
        background_border_width: if border.is_some() { 1.5 } else { 0.0 },
        background_border_color: border.unwrap_or(Color::TRANSPARENT),
        foreground,
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: Some(text_color),
        border_radius: None,
        padding_ratio: 0.2,
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
                styled(
                    primary.color.into(),
                    primary.on_primary.into(),
                    primary.on_primary,
                    None,
                )
            } else {
                styled(
                    secondary.secondary_container.into(),
                    surface.on_surface_variant.into(),
                    surface.on_surface,
                    Some(outline.variant),
                )
            }
        }

        Status::Hovered { is_toggled } => {
            if is_toggled {
                styled(
                    mix(primary.color, primary.on_primary, HOVERED_LAYER_OPACITY).into(),
                    primary.on_primary.into(),
                    primary.on_primary,
                    None,
                )
            } else {
                styled(
                    mix(
                        secondary.secondary_container,
                        surface.on_surface,
                        HOVERED_LAYER_OPACITY,
                    )
                    .into(),
                    surface.on_surface.into(),
                    surface.on_surface,
                    Some(outline.color),
                )
            }
        }

        Status::Disabled { is_toggled } => {
            if is_toggled {
                styled(
                    disabled_container(primary.color).into(),
                    disabled_text(primary.on_primary).into(),
                    surface.on_surface,
                    None,
                )
            } else {
                styled(
                    disabled_container(secondary.secondary_container).into(),
                    disabled_text(surface.on_surface).into(),
                    surface.on_surface,
                    Some(disabled_text(surface.on_surface)),
                )
            }
        }
    }
}

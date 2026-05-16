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
            background: surface.container.lowest.into(),
            icon_color: surface.container.lowest,
            border: Border {
                color: outline.color,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(surface.text_variant),
        },

        Status::Active { is_checked: true } => Style {
            background: primary.color.into(),
            icon_color: primary.text,
            border: Border {
                color: primary.color,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(surface.text),
        },

        Status::Hovered { is_checked: false } => Style {
            background: surface.container.base.into(),
            icon_color: surface.container.base,
            border: Border {
                color: outline.variant,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(surface.text),
        },

        Status::Hovered { is_checked: true } => Style {
            background: primary.container.into(),
            icon_color: primary.text,
            border: Border {
                color: primary.container_text,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(surface.text),
        },

        Status::Disabled { is_checked: false } => Style {
            background: surface.container.lowest.into(),
            icon_color: surface.container.lowest,
            border: Border {
                color: surface.container.base,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(secondary.color),
        },

        Status::Disabled { is_checked: true } => Style {
            background: Background::Color(primary.container),
            icon_color: secondary.container_text,
            border: Border {
                color: primary.container,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(secondary.color),
        },
    }
}

use iced::widget::text_input::{Catalog, Status, Style, StyleFn};
use iced::{Border, Color};

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
        background: surface.container.lowest.into(),
        border: active_border,
        icon: surface.text_variant,
        placeholder: surface.text_variant,
        value: primary.text,
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
                color: disabled_container(surface.text),
                width: 1.5,
                radius: 10.into(),
            },
            icon: disabled_text(surface.text),
            placeholder: disabled_text(surface.text),
            value: disabled_text(surface.text),
            selection: disabled_text(surface.text),
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
        background: surface.container.base.into(),
        border: active_border,
        icon: surface.text_variant,
        placeholder: surface.text_variant,
        value: surface.text,
        selection: primary.color.scale_alpha(0.35),
    };

    match status {
        Status::Active | Status::Hovered | Status::Focused { .. } => active,

        Status::Disabled => Style {
            background: Color::TRANSPARENT.into(),
            border: Border {
                color: disabled_container(surface.text),
                width: 1.5,
                radius: 10.into(),
            },
            icon: disabled_text(surface.text),
            placeholder: disabled_text(surface.text),
            value: disabled_text(surface.text),
            selection: disabled_text(surface.text),
        },
    }
}

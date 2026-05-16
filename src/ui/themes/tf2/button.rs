use iced::widget::button::{Catalog, Status, Style, StyleFn};
use iced::{Background, Border, Color, border};

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
    let primary = theme.colors().primary;
    let secondary = theme.colors().secondary;
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;

    let base = Style {
        background: Some(Background::Color(surface.container.lowest)),
        text_color: secondary.color,
        border: border::color(outline.color).rounded(8).width(1),
        snap: true,
        ..Default::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(primary.color.scale_alpha(0.1))),
            text_color: primary.container_text,
            border: base.border.color(primary.color),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(primary.color.scale_alpha(0.2))),
            text_color: primary.container_text,
            border: base.border.color(primary.color),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(surface.container.lowest)),
            text_color: surface.text_variant,
            border: Border {
                color: outline.variant,
                ..base.border
            },
            ..base
        },
    }
}

pub fn primary(theme: &Theme, status: Status) -> Style {
    let primary = theme.colors().primary;
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;

    let base = Style {
        background: Some(Background::Color(primary.color)),
        text_color: primary.text,
        border: border::color(outline.color).rounded(8).width(1),
        snap: true,
        ..Default::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => Style {
            background: Some(Background::Color(primary.color.scale_alpha(1.2))),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(surface.container.lowest)),
            text_color: surface.text_variant,
            border: Border {
                color: outline.variant,
                ..base.border
            },
            ..base
        },
    }
}

pub fn success(theme: &Theme, status: Status) -> Style {
    let primary = theme.colors().primary;
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;
    let success = theme.colors().success;

    let base = Style {
        background: Some(success.color.into()),
        text_color: success.text,
        border: border::color(outline.color).rounded(8).width(1),
        snap: true,
        ..Default::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => Style {
            border: base.border.color(primary.color),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(surface.container.lowest)),
            text_color: surface.text_variant,
            border: Border {
                color: outline.variant,
                ..base.border
            },
            ..base
        },
    }
}

pub fn error(theme: &Theme, status: Status) -> Style {
    let primary = theme.colors().primary;
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;
    let error = theme.colors().error;

    let base = Style {
        background: Some(error.color.into()),
        text_color: error.text,
        border: border::color(outline.color).rounded(8).width(1),
        snap: true,
        ..Default::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => Style {
            border: base.border.color(primary.color),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(surface.container.lowest)),
            text_color: surface.text_variant,
            border: Border {
                color: outline.variant,
                ..base.border
            },
            ..base
        },
    }
}

pub fn text(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;

    let base = Style {
        background: None,
        text_color: surface.text,
        border: border::rounded(8).width(1),
        snap: true,
        ..Default::default()
    };

    match status {
        Status::Hovered => Style {
            background: Some(Color::BLACK.scale_alpha(0.2).into()),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Color::BLACK.scale_alpha(0.4).into()),
            ..base
        },
        _ => base,
    }
}

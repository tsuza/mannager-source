use iced::widget::button::{Catalog, Status, Style, StyleFn};
use iced::{Background, Border, Color, border};

use super::super::Theme;
use super::super::{
    HOVERED_LAYER_OPACITY, PRESSED_LAYER_OPACITY, disabled_container, disabled_text, elevation,
    mix, shadow_from_elevation,
};

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
        background: Some(Background::Color(surface.surface_container.lowest)),
        text_color: secondary.color,
        border: border::color(outline.color).rounded(8).width(1),
        snap: true,
        ..Default::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => Style {
            background: Some(Background::Color(primary.color.scale_alpha(0.1))),
            text_color: primary.on_primary_container,
            border: base.border.color(primary.color),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(surface.surface_container.lowest)),
            text_color: surface.on_surface_variant,
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
        text_color: primary.on_primary,
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
            background: Some(Background::Color(surface.surface_container.lowest)),
            text_color: surface.on_surface_variant,
            border: Border {
                color: outline.variant,
                ..base.border
            },
            ..base
        },
    }
}

fn styled(
    background: Color,
    foreground: Color,
    disabled: Color,
    shadow_color: Color,
    elevation_level: u8,
    status: Status,
) -> Style {
    let base = Style {
        background: Some(Background::Color(background)),
        text_color: foreground,
        border: border::rounded(10),
        shadow: shadow_from_elevation(elevation(elevation_level), shadow_color),
        ..Default::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(mix(
                background,
                foreground,
                HOVERED_LAYER_OPACITY,
            ))),
            shadow: shadow_from_elevation(elevation(elevation_level + 1), shadow_color),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(mix(
                background,
                foreground,
                PRESSED_LAYER_OPACITY * 1.2,
            ))),
            shadow: shadow_from_elevation(
                elevation(elevation_level.saturating_sub(1)),
                shadow_color,
            ),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(disabled_container(disabled))),
            text_color: disabled_text(disabled),
            border: border::rounded(10),
            shadow: Default::default(),
            snap: true,
        },
    }
}

pub fn secondary(theme: &Theme, status: Status) -> Style {
    let c = theme.colors().secondary;
    styled(
        c.secondary_container,
        c.on_secondary_container,
        theme.colors().surface.on_surface,
        theme.colors().shadow,
        0,
        status,
    )
}

pub fn tertiary(theme: &Theme, status: Status) -> Style {
    let c = theme.colors().tertiary;
    styled(
        c.tertiary_container,
        c.on_tertiary_container,
        theme.colors().surface.on_surface,
        theme.colors().shadow,
        0,
        status,
    )
}

pub fn success(theme: &Theme, status: Status) -> Style {
    let primary = theme.colors().primary;
    let surface = theme.colors().surface;
    let outline = theme.colors().outline;
    let success = theme.colors().success;

    let base = Style {
        background: Some(success.color.into()),
        text_color: success.on_success,
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
            background: Some(Background::Color(surface.surface_container.lowest)),
            text_color: surface.on_surface_variant,
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
        text_color: error.on_error,
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
            background: Some(Background::Color(surface.surface_container.lowest)),
            text_color: surface.on_surface_variant,
            border: Border {
                color: outline.variant,
                ..base.border
            },
            ..base
        },
    }
}

pub fn elevated(theme: &Theme, status: Status) -> Style {
    let s = theme.colors().surface;
    styled(
        s.surface_container.low,
        s.on_surface,
        s.on_surface_variant,
        theme.colors().shadow,
        2,
        status,
    )
}

pub fn outlined(theme: &Theme, status: Status) -> Style {
    let s = theme.colors().surface;
    let outline = theme.colors().outline.color;

    let mut style = styled(
        Color::TRANSPARENT,
        theme.colors().primary.color,
        s.on_surface,
        Color::TRANSPARENT,
        0,
        status,
    );

    style.border = match status {
        Status::Disabled => Border {
            color: theme.colors().outline.variant,
            width: 1.0,
            radius: 10.into(),
        },
        _ => Border {
            color: outline,
            width: 1.2,
            radius: 10.into(),
        },
    };

    style
}

pub fn text(theme: &Theme, status: Status) -> Style {
    let s = theme.colors().surface;

    let base = styled(
        Color::TRANSPARENT,
        s.on_surface,
        s.on_surface_variant,
        Color::TRANSPARENT,
        0,
        status,
    );

    match status {
        Status::Hovered => Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
            ..base
        },
        _ => Style {
            background: None,
            ..base
        },
    }
}

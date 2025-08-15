use iced::widget::container::{Catalog, Style, StyleFn};
use iced::{Background, Border, border};

use super::super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn transparent(_theme: &Theme) -> Style {
    Style {
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn primary(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        background: Some(Background::Color(primary.color)),
        text_color: Some(primary.on_primary),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn primary_container(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        background: Some(Background::Color(primary.primary_container)),
        text_color: Some(primary.on_primary_container),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn secondary(theme: &Theme) -> Style {
    let secondary = theme.colors().secondary;

    Style {
        background: Some(Background::Color(secondary.color)),
        text_color: Some(secondary.on_secondary),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn secondary_container(theme: &Theme) -> Style {
    let secondary = theme.colors().secondary;

    Style {
        background: Some(Background::Color(secondary.secondary_container)),
        text_color: Some(secondary.on_secondary_container),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn tertiary(theme: &Theme) -> Style {
    let tertiary = theme.colors().tertiary;

    Style {
        background: Some(Background::Color(tertiary.color)),
        text_color: Some(tertiary.on_tertiary),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn tertiary_container(theme: &Theme) -> Style {
    let tertiary = theme.colors().tertiary;

    Style {
        background: Some(Background::Color(tertiary.tertiary_container)),
        text_color: Some(tertiary.on_tertiary_container),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn error(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.color)),
        text_color: Some(error.on_error),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn error_container(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.error_container)),
        text_color: Some(error.on_error_container),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn surface(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.color)),
        text_color: Some(surface.on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn surface_container_lowest(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.lowest)),
        text_color: Some(surface.on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn surface_container_low(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.low)),
        text_color: Some(surface.on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn surface_container(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.base)),
        text_color: Some(surface.on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn surface_container_high(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.high)),
        text_color: Some(surface.on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn surface_container_highest(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.highest)),
        text_color: Some(surface.on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn inverse_surface(theme: &Theme) -> Style {
    let inverse = theme.colors().inverse;

    Style {
        background: Some(Background::Color(inverse.inverse_surface)),
        text_color: Some(inverse.inverse_on_surface),
        border: border::rounded(3),
        ..Style::default()
    }
}

pub fn outlined(theme: &Theme) -> Style {
    let base = transparent(theme);

    Style {
        border: Border {
            color: theme.colors().outline.color,
            width: 2.0,
            ..base.border
        },
        ..base
    }
}

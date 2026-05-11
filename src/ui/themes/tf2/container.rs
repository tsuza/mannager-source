use iced::widget::container::{Catalog, Style, StyleFn};
use iced::{Background, Border, Shadow, Vector, border, color, gradient};

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

pub fn default(_theme: &Theme) -> Style {
    transparent(_theme)
}

pub fn transparent(_theme: &Theme) -> Style {
    Style::default()
}

pub fn primary(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        background: Some(Background::Color(primary.color)),
        text_color: Some(primary.on_primary),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        },
        ..Style::default()
    }
}

pub fn primary_container(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        background: Some(Background::Color(primary.primary_container)),
        text_color: Some(primary.on_primary_container),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        },
        ..Style::default()
    }
}

pub fn secondary(theme: &Theme) -> Style {
    let secondary = theme.colors().secondary;

    Style {
        background: Some(Background::Color(secondary.secondary_container)),
        text_color: Some(secondary.on_secondary_container),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..Style::default()
    }
}

pub fn secondary_container(theme: &Theme) -> Style {
    let secondary = theme.colors().secondary;

    Style {
        background: Some(Background::Color(secondary.color)),
        text_color: Some(secondary.on_secondary),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..Style::default()
    }
}

pub fn tertiary(theme: &Theme) -> Style {
    let tertiary = theme.colors().tertiary;

    Style {
        background: Some(Background::Color(tertiary.color)),
        text_color: Some(tertiary.on_tertiary),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..Style::default()
    }
}

pub fn tertiary_container(theme: &Theme) -> Style {
    let tertiary = theme.colors().tertiary;

    Style {
        background: Some(Background::Color(tertiary.tertiary_container)),
        text_color: Some(tertiary.on_tertiary_container),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..Style::default()
    }
}

pub fn error(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.color)),
        text_color: Some(error.on_error),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 10.0),
            blur_radius: 22.0,
        },
        ..Style::default()
    }
}

pub fn error_container(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.error_container)),
        text_color: Some(error.on_error_container),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Style::default()
    }
}

pub fn surface(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.color)),
        text_color: Some(surface.on_surface),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Style::default()
    }
}

pub fn surface_container_lowest(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.lowest)),
        text_color: Some(surface.on_surface),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 10.0,
        },
        ..Style::default()
    }
}

pub fn surface_container_low(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.low)),
        text_color: Some(surface.on_surface),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Style::default()
    }
}

pub fn surface_container(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.base)),
        text_color: Some(surface.on_surface),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 14.0,
        },
        ..Style::default()
    }
}

pub fn surface_container_high(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.high)),
        text_color: Some(surface.on_surface),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Style::default()
    }
}

pub fn surface_container_highest(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.highest)),
        text_color: Some(surface.on_surface),
        border: border::rounded(12),
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 10.0),
            blur_radius: 22.0,
        },
        ..Style::default()
    }
}

pub fn tooltip(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Gradient(iced::Gradient::Linear(
            gradient::Linear::new(0)
                .add_stop(0.0, color!(0x221d1c))
                .add_stop(1.0, color!(0x2a2321)),
        ))),
        text_color: Some(surface.on_surface),
        border: Border {
            color: color!(0x3a3431),
            width: 1.0,
            radius: 12.into(),
        },
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Style::default()
    }
}

pub fn inverse_surface(theme: &Theme) -> Style {
    let inverse = theme.colors().inverse;

    Style {
        background: Some(Background::Color(inverse.inverse_surface)),
        text_color: Some(inverse.inverse_on_surface),
        border: border::rounded(12),
        ..Style::default()
    }
}

pub fn outlined(theme: &Theme) -> Style {
    Style {
        border: Border {
            color: theme.colors().outline.color,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Style::default()
    }
}

// TODO: hardcoded
pub fn info_container(theme: &Theme) -> Style {
    Style {
        background: Some(color!(255, 255, 255, 0.03).into()),
        border: Border {
            radius: 10.into(),
            ..Default::default()
        },
        ..Style::default()
    }
}

pub fn base(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.color)),
        border: Border {
            color: theme.colors().outline.color,
            width: 1.0,
            radius: 10.into(),
        },
        shadow: Shadow {
            color: color!(0, 0, 0, 0.45),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 16.0,
        },
        ..Style::default()
    }
}

pub fn card(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.base)),
        border: Border {
            color: theme.colors().outline.color,
            width: 1.0,
            radius: 10.into(),
        },
        shadow: Shadow {
            color: color!(0, 0, 0, 0.45),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 16.0,
        },
        ..Style::default()
    }
}

pub fn main(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.surface_container.lowest)),
        border: Border {
            color: theme.colors().outline.variant,
            width: 1.0,
            radius: 8.into(),
        },
        ..Style::default()
    }
}

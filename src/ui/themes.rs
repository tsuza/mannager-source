pub mod tf2;

use std::borrow::Cow;

use iced::{
    Color, Shadow, Vector,
    theme::{Base, Style},
};

#[allow(clippy::cast_precision_loss)]
macro_rules! from_argb {
    ($hex:expr) => {{
        let hex = $hex as u32;

        let a = ((hex & 0xff000000) >> 24) as f32 / 255.0;
        let r = (hex & 0x00ff0000) >> 16;
        let g = (hex & 0x0000ff00) >> 8;
        let b = (hex & 0x000000ff);

        ::iced::color!(r as u8, g as u8, b as u8, a)
    }};
}

pub(crate) use from_argb;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    TeamFortress2,
    Custom(Custom),
}

impl Theme {
    pub const ALL: &'static [Self] = &[Self::TeamFortress2];

    pub fn new(name: impl Into<Cow<'static, str>>, colorscheme: ColorScheme) -> Self {
        Self::Custom(Custom {
            name: name.into(),
            is_dark: lightness(colorscheme.surface.color) <= 0.5,
            colorscheme,
        })
    }

    pub const fn new_const(name: &'static str, colorscheme: ColorScheme) -> Self {
        Self::Custom(Custom {
            name: Cow::Borrowed(name),
            is_dark: lightness(colorscheme.surface.color) <= 0.5,
            colorscheme,
        })
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self {
            Self::TeamFortress2 => "Dark".into(),
            Self::Custom(custom) => custom.name.clone(),
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::TeamFortress2 => true,
            Self::Custom(custom) => custom.is_dark,
        }
    }

    pub fn colors(&self) -> ColorScheme {
        match self {
            Self::TeamFortress2 => tf2::color_scheme(),
            Self::Custom(custom) => custom.colorscheme,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::TeamFortress2
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Base for Theme {
    fn base(&self) -> Style {
        Style {
            background_color: self.colors().surface.color,
            text_color: self.colors().surface.on_surface,
        }
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        let colors = self.colors();

        Some(iced::theme::Palette {
            background: colors.surface.color,
            text: colors.surface.on_surface,
            primary: colors.primary.color,
            success: colors.primary.primary_container,
            warning: mix(from_argb!(0xffffff00), colors.primary.color, 0.25),
            danger: colors.error.color,
        })
    }
}

/// A custom [`Theme`].
#[derive(Debug, PartialEq)]
pub struct Custom {
    /// The [`Theme`]'s name.
    pub name: Cow<'static, str>,
    /// Whether the [`Theme`] is dark.
    pub is_dark: bool,
    /// The [`Theme`]'s [`ColorScheme`].
    pub colorscheme: ColorScheme,
}

impl From<Custom> for Theme {
    fn from(custom: Custom) -> Self {
        Self::Custom(custom)
    }
}

impl From<Theme> for Custom {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Custom(custom) => custom,
            theme => Self {
                name: theme.name(),
                is_dark: theme.is_dark(),
                colorscheme: theme.colors(),
            },
        }
    }
}

impl Clone for Custom {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            is_dark: self.is_dark,
            colorscheme: self.colorscheme,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.name.clone_from(&source.name);
        self.is_dark = source.is_dark;
        self.colorscheme = source.colorscheme;
    }
}

/// A [`Theme`]'s color scheme.
///
/// These color roles are base on Material Design 3. For more information about them, visit the
/// official [M3 documentation](https://m3.material.io/styles/color/roles).
///
/// [M3 page]: https://m3.material.io/styles/color/roles
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    /// The primary colors.
    pub primary: Primary,
    /// The secondary colors.
    pub secondary: Secondary,
    /// The tertiary colors.
    pub tertiary: Tertiary,
    /// The error colors.
    pub error: Error,
    /// The surface colors.
    pub surface: Surface,
    /// The inverse colors.
    pub inverse: Inverse,
    /// The outline colors.
    pub outline: Outline,
    /// The shadow color.
    pub shadow: Color,
    /// The scrim color.
    pub scrim: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Primary {
    pub color: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Secondary {
    pub color: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tertiary {
    pub color: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Error {
    pub color: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub color: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub surface_container: SurfaceContainer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceContainer {
    pub lowest: Color,
    pub low: Color,
    pub base: Color,
    pub high: Color,
    pub highest: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inverse {
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub color: Color,
    pub variant: Color,
}

const COLOR_ERROR_MARGIN: f32 = 0.0001;

pub const HOVERED_LAYER_OPACITY: f32 = 0.08;
pub const PRESSED_LAYER_OPACITY: f32 = 0.1;

pub const DISABLED_TEXT_OPACITY: f32 = 0.38;
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;

pub fn elevation(elevation_level: u8) -> f32 {
    (match elevation_level {
        0 => 0.0,
        1 => 1.0,
        2 => 3.0,
        3 => 6.0,
        4 => 8.0,
        _ => 12.0,
    } as f32)
}

pub fn shadow_from_elevation(elevation: f32, color: Color) -> Shadow {
    Shadow {
        color,
        offset: Vector {
            x: 0.0,
            y: elevation,
        },
        blur_radius: (elevation) * (1.0 + 0.4_f32.powf(elevation)),
    }
}

pub fn disabled_text(color: Color) -> Color {
    Color {
        a: DISABLED_TEXT_OPACITY,
        ..color
    }
}

pub fn disabled_container(color: Color) -> Color {
    Color {
        a: DISABLED_CONTAINER_OPACITY,
        ..color
    }
}

pub fn parse_argb(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#').unwrap_or(s);

    let parse_channel = |from: usize, to: usize| {
        let num = usize::from_str_radix(&hex[from..=to], 16).ok()? as f32 / 255.0;

        // If we only got half a byte (one letter), expand it into a full byte (two letters)
        Some(if from == to { num + num * 16.0 } else { num })
    };

    Some(match hex.len() {
        3 => Color::from_rgb(
            parse_channel(0, 0)?,
            parse_channel(1, 1)?,
            parse_channel(2, 2)?,
        ),
        4 => Color::from_rgba(
            parse_channel(1, 1)?,
            parse_channel(2, 2)?,
            parse_channel(3, 3)?,
            parse_channel(0, 0)?,
        ),
        6 => Color::from_rgb(
            parse_channel(0, 1)?,
            parse_channel(2, 3)?,
            parse_channel(4, 5)?,
        ),
        8 => Color::from_rgba(
            parse_channel(2, 3)?,
            parse_channel(4, 5)?,
            parse_channel(6, 7)?,
            parse_channel(0, 1)?,
        ),
        _ => None?,
    })
}

pub fn color_to_argb(color: Color) -> String {
    use std::fmt::Write;

    let mut hex = String::with_capacity(9);

    let [r, g, b, a] = color.into_rgba8();

    let _ = write!(&mut hex, "#");

    if a < u8::MAX {
        let _ = write!(&mut hex, "{a:02X}");
    }

    let _ = write!(&mut hex, "{r:02X}");
    let _ = write!(&mut hex, "{g:02X}");
    let _ = write!(&mut hex, "{b:02X}");

    hex
}

pub const fn lightness(color: Color) -> f32 {
    color.r * 0.299 + color.g * 0.587 + color.b * 0.114
}

pub fn mix(color1: Color, color2: Color, p2: f32) -> Color {
    if p2 <= 0.0 {
        return color1;
    } else if p2 >= 1.0 {
        return color2;
    }

    let p1 = 1.0 - p2;

    if (color1.a - 1.0).abs() > COLOR_ERROR_MARGIN || (color2.a - 1.0).abs() > COLOR_ERROR_MARGIN {
        let a = color1.a * p1 + color2.a * p2;
        if a > 0.0 {
            let c1 = color1.into_linear().map(|c| c * color1.a * p1);
            let c2 = color2.into_linear().map(|c| c * color2.a * p2);

            let [r, g, b] = [c1[0] + c2[0], c1[1] + c2[1], c1[2] + c2[2]].map(|u| u / a);

            return Color::from_linear_rgba(r, g, b, a);
        }
    }

    let c1 = color1.into_linear().map(|c| c * p1);
    let c2 = color2.into_linear().map(|c| c * p2);

    Color::from_linear_rgba(c1[0] + c2[0], c1[1] + c2[1], c1[2] + c2[2], c1[3] + c2[3])
}

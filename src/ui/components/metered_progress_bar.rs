//! Progress bars visualize the progression of an extended computer operation, such as a download, file transfer, or installation.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::metered_progress_bar;
//!
//! struct State {
//!    progress: f32,
//! }
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     metered_progress_bar(0.0..=100.0, state.progress).into()
//! }
//! ```
use iced::Pixels;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::Tree;
use iced::advanced::{Layout, Widget, layout};
use iced::border::{self, Border};
use iced::{self, Background, Color, Element, Length, Rectangle, Size, Theme};

use std::ops::RangeInclusive;

/// A bar that displays progress.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::metered_progress_bar;
///
/// struct State {
///    progress: f32,
/// }
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     metered_progress_bar(0.0..=100.0, state.progress).into()
/// }
/// ```
pub struct MeteredProgressBar<'a, Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<f32>,
    value: f32,
    length: Length,
    girth: Length,
    spacing: f32,
    bars: usize,
    is_vertical: bool,
    class: Theme::Class<'a>,
}

impl<'a, Theme> MeteredProgressBar<'a, Theme>
where
    Theme: Catalog,
{
    /// The default girth of a [`MeteredProgressBar`].
    pub const DEFAULT_GIRTH: f32 = 30.0;

    /// The default spacing of a [`MeteredProgressBar`].
    pub const DEFAULT_SPACING: f32 = 2.0;

    /// The default maximum amount of bars of a [`MeteredProgressBar`].
    pub const DEFAULT_BARS: usize = 10;

    /// Creates a new [`MeteredProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`MeteredProgressBar`]
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        MeteredProgressBar {
            value: value.clamp(*range.start(), *range.end()),
            range,
            length: Length::Fill,
            girth: Length::from(Self::DEFAULT_GIRTH),
            spacing: Self::DEFAULT_SPACING,
            bars: Self::DEFAULT_BARS,
            is_vertical: false,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`MeteredProgressBar`].
    pub fn length(mut self, length: impl Into<Length>) -> Self {
        self.length = length.into();

        self
    }

    /// Sets the height of the [`MeteredProgressBar`].
    pub fn girth(mut self, girth: impl Into<Length>) -> Self {
        self.girth = girth.into();

        self
    }

    /// Sets the spacing _between_ elements.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;

        self
    }

    /// Sets the maximum amounts of bars.
    pub fn bars(mut self, amount: usize) -> Self {
        self.bars = amount;

        self
    }

    /// Turns the [`MeteredProgressBar`] into a vertical [`MeteredProgressBar`].
    ///
    /// By default, a [`MeteredProgressBar`] is horizontal.
    pub fn vertical(mut self) -> Self {
        self.is_vertical = true;
        self
    }

    /// Sets the style of the [`MeteredProgressBar`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`MeteredProgressBar`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn width(&self) -> Length {
        if self.is_vertical {
            self.girth
        } else {
            self.length
        }
    }

    fn height(&self) -> Length {
        if self.is_vertical {
            self.length
        } else {
            self.girth
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for MeteredProgressBar<'_, Theme>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width(), self.height())
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let (range_start, range_end) = self.range.clone().into_inner();

        let length = if self.is_vertical {
            bounds.height
        } else {
            bounds.width
        };

        let filled_percentage = (self.value - range_start) / (range_end - range_start);

        let active_progress_length = if range_start >= range_end {
            0.0
        } else {
            length * filled_percentage
        };

        let style = theme.style(&self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { ..bounds },
                border: style.border,
                snap: false,
                ..renderer::Quad::default()
            },
            style.background,
        );

        if active_progress_length > 0.0 {
            let bars_amount = (self.bars as f32 * filled_percentage).floor() as usize;
            let bar_length = {
                let total_spacing = self.spacing * self.bars.saturating_sub(1) as f32;
                let side_padding = self.spacing * 2.0;

                (length - total_spacing - side_padding) / self.bars as f32
            };

            for bar in 0..bars_amount {
                let bar_bounds = if self.is_vertical {
                    Rectangle {
                        y: bounds.y + bounds.height - active_progress_length,
                        height: active_progress_length,
                        ..bounds
                    }
                } else {
                    Rectangle {
                        x: bounds.x + self.spacing + bar as f32 * (bar_length + self.spacing),
                        y: bounds.y + self.spacing,
                        width: bar_length,
                        height: bounds.height - (self.spacing * 2.0),
                    }
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: bar_bounds,
                        border: Border {
                            color: Color::TRANSPARENT,
                            ..style.border
                        },
                        snap: false,
                        ..renderer::Quad::default()
                    },
                    style.bar,
                );
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<MeteredProgressBar<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(
        metered_progress_bar: MeteredProgressBar<'a, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(metered_progress_bar)
    }
}

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the progress bar.
    pub background: Background,
    /// The [`Background`] of the bar of the progress bar.
    pub bar: Background,
    /// The [`Border`] of the progress bar.
    pub border: Border,
}

/// The theme catalog of a [`MeteredProgressBar`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`MeteredProgressBar`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The primary style of a [`MeteredProgressBar`].
pub fn primary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.primary.base.color)
}

/// The secondary style of a [`MeteredProgressBar`].
pub fn secondary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.secondary.base.color,
    )
}

/// The success style of a [`MeteredProgressBar`].
pub fn success(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.success.base.color)
}

/// The warning style of a [`MeteredProgressBar`].
pub fn warning(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.warning.base.color)
}

/// The danger style of a [`MeteredProgressBar`].
pub fn danger(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.danger.base.color)
}

fn styled(background: impl Into<Background>, bar: impl Into<Background>) -> Style {
    Style {
        background: background.into(),
        bar: bar.into(),
        border: border::rounded(2),
    }
}

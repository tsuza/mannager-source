use iced::advanced::widget::tree;
use iced::advanced::{self, Layout, Shell, Widget, layout, mouse, renderer, widget};
use iced::{self, Border, Color, Element, Event, Length, Rectangle, Size};

use std::convert::identity;
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

use iced::widget::progress_bar::{Catalog, Style, StyleFn};

pub fn animated_progress_bar<'a, Theme>(
    range: RangeInclusive<f32>,
    value: f32,
) -> AnimatedProgressBar<'a, Theme>
where
    Theme: Catalog,
{
    AnimatedProgressBar::new(range, value)
}

pub struct AnimatedProgressBar<'a, Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<f32>,
    value: f32,
    length: Length,
    girth: Length,
    is_vertical: bool,
    class: Theme::Class<'a>,
}

impl<'a, Theme> AnimatedProgressBar<'a, Theme>
where
    Theme: Catalog,
{
    /// The default girth of a [`ProgressBar`].
    pub const DEFAULT_GIRTH: f32 = 30.0;

    /// Creates a new [`ProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`ProgressBar`]
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        AnimatedProgressBar {
            value: value.clamp(*range.start(), *range.end()),
            range,
            length: Length::Fill,
            girth: Length::from(Self::DEFAULT_GIRTH),
            is_vertical: false,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`ProgressBar`].
    pub fn length(mut self, length: impl Into<Length>) -> Self {
        self.length = length.into();
        self
    }

    /// Sets the height of the [`ProgressBar`].
    pub fn girth(mut self, girth: impl Into<Length>) -> Self {
        self.girth = girth.into();
        self
    }

    /// Turns the [`ProgressBar`] into a vertical [`ProgressBar`].
    ///
    /// By default, a [`ProgressBar`] is horizontal.
    pub fn vertical(mut self) -> Self {
        self.is_vertical = true;
        self
    }

    /// Sets the style of the [`ProgressBar`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`ProgressBar`].
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for AnimatedProgressBar<'_, Theme>
where
    Theme: Catalog,
    Renderer: advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width(), self.height())
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn update(
        &mut self,
        state: &mut tree::Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let Event::Window(iced::window::Event::RedrawRequested(instant)) = event {
            let state: &mut State = state.state.downcast_mut();
            state.now = *instant;
            state.animation.go_mut(self.value, state.now);
        }
    }

    fn draw(
        &self,
        _state: &widget::Tree,
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

        let active_progress_length = if range_start >= range_end {
            0.0
        } else {
            let state: &State = _state.state.downcast_ref();
            let value = state.animation.interpolate_with(identity, state.now);

            length * (value - range_start) / (range_end - range_start)
        };

        let style = theme.style(&self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { ..bounds },
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        if active_progress_length > 0.0 {
            let bounds = if self.is_vertical {
                Rectangle {
                    y: bounds.y + bounds.height - active_progress_length,
                    height: active_progress_length,
                    ..bounds
                }
            } else {
                Rectangle {
                    width: active_progress_length,
                    ..bounds
                }
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: Color::TRANSPARENT,
                        ..style.border
                    },
                    ..renderer::Quad::default()
                },
                style.bar,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<AnimatedProgressBar<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + advanced::Renderer,
{
    fn from(progress_bar: AnimatedProgressBar<'a, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(progress_bar)
    }
}

struct State {
    now: Instant,
    animation: iced::Animation<f32>,
}

impl State {
    fn new() -> Self {
        Self {
            now: Instant::now(),
            animation: iced::Animation::new(Default::default())
                .easing(iced::animation::Easing::Linear)
                .duration(Duration::from_millis(175)),
        }
    }
}

use iced::{
    border, color, padding,
    widget::{self, scrollable, text_input},
    Background, Color, Theme,
};
use iced_aw::menu;

pub struct Style;

impl Style {
    pub fn scrollable(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
        widget::scrollable::Style {
            vertical_rail: scrollable::Rail {
                background: Some(iced::Background::Color(color!(0x686252))),
                border: border::rounded(0),
                scroller: scrollable::Scroller {
                    color: color!(0xada28d),
                    border: border::rounded(0),
                },
            },
            gap: None,
            ..scrollable::default(_theme, _status)
        }
    }

    pub fn text_input(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
        text_input::Style {
            background: iced::Background::Color(color!(0x2a2421)),
            border: border::width(0),
            value: color!(0xffffff),
            ..text_input::default(_theme, _status)
        }
    }

    pub fn menu(_theme: &Theme, _status: iced_aw::style::Status) -> menu::Style {
        menu::Style {
            bar_background: Background::Color(Color::TRANSPARENT),
            bar_border: border::width(0),
            menu_background_expand: padding::all(5),
            menu_background: Background::Color(color!(0x2A2725)),
            menu_border: border::width(3).rounded(3).color(color!(0x6b6664)),
            ..Default::default()
        }
    }
}

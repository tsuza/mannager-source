use iced::{
    border, color, padding,
    widget::{self, button, scrollable, text_input},
    Background, Color, Theme,
};
use iced_aw::{menu, style::colors};

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
        match _status {
            text_input::Status::Active
            | text_input::Status::Hovered
            | text_input::Status::Disabled => text_input::Style {
                background: iced::Background::Color(color!(0xFBECCB)),
                value: color!(0x524a42),
                border: border::rounded(7).width(2).color(color!(0x645e51)),
                ..widget::text_input::default(_theme, _status)
            },
            text_input::Status::Focused => text_input::Style {
                background: iced::Background::Color(color!(0xFBECCB)),
                value: color!(0x524a42),
                border: border::rounded(0).width(2).color(colors::SKY_BLUE),
                ..widget::text_input::default(_theme, _status)
            },
        }
    }

    pub fn server_text_input(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
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
            bar_border: border::rounded(0),
            menu_background_expand: padding::all(5),
            menu_background: Background::Color(color!(0x2A2725)),
            menu_border: border::width(3).rounded(3).color(color!(0x6b6664)),
            ..Default::default()
        }
    }

    pub fn menu_button(_theme: &Theme, _status: button::Status) -> button::Style {
        match _status {
            button::Status::Active => button::Style {
                background: None,
                text_color: color!(0xeee5cf),
                border: border::rounded(3),
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: None,
                text_color: color!(128, 128, 128, 0.9),
                border: border::rounded(3),
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(color!(0x994f3f))),
                text_color: color!(0xffffff),
                border: border::rounded(3),
                ..Default::default()
            },
        }
    }

    pub fn button(_theme: &Theme, _status: button::Status) -> button::Style {
        match _status {
            button::Status::Active | button::Status::Disabled => button::Style {
                background: Some(Background::Color(color!(0x7e7366))),
                text_color: color!(0xeee5cf),
                border: border::rounded(3),
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(color!(0x994f3f))),
                text_color: color!(0xffffff),
                border: border::rounded(3),
                ..Default::default()
            },
        }
    }

    pub fn play_button(_theme: &Theme, _status: button::Status) -> button::Style {
        match _status {
            button::Status::Active | button::Status::Disabled => button::Style {
                background: Some(Background::Color(color!(0x537321))),
                text_color: color!(0xeee5cf),
                border: border::rounded(3),
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(color!(0x669e33))),
                text_color: color!(0xeee5cf),
                border: border::rounded(3),
                ..Default::default()
            },
        }
    }

    pub fn form_button(_theme: &Theme, _status: button::Status) -> button::Style {
        match _status {
            button::Status::Active | button::Status::Disabled => button::Style {
                background: Some(Background::Color(color!(0x7e7366))),
                text_color: Color::WHITE,
                border: border::rounded(0),
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(color!(0x994f3f))),
                text_color: Color::WHITE,
                border: border::rounded(0),
                ..Default::default()
            },
        }
    }
}

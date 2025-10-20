use iced::{Length, widget::center};

use crate::ui::Element;

pub fn loading<'a, Message: 'a>() -> Element<'a, Message> {
    center("Loading...")
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

use iced::widget::horizontal_space;

use crate::ui::Element;

pub fn loading<'a, Message: 'a>() -> Element<'a, Message> {
    horizontal_space().into()
}

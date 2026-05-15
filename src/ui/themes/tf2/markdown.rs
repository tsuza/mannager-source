use iced::widget::{container, markdown};

use crate::ui::themes::{Theme, tf2};

impl markdown::Catalog for Theme {
    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(tf2::container::card)
    }
}

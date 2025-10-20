use std::sync::LazyLock;

use iced::widget::svg;

use crate::core::{Game, SourceEngineVersion};

pub struct SourceGame {
    pub game: Game,
    pub image: svg::Handle,
    pub engine: SourceEngineVersion,
}

pub static SOURCE_GAMES: LazyLock<Vec<SourceGame>> = LazyLock::new(|| {
    vec![
        SourceGame {
            game: Game::TeamFortress2,
            image: svg::Handle::from_memory(include_bytes!("../../images/tf2-logo.svg")),
            engine: SourceEngineVersion::Source1,
        },
        SourceGame {
            game: Game::CounterStrikeSource,
            image: svg::Handle::from_memory(include_bytes!("../../images/css-logo.svg")),
            engine: SourceEngineVersion::Source1,
        },
        SourceGame {
            game: Game::LeftForDead1,
            image: svg::Handle::from_memory(include_bytes!("../../images/l4d1-logo.svg")),
            engine: SourceEngineVersion::Source1,
        },
        SourceGame {
            game: Game::LeftForDead2,
            image: svg::Handle::from_memory(include_bytes!("../../images/l4d2-logo.svg")),
            engine: SourceEngineVersion::Source1,
        },
        SourceGame {
            game: Game::NoMoreRoomInHell,
            image: svg::Handle::from_memory(include_bytes!("../../images/nmrih-logo.svg")),
            engine: SourceEngineVersion::Source1,
        },
        SourceGame {
            game: Game::HalfLife2DM,
            image: svg::Handle::from_memory(include_bytes!("../../images/hl2mp-logo.svg")),
            engine: SourceEngineVersion::Source1,
        },
        SourceGame {
            game: Game::CounterStrike2,
            image: svg::Handle::from_memory(include_bytes!("../../images/cs2-logo.svg")),
            engine: SourceEngineVersion::Source2,
        },
        SourceGame {
            game: Game::Deadlock,
            image: svg::Handle::from_memory(include_bytes!("../../images/deadlock-logo.svg")),
            engine: SourceEngineVersion::Source2,
        },
    ]
});

use ui::State;

pub mod core;
pub mod ui;

fn main() -> iced::Result {
    iced::application(State::title, State::update, State::view)
        .subscription(State::subscription)
        .run_with(State::new)
}

/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    //iced::application(State::title, State::update, State::view).run_with(State::new)
    let mut depot = steamcmd::DepotDownloader::new("./depotdownloader").await?;

    let stdout = depot
        .create_game_server("/home/suza/Coding/Rust/mannager-source/servers/tf2", 232250)
        .await?;

    if let Some(stdout) = stdout {
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await? {
            println!("Output: {}", line);
        }
    }

    println!("Done!");

    Ok(())
}
*/

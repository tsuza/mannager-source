pub mod loading;
pub mod serverboot;
pub mod servercreation;
pub mod serverlist;

pub enum Screen {
    Loading,
    ServerList,
    ServerCreation(servercreation::State),
    ServerTerminal(usize),
}

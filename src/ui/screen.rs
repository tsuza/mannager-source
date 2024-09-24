pub mod serverboot;
pub mod servercreation;
pub mod serverlist;

pub struct Screen {
    pub current_page: ScreenKind,

    pub serverlist_page: serverlist::State,
}

pub enum ScreenKind {
    ServerList,
}

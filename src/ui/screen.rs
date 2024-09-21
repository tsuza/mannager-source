use serverlist::ServerList;

pub mod servercreation;
pub mod serverlist;

pub struct Screen {
    pub current_page: ScreenKind,

    pub serverlist_page: ServerList,
}

pub enum ScreenKind {
    ServerList,
}

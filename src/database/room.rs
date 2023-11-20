use crate::net::nation::Nation;
use crate::net::session_info::SessionInfo;
use crate::net::session_type::SessionType;

pub struct Room {
    pub id: u32,
    pub name: String,
    pub session: SessionInfo,
}

impl Room {
    pub fn new(id: u32, name: &str, nation: Nation) -> Self {
        Self {
            id,
            name: name.to_string(),
            session: SessionInfo::new(nation, SessionType::Room),
        }
    }
}
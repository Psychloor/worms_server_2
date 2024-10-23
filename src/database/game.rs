use crate::database::Database;
use crate::net::nation::Nation;
use crate::net::session_access::SessionAccess;
use crate::net::session_info::SessionInfo;
use crate::net::session_type::SessionType;
use std::net::IpAddr;
use std::sync::Arc;

pub struct Game {
    pub id: u32,
    pub name: String,
    pub room_id: u32,
    pub ip: IpAddr,
    pub session: Arc<SessionInfo>,
}

impl Game {
    pub fn new(
        id: u32,
        name: &str,
        nation: Nation,
        room_id: u32,
        address: IpAddr,
        access: SessionAccess,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            room_id,
            ip: address,
            session: SessionInfo::new_with_access(nation, SessionType::Game, access),
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        Database::recycle_id(self.id);
    }
}

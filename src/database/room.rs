use crate::database::Database;
use crate::net::nation::Nation;
use crate::net::session_info::SessionInfo;
use crate::net::session_type::SessionType;
use std::sync::Arc;

pub struct Room {
    pub id: u32,
    pub name: String,
    pub session: Arc<SessionInfo>,
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

impl Drop for Room {
    fn drop(&mut self) {
        Database::recycle_id(self.id);
    }
}
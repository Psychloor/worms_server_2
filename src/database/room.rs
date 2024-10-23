use crate::database::Database;
use crate::net::nation::Nation;
use crate::net::session_info::SessionInfo;
use crate::net::session_type::SessionType;
use std::sync::{Arc, Weak};

pub struct Room {
    pub id: u32,
    pub name: String,
    pub session: Arc<SessionInfo>,
    db: Weak<Database>,
}

impl Room {
    pub fn new(id: u32, name: &str, nation: Nation, db: Weak<Database>) -> Self {
        Self {
            id,
            name: name.to_string(),
            session: SessionInfo::new(nation, SessionType::Room),
            db,
        }
    }
}

impl Drop for Room {
    fn drop(&mut self) {
        Database::recycle_id(self.db.clone(), self.id);
    }
}

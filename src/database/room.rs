use crate::database::Database;
use crate::net::nation::Nation;
use crate::net::session_info::SessionInfo;
use crate::net::session_type::SessionType;
use std::sync::{Arc, Weak};
use tokio::task;

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
        let id = self.id;
        let db = self.db.clone();

        task::spawn(async move {
            if let Some(db) = db.upgrade() {
                Database::recycle_id(db, id).await;
            }
        });
    }
}

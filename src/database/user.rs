use crate::database::Database;
use crate::net::nation::Nation;
use crate::net::session_info::SessionInfo;
use crate::net::session_type::SessionType;
use std::sync::Arc;
use tokio::sync::mpsc::WeakSender;
use tokio_util::bytes::Bytes;

pub struct User {
    pub sender: WeakSender<Arc<Bytes>>,
    pub id: u32,
    pub name: String,
    pub session: Arc<SessionInfo>,
    pub room_id: u32,
}

impl User {
    pub fn new(sender: WeakSender<Arc<Bytes>>, id: u32, name: &str, nation: Nation) -> Self {
        Self {
            sender,
            id,
            name: name.to_string(),
            session: SessionInfo::new(nation, SessionType::User),
            room_id: 0,
        }
    }

    pub async fn send_packet(&self, packet: Arc<Bytes>) -> anyhow::Result<()> {
        if let Some(sender) = self.sender.upgrade() {
            sender.send(packet).await?
        }

        // if it failed, the user connection doesn't exist anymore
        Ok(())
    }
}

impl Drop for User {
    fn drop(&mut self) {
        Database::recycle_id(self.id);
    }
}
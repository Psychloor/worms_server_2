use crate::database::Database;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use anyhow::bail;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct ListUsersHandler;

#[async_trait]
impl PacketHandler for ListUsersHandler {
    async fn handle_packet(
        db: &Arc<Database>,
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        let user_room_id = db.users.get(&client_id).map_or(0, |user| user.room_id);

        if user_room_id < Database::ID_START
            || packet.value_2 != Some(user_room_id)
            || packet.value_4 != Some(0)
        {
            bail!("Invalid Data!");
        }

        for user in db.users.iter().filter(|p| p.room_id == user_room_id) {
            let packet = WormsPacket::new(PacketCode::ListItem)
                .value_1(*user.key())
                .name(&user.name)
                .session(user.session.clone())
                .build()?;

            tx.send(packet).await?;
        }

        let packet = WormsPacket::new(PacketCode::ListEnd).build()?;
        tx.send(packet).await?;

        Ok(())
    }
}
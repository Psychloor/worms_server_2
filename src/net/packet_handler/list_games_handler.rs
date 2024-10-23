use crate::database::{Database, DATABASE};
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use anyhow::bail;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct ListGamesHandler;

#[async_trait]
impl PacketHandler for ListGamesHandler {
    async fn handle_packet(
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        let db = &DATABASE;
        let user_room_id = db.users.get(&client_id).map_or(0, |user| user.room_id);

        if user_room_id < Database::ID_START
            || packet.value_2 != Some(user_room_id)
            || packet.value_4 != Some(0)
        {
            bail!("Invalid Data!");
        }

        for game in db.games.iter().filter(|g| g.room_id == user_room_id) {
            let packet = WormsPacket::create(PacketCode::ListItem)
                .with_value_1(*game.key())
                .with_data(&game.ip.to_string())
                .with_name(&game.name)
                .with_session(&game.session)
                .build()?;
            tx.send(packet).await?;
        }

        let packet = WormsPacket::create(PacketCode::ListEnd).build()?;
        tx.send(packet).await?;

        Ok(())
    }
}

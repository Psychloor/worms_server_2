use crate::database::DATABASE;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use anyhow::anyhow;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct ConnectGameHandler;

#[async_trait]
impl PacketHandler for ConnectGameHandler {
    async fn handle_packet(
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        let game_id = packet.value_0.ok_or(anyhow!("no game id included!"))?;

        if let Some(game) = DATABASE.games.get(&game_id) {
            let user_room_id = { DATABASE.users.get(&client_id).map(|u| u.room_id) };

            if Some(game.room_id) == user_room_id {
                let packet = WormsPacket::create(PacketCode::ConnectGameReply)
                    .with_data(&game.ip.to_string())
                    .with_error_code(0)
                    .build()?;
                tx.send(packet).await?;
                return Ok(());
            }
        }

        let error_packet = WormsPacket::create(PacketCode::ConnectGameReply)
            .with_data("")
            .with_error_code(1)
            .build()?;
        tx.send(error_packet).await?;

        Ok(())
    }
}

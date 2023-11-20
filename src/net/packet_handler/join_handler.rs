use crate::database::Database;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use crate::server::Server;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct JoinHandler;

#[async_trait]
impl PacketHandler for JoinHandler {
    async fn handle_packet(
        db: &Arc<Database>,
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        let join_id = packet.value_2.as_ref().map_or(0, |p| *p);
        let user_room_id_original = db.users.get(&client_id).map_or(0, |u| u.room_id);

        if join_id == 0 || packet.value_10 != Some(client_id) {
            bail!("Invalid Data!");
        }

        // Check rooms
        if db.rooms.get(&join_id).is_some() {
            db.users
                .get_mut(&client_id)
                .ok_or(anyhow!("User not found!"))?
                .room_id = join_id;

            let packet = WormsPacket::new(PacketCode::Join)
                .value_2(join_id)
                .value_10(client_id)
                .build()?;
            Server::broadcast_all_except(Arc::clone(db), packet, &client_id).await?;

            let packet = WormsPacket::new(PacketCode::JoinReply)
                .error_code(0)
                .build()?;
            tx.send(packet).await?;

            return Ok(());
        } else if let Some(game) = db.games.get(&join_id) {
            if game.room_id == user_room_id_original {
                let packet = WormsPacket::new(PacketCode::Join)
                    .value_2(join_id)
                    .value_10(client_id)
                    .build()?;
                Server::broadcast_all_except(Arc::clone(db), packet, &client_id).await?;

                let packet = WormsPacket::new(PacketCode::JoinReply)
                    .error_code(0)
                    .build()?;
                tx.send(packet).await?;

                return Ok(());
            }
        }

        // if we got to here then there was no room or game to join
        let packet = WormsPacket::new(PacketCode::JoinReply)
            .error_code(1)
            .build()?;
        tx.send(packet).await?;

        Ok(())
    }
}
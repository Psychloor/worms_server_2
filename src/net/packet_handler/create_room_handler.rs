use crate::database::room::Room;
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

pub struct CreateRoomHandler;

#[async_trait]
impl PacketHandler for CreateRoomHandler {
    async fn handle_packet(
        db: &Arc<Database>,
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        if packet.value_1 != Some(0)
            || packet.value_4 != Some(0)
            || packet.data.is_none()
            || packet.name.is_none()
            || packet.session.is_none()
        {
            bail!("Invalid Data!");
        }

        let room_name = packet
            .name
            .as_ref()
            .ok_or(anyhow!("no room name included in create room handler!"))?;

        if db
            .rooms
            .iter()
            .any(|r| r.name.eq_ignore_ascii_case(room_name))
        {
            let packet = WormsPacket::new(PacketCode::CreateRoomReply)
                .value_1(0)
                .error_code(1)
                .build()?;
            tx.send(packet).await?;
        } else {
            let new_id = Database::get_next_id(db).await;
            let new_room = Room::new(new_id, room_name, packet.session.as_ref().unwrap().nation);

            // Notify all users of this newly made room, made early since the room will be consumed
            let packet = WormsPacket::new(PacketCode::CreateRoom)
                .value_1(new_id)
                .value_4(0)
                .data("")
                .name(room_name)
                .session(new_room.session.clone())
                .build()?;
            db.rooms.insert(new_id, new_room);

            Server::broadcast_all_except(Arc::clone(db), packet, &client_id).await?;

            // Success packet to sender
            let packet = WormsPacket::new(PacketCode::CreateRoomReply)
                .value_1(new_id)
                .error_code(0)
                .build()?;
            tx.send(packet).await?;
        }

        Ok(())
    }
}
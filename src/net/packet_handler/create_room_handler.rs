use crate::database::room::Room;
use crate::database::{Database, DATABASE};
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use crate::server::Server;
use anyhow::{anyhow, bail};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct CreateRoomHandler;

impl PacketHandler for CreateRoomHandler {
    async fn handle_packet(
        tx: Sender<Arc<Bytes>>,
        packet: Arc<WormsPacket>,
        client_id: u32,
        _address: SocketAddr,
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

        if DATABASE
            .rooms
            .iter()
            .any(|r| r.name.eq_ignore_ascii_case(room_name))
        {
            let packet = WormsPacket::create(PacketCode::CreateRoomReply)
                .with_value_1(0)
                .with_error_code(1)
                .build()?;
            tx.send(packet).await?;
        } else {
            let new_id = Database::get_next_id().await;
            let new_room = Room::new(new_id, room_name, packet.session.as_ref().unwrap().nation);

            // Notify all users of this newly made room, made early since the room will be consumed
            let packet = WormsPacket::create(PacketCode::CreateRoom)
                .with_value_1(new_id)
                .with_value_4(0)
                .with_data("")
                .with_name(room_name)
                .with_session(&new_room.session)
                .build()?;
            DATABASE.rooms.insert(new_id, new_room);

            Server::broadcast_all_except(packet, &client_id).await?;

            // Success packet to sender
            let packet = WormsPacket::create(PacketCode::CreateRoomReply)
                .with_value_1(new_id)
                .with_error_code(0)
                .build()?;
            tx.send(packet).await?;
        }

        Ok(())
    }
}

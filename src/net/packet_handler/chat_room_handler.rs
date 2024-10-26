use crate::database::DATABASE;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use anyhow::{anyhow, bail};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct ChatRoomHandler;

impl PacketHandler for ChatRoomHandler {
    async fn handle_packet(
        tx: Sender<Arc<Bytes>>,
        packet: Arc<WormsPacket>,
        client_id: u32,
        _address: SocketAddr,
    ) -> anyhow::Result<()> {
        if packet.value_0 != Some(client_id) {
            bail!("From user invalid!");
        }

        let message = packet
            .data
            .as_ref()
            .ok_or(anyhow!("No message included in chat packet!"))?;
        let target_id = packet
            .value_3
            .ok_or(anyhow!("No target id included in chat packet!"))?;

        let client_user = DATABASE
            .users
            .get(&client_id)
            .ok_or(anyhow!("User '{}' not found!", client_id))?;

        // Regular chat
        let prefix = format!("GRP:[ {} ]  ", &client_user.name);
        if message.starts_with(&prefix) {
            // Check if user can access the room.
            if client_user.room_id == target_id {
                let packet = WormsPacket::create(PacketCode::ChatRoom)
                    .with_value_0(client_id)
                    .with_value_3(client_user.room_id)
                    .with_data(message)
                    .build()?;

                for user in DATABASE
                    .users
                    .iter()
                    .filter(|u| u.id != client_id && u.room_id == client_user.room_id)
                {
                    user.send_packet(Arc::clone(&packet)).await?;
                }

                let packet = WormsPacket::create(PacketCode::ChatRoomReply)
                    .with_error_code(0)
                    .build()?;

                tx.send(packet).await?;

                return Ok(());
            }
        }

        // Private chat
        let prefix = format!("PRV:[ {} ]  ", &client_user.name);
        if message.starts_with(&prefix) {
            // Check if user can access the user.
            if let Some(target_user) = DATABASE.users.get(&target_id) {
                if target_user.room_id == client_user.room_id {
                    let packet = WormsPacket::create(PacketCode::ChatRoom)
                        .with_value_0(client_id)
                        .with_value_3(target_user.id)
                        .with_data(message)
                        .build()?;

                    target_user.send_packet(packet).await?;

                    let packet = WormsPacket::create(PacketCode::ChatRoomReply)
                        .with_error_code(0)
                        .build()?;
                    tx.send(packet).await?;
                    return Ok(());
                }
            }
        }

        // Failed to send
        let packet = WormsPacket::create(PacketCode::ChatRoomReply)
            .with_error_code(1)
            .build()?;
        tx.send(packet).await?;

        Ok(())
    }
}

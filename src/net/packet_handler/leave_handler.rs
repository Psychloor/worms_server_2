use crate::database::DATABASE;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use crate::server::Server;
use anyhow::bail;
use async_trait::async_trait;
use log::error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct LeaveHandler;

#[async_trait]
impl PacketHandler for LeaveHandler {
    async fn handle_packet(
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        if packet.value_2.is_none() || packet.value_10 != Some(client_id) {
            bail!("Invalid Data!");
        }
        let client_room_id = { DATABASE.users.get(&client_id).map_or(0, |u| u.room_id) };

        if packet.value_2 == Some(client_room_id) {
            let leave_result = Server::leave_room(client_room_id, client_id).await;
            {
                if leave_result.is_err() {
                    error!("Error leaving room: {:?}", leave_result.err().unwrap())
                }
                if let Some(mut user) = DATABASE.users.get_mut(&client_id) {
                    user.room_id = 0;
                }
            }

            let packet = WormsPacket::create(PacketCode::LeaveReply)
                .with_error_code(0)
                .build()?;
            tx.send(packet).await?;
        } else {
            let packet = WormsPacket::create(PacketCode::LeaveReply)
                .with_error_code(1)
                .build()?;
            tx.send(packet).await?;
        }

        Ok(())
    }
}

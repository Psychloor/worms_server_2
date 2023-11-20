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

pub struct ListRoomsHandler;

#[async_trait]
impl PacketHandler for ListRoomsHandler {
    async fn handle_packet(
        db: &Arc<Database>,
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        _client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()> {
        if packet.value_4 != Some(0) {
            bail!("Invalid Data!");
        }

        for room in db.rooms.iter() {
            let packet = WormsPacket::new(PacketCode::ListItem)
                .value_1(*room.key())
                .data("")
                .name(room.name.as_str())
                .session(room.session.clone())
                .build()?;
            tx.send(packet).await?;
        }

        let packet = WormsPacket::new(PacketCode::ListEnd).build()?;
        tx.send(packet).await?;

        Ok(())
    }
}
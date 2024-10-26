use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use eyre::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct CloseHandler;

impl PacketHandler for CloseHandler {
    async fn handle_packet(
        tx: Sender<Arc<Bytes>>,
        packet: Arc<WormsPacket>,
        _client_id: u32,
        _address: SocketAddr,
    ) -> Result<()> {
        if packet.value_10.is_some() {
            let packet = WormsPacket::create(PacketCode::CloseReply)
                .with_error_code(0)
                .build()?;
            tx.send(packet).await?;
        }

        Ok(())
    }
}

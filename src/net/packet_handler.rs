pub(crate) mod chat_room_handler;
pub(crate) mod close_handler;
pub(crate) mod connect_game_handler;
pub(crate) mod create_game_handler;
pub(crate) mod create_room_handler;
pub(crate) mod join_handler;
pub(crate) mod leave_handler;
pub(crate) mod list_games_handler;
pub(crate) mod list_rooms_handler;
pub(crate) mod list_users_handler;

use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::chat_room_handler::ChatRoomHandler;
use crate::net::packet_handler::close_handler::CloseHandler;
use crate::net::packet_handler::connect_game_handler::ConnectGameHandler;
use crate::net::packet_handler::create_game_handler::CreateGameHandler;
use crate::net::packet_handler::create_room_handler::CreateRoomHandler;
use crate::net::packet_handler::join_handler::JoinHandler;
use crate::net::packet_handler::leave_handler::LeaveHandler;
use crate::net::packet_handler::list_games_handler::ListGamesHandler;
use crate::net::packet_handler::list_rooms_handler::ListRoomsHandler;
use crate::net::packet_handler::list_users_handler::ListUsersHandler;
use crate::net::worms_packet::WormsPacket;
use anyhow::anyhow;
use async_trait::async_trait;
use log::{debug, error};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

#[async_trait]
pub trait PacketHandler {
    async fn handle_packet(
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        _client_id: u32,
        _address: &SocketAddr,
    ) -> anyhow::Result<()>;
}

pub async fn dispatch(
    code: PacketCode,
    tx: &Sender<Arc<Bytes>>,
    packet: &Arc<WormsPacket>,
    client_id: u32,
    address: &SocketAddr,
) -> anyhow::Result<()> {
    debug!("Dispatching handler for: {:?}", &code);
    match code {
        PacketCode::ListRooms => {
            ListRoomsHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::CreateRoom => {
            CreateRoomHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::ListUsers => {
            ListUsersHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::ListGames => {
            ListGamesHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::Join => JoinHandler::handle_packet(tx, packet, client_id, address).await,

        PacketCode::CreateGame => {
            CreateGameHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::ChatRoom => {
            ChatRoomHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::ConnectGame => {
            ConnectGameHandler::handle_packet(tx, packet, client_id, address).await
        }

        PacketCode::Close => CloseHandler::handle_packet(tx, packet, client_id, address).await,

        PacketCode::Leave => LeaveHandler::handle_packet(tx, packet, client_id, address).await,

        _ => Err(anyhow!("Unknown packet dispatched! {:?}", code)),
    }
    .map_err(|e| {
        error!("Error Dispatching Packet: {}", e);
        e
    })
}
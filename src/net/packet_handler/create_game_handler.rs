use crate::database::game::Game;
use crate::database::Database;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler::PacketHandler;
use crate::net::worms_packet::WormsPacket;
use crate::server::Server;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::bytes::Bytes;

pub struct CreateGameHandler;

const INVALID_MESSAGE: &'static str = "GRP:Cannot host your game. Please use FrontendKitWS with fkNetcode. More information at worms2d.info/fkNetcode";

#[async_trait]
impl PacketHandler for CreateGameHandler {
    async fn handle_packet(
        db: &Arc<Database>,
        tx: &Sender<Arc<Bytes>>,
        packet: &Arc<WormsPacket>,
        client_id: u32,
        address: &SocketAddr,
    ) -> anyhow::Result<()> {
        let client_user = db
            .users
            .get(&client_id)
            .ok_or(anyhow!("client user not found!"))?;

        if packet.value_1 != Some(0)
            || packet.value_2 != Some(client_user.room_id)
            || packet.value_4 != Some(0x800)
            || packet.data.is_none()
            || packet.name.is_none()
            || packet.session.is_none()
        {
            bail!("Invalid Data!");
        }

        let ip_result = packet
            .data
            .as_ref()
            .ok_or(anyhow!(
                "No ip received in create game handler!, got {:?}",
                packet.data
            ))?
            .parse::<IpAddr>();

        if let Ok(ip) = ip_result {
            if address.ip().to_string() == "127.0.0.1" || ip == address.ip() {
                let new_id = Database::get_next_id(db).await;

                let game = Game::new(
                    new_id,
                    &client_user.name,
                    client_user.session.nation,
                    client_user.room_id,
                    address.ip(),
                    packet.session.as_ref().unwrap().access,
                );

                {
                    db.user_to_game.insert(client_user.name.clone(), new_id);
                }

                let packet = WormsPacket::create(PacketCode::CreateGame)
                    .with_value_1(new_id)
                    .with_value_2(game.room_id)
                    .with_value_4(0x800)
                    .with_data(&address.ip().to_string())
                    .with_name(&game.name)
                    .with_session(game.session.clone())
                    .build()?;

                db.games.insert(new_id, game);
                Server::broadcast_all_except(Arc::clone(db), packet, &client_id).await?;

                let packet = WormsPacket::create(PacketCode::CreateGameReply)
                    .with_value_1(new_id)
                    .with_error_code(0)
                    .build()?;
                tx.send(packet).await?;

                return Ok(());
            }
        }

        let packet = WormsPacket::create(PacketCode::CreateGameReply)
            .with_value_1(0)
            .with_error_code(2)
            .build()?;
        tx.send(packet).await?;

        let packet = WormsPacket::create(PacketCode::ChatRoom)
            .with_value_1(client_user.id)
            .with_value_3(client_user.room_id)
            .with_data(INVALID_MESSAGE)
            .build()?;
        tx.send(packet).await?;

        Ok(())
    }
}
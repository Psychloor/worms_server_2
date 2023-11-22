use crate::database::user::User;
use crate::database::Database;
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler;
use crate::net::worms_codec::WormCodec;
use crate::net::worms_packet::WormsPacket;
use anyhow::{anyhow, bail};
use futures_util::SinkExt;
use futures_util::StreamExt;
use log::{debug, error, info};

use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::Sender;
use tokio::time;
use tokio_util::bytes::Bytes;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;

pub(crate) struct Server;
impl Server {
    const AUTHORIZED_TTL: Duration = Duration::from_secs(10 * 60);
    const UNAUTHORIZED_TTL: Duration = Duration::from_secs(3);
    pub async fn start_server(
        database: Arc<Database>,
        address: impl ToSocketAddrs,
        token: CancellationToken,
    ) -> anyhow::Result<()> {
        let listen_result = TcpListener::bind(address).await;
        if let Err(e) = listen_result {
            return Err(anyhow!("Error starting TCP Listener: {}", e));
        }

        let listener = listen_result?;
        let local_addr = listener.local_addr().expect("Expected local address");
        info!("Server listening at {}", local_addr);
        info!("Press Ctrl + C to shutdown!");

        'server: loop {
            tokio::select! {
                listen_result = listener.accept() => {
                    if let Ok((stream, _)) = listen_result {
                        stream.set_nodelay(true)?;

                        let db_clone = Arc::clone(&database);
                        let token_clone = token.clone();
                        tokio::spawn(Server::handle_connection(stream, db_clone, token_clone));
                    }
                },
                _ = token.cancelled() => {
                    break 'server;
                }
            }
        }

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        database: Arc<Database>,
        cancellation_token: CancellationToken,
    ) -> anyhow::Result<()> {
        let mut user_id: u32 = 0;
        let mut timeout_duration = Server::UNAUTHORIZED_TTL;
        let sender_addr = stream.peer_addr()?;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Arc<Bytes>>(100);

        let framed = Framed::new(stream, WormCodec);
        let (mut sink, mut stream) = framed.split();

        'client: loop {
            tokio::select! {
                frame_result = time::timeout(timeout_duration, stream.next()) => match frame_result {
                    Ok(Some(Ok(ref packet))) => {
                        debug!("Received Packet: {:?}", packet);
                        match packet.header_code {
                            PacketCode::Login if user_id == 0 => {
                                let login_result = Server::login_client(&database,&packet, &tx).await;
                                match login_result {
                                    Ok(id) => {
                                        user_id = id;
                                        timeout_duration = Server::AUTHORIZED_TTL;
                                    },
                                    Err(e) => {
                                        error!("Error logging in: {}", e);
                                        break 'client;
                                    }
                                }
                            },

                            _ if user_id == 0 =>{continue;},

                            packet_code => {
                                if let Err(e) = packet_handler::dispatch(packet_code, &database, &tx, packet, user_id, &sender_addr).await {
                                    error!("Error Handling Packet: {}", e);
                                    break 'client;
                                }
                            },
                        }
                    },
                    Ok(Some(Err(e))) => {
                        // Handle error in received packet, break from loop if necessary
                        error!("Error receiving packet: {}", e);
                        break 'client;
                    },
                    Ok(None) => {
                        // The stream has ended normally (stream.next() returned None)
                        break 'client;
                    },
                    Err(e) => {
                        // Timeout occurred
                        info!("Timeout {}: {}", user_id, e);
                        break 'client;
                    }
                },

                rx_result = rx.recv() => {
                    if let Some(packet) = rx_result {
                                    let mut sent_packets = 1usize;

                                    if let Err(e) =sink.feed(packet).await {
                                        error!("Error feeding packet! {}", e);
                                        break 'client;
                                    }
                                    // see if we have more packets
                                    'packet_feed: while let Ok(packet) = rx.try_recv() {
                                        sent_packets += 1;
                                         if let Err(e) =sink.feed(packet).await {
                                            error!("Error feeding packet! {}", e);
                                            break 'client;
                                        }
                                        // No overwhelming
                                        if sent_packets >= 20 {
                                            break 'packet_feed;
                                        }
                                    }
                                     if let Err(e) =sink.flush().await {
                                        error!("Error flushing sink! {}", e);
                                        break 'client;
                                    }
                                } else {
                                    // The channel has been closed
                                    break 'client;
                                }
                            },
                _ = cancellation_token.cancelled() => {
                    // since server is shutting down, no need to notify and remove and all
                    return Ok(());
                }
            }
        }

        Server::disconnect_user(Arc::clone(&database), user_id).await?;

        Ok(())
    }

    async fn login_client(
        db: &Arc<Database>,
        packet: &Arc<WormsPacket>,
        tx: &Sender<Arc<Bytes>>,
    ) -> anyhow::Result<u32> {
        let name = packet.name.as_ref().ok_or(anyhow!("No name specified!"))?;
        let session_nation = packet
            .session
            .as_ref()
            .map(|s| s.nation.clone())
            .ok_or(anyhow!("No nation specified!"))?;

        if Database::name_exists(db, name).await {
            let packet = WormsPacket::new(PacketCode::LoginReply)
                .value_1(0)
                .error_code(1)
                .build()?;
            tx.send(packet).await?;
            bail!("Failed to login: Name already exists")
        } else {
            let new_id = Database::get_next_id(db).await;
            let new_user = User::new(tx.clone().downgrade(), new_id, name, session_nation);

            info!("User '{}' {} joined!", name, new_id);

            let packet = WormsPacket::new(PacketCode::Login)
                .value_1(new_id)
                .value_4(0)
                .name(name)
                .session(new_user.session.clone())
                .build()?;
            db.users.insert(new_id, new_user);
            Server::broadcast_all(Arc::clone(db), packet).await?;

            let packet = WormsPacket::new(PacketCode::LoginReply)
                .value_1(new_id)
                .error_code(0)
                .build()?;
            tx.send(packet).await?;

            Ok(new_id)
        }
    }

    pub async fn broadcast_all(db: Arc<Database>, packet: Arc<Bytes>) -> anyhow::Result<()> {
        for user in db.users.iter() {
            if let Err(e) = user.send_packet(Arc::clone(&packet)).await {
                error!("Error sending packet! {}", e);
            }
        }

        Ok(())
    }
    pub async fn broadcast_all_except(
        db: Arc<Database>,
        packet: Arc<Bytes>,
        ignored: &u32,
    ) -> anyhow::Result<()> {
        for user in db.users.iter() {
            if user.id == *ignored {
                continue;
            }

            if let Err(e) = user.send_packet(Arc::clone(&packet)).await {
                error!("Error sending packet! {}", e);
            }
        }

        Ok(())
    }

    pub async fn disconnect_user(db: Arc<Database>, client_id: u32) -> anyhow::Result<()> {
        if client_id < Database::ID_START {
            return Ok(());
        }

        info!("Disconnecting User: '{}'", {
            db.users
                .get(&client_id)
                .map_or(client_id.to_string(), |u| u.name.to_string())
        });

        let mut left_id = client_id;
        let old_user = db.users.remove(&client_id);

        let (mut room_id, client_name) =
            old_user.map_or((0, "".to_string()), |(_, u)| (u.room_id, u.name.clone()));

        // check existing games
        if let Some((_, lookup_gid)) = db.user_to_game.remove(&client_name) {
            if let Some((game_id, game)) = db.games.remove(&lookup_gid) {
                room_id = game.room_id;
                left_id = game_id;

                debug!("Removing Game '{}'", game.name);
                let leave_packet = WormsPacket::new(PacketCode::Leave)
                    .value_2(left_id)
                    .value_10(client_id)
                    .build()?;
                let close_packet = WormsPacket::new(PacketCode::Close)
                    .value_10(left_id)
                    .build()?;

                Server::broadcast_all(Arc::clone(&db), leave_packet).await?;
                Server::broadcast_all(Arc::clone(&db), close_packet).await?;
            }
        }

        Server::leave_room(Arc::clone(&db), room_id, left_id)
            .await
            .map_err(|e| anyhow!("Error leaving room for id '{}': {}", client_id, e))?;

        let packet = WormsPacket::new(PacketCode::DisconnectUser)
            .value_10(client_id)
            .build()?;
        Server::broadcast_all(db, packet).await?;

        Ok(())
    }

    pub async fn leave_room(db: Arc<Database>, room_id: u32, left_id: u32) -> anyhow::Result<()> {
        let room_exists = db.rooms.contains_key(&room_id);

        // Close an abandoned room.
        let room_abandoned = {
            if room_exists {
                let any_users_connected = db
                    .users
                    .iter()
                    .any(|u| u.id != left_id && u.room_id == room_id);
                let any_games_connected = db
                    .games
                    .iter()
                    .any(|g| g.id != left_id && g.room_id == room_id);

                !any_users_connected && !any_games_connected
            } else {
                false
            }
        };

        if room_abandoned {
            if let Some(room) = db.rooms.remove(&room_id) {
                debug!("Removed room '{}'", room.1.name);
            }
        }

        // Notify users
        if room_exists {
            let packet = WormsPacket::new(PacketCode::Leave)
                .value_2(room_id)
                .value_10(left_id)
                .build()?;
            Server::broadcast_all_except(Arc::clone(&db), packet, &left_id).await?;
        }

        if room_abandoned {
            let packet = WormsPacket::new(PacketCode::Close)
                .value_10(room_id)
                .build()?;
            Server::broadcast_all_except(Arc::clone(&db), packet, &left_id).await?;
        }

        Ok(())
    }
}
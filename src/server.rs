use crate::database::user::User;
use crate::database::{Database, DATABASE, SHUTDOWN_TOKEN};
use crate::net::packet_code::PacketCode;
use crate::net::packet_handler;
use crate::net::worms_codec::WormCodec;
use crate::net::worms_packet::WormsPacket;
use eyre::{bail, eyre, Result, WrapErr};
use futures_util::StreamExt;
use futures_util::{FutureExt, SinkExt};
use governor::{Quota, RateLimiter};
use log::{debug, error, info};

use futures_util::future::join_all;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::Sender;
use tokio::time;
use tokio_util::bytes::Bytes;
use tokio_util::codec::Framed;

pub(crate) struct Server;

impl Server {
    const AUTHORIZED_TTL: Duration = Duration::from_secs(10 * 60);
    const UNAUTHORIZED_TTL: Duration = Duration::from_secs(3);

    pub async fn start_server(address: impl ToSocketAddrs) -> Result<()> {
        let cancellation_token = SHUTDOWN_TOKEN.clone();
        let rate_limiter = RateLimiter::dashmap(Quota::per_second(NonZeroU32::new(1).unwrap()));

        let listener = TcpListener::bind(address).await?;
        let local_addr = listener
            .local_addr()
            .map_err(|e| eyre!("Unable to get local address: {}", e))?;

        println!("Server listening at {local_addr}");
        println!("Press Ctrl + C to shutdown!");
        'server: loop {
            tokio::select! {
                listen_result = listener.accept() => {
                    if let Ok((stream, _)) = listen_result {
                        // Limit login attempts to 1 per second
                        let addr = stream.peer_addr()?;
                        if let Err(_) = rate_limiter.check_key(&addr) {
                            error!("Rate limit exceeded for {}", addr);
                            continue;
                        }

                        // Set TCP_NODELAY to true
                        stream.set_nodelay(true)?;

                        // Handle the connection in a separate task
                        tokio::spawn(Server::handle_connection(stream));
                    }
                },
                    () = cancellation_token.cancelled().fuse() => {
                    break 'server;
                }
            }
        }

        Ok(())
    }

    async fn handle_connection(stream: TcpStream) -> Result<()> {
        let user_id;

        let cancellation_token = SHUTDOWN_TOKEN.clone();
        let rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));
        let mut limited_count = 0;
        const MAX_LIMITED_COUNT: u32 = 10;

        let sender_addr = stream.peer_addr()?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Arc<Bytes>>(100);
        let framed = Framed::new(stream, WormCodec);
        let (mut sink, mut stream) = framed.split();

        let mut packets_to_send = Vec::with_capacity(50);

        // authorize the client
        let packet = time::timeout(Server::UNAUTHORIZED_TTL, stream.next()).await?;
        if let Some(Ok(ref packet)) = packet {
            if packet.header_code != PacketCode::Login {
                bail!("First packet must be a login packet");
            }

            let login_result = Server::login_client(packet, &tx).await;
            match login_result {
                Ok(id) => {
                    user_id = id;
                }
                Err(e) => {
                    error!("Error logging in: {}", e);
                    return Ok(());
                }
            }
        } else {
            bail!("First packet must be a login packet");
        }

        // main loop for the client connection handling packets and sending them out
        'client: loop {
            tokio::select! {
                frame_result = time::timeout(Server::AUTHORIZED_TTL, stream.next()) => {
                    // Limit packets to 5 per second
                    if rate_limiter.check().is_err() {
                        limited_count += 1;

                        // If the user sends too many packets, disconnect them
                        if limited_count > MAX_LIMITED_COUNT {
                            error!("Rate limit exceeded for {}", sender_addr);
                            break 'client;
                        }

                        continue;
                    } else {
                        limited_count = 0;
                    }

                    match frame_result {
                        Ok(Some(Ok(packet))) => {
                            debug!("Received Packet: {:?}", packet);

                            if user_id < Database::ID_START {
                                break 'client; // Disconnect invalid users
                            }

                            if let Err(e) = packet_handler::dispatch(
                                packet.header_code,
                                tx.clone(),
                                packet.clone(),
                                user_id,
                                sender_addr,
                            ).await
                            {
                                error!("Error Handling Packet: {}", e);
                                break 'client;
                            }
                        }
                        Ok(Some(Err(e))) => {
                            error!("Error receiving packet: {}", e);
                            break 'client;
                        }
                        Ok(None) => break 'client, // Stream ended
                        Err(e) => {
                            info!("Timeout {}: {}", user_id, e);
                            break 'client;
                        }
                    }
                },
                // Receive up to 50 packets to send at a time
                packet_count = rx.recv_many(&mut packets_to_send, 50) => {
                    // if the result's 0, this channel has been closed
                    if packet_count == 0 {
                        break 'client;
                    }

                    // Drain and send each packet in the batch
                    // Sadly since some packets depends on order we can't parallelize this
                    let packets = packets_to_send.drain(..packet_count);
                    for packet in packets {
                        if let Err(e) = sink.feed(packet).await {
                            error!("Error sending packet: {}", e);
                            break 'client;
                        }
                    }

                    // Flush all packets since send did flush apparently
                    if let Err(e) = sink.flush().await {
                        error!("Error flushing packets: {}", e);
                        break 'client;
                    }
                },
                () = cancellation_token.cancelled().fuse() => {
                    return Ok(());
                }
            }
        }

        Server::disconnect_user(user_id).await?;
        Ok(())
    }

    async fn login_client(packet: &Arc<WormsPacket>, tx: &Sender<Arc<Bytes>>) -> Result<u32> {
        let name = packet.name.as_ref().ok_or(eyre!("No name specified!"))?;
        let session_nation = packet
            .session
            .as_ref()
            .map(|s| s.nation)
            .ok_or(eyre!("No nation specified!"))?;

        if Database::check_user_exists(name) {
            let packet = WormsPacket::create(PacketCode::LoginReply)
                .with_value_1(0)
                .with_error_code(1)
                .build()?;
            tx.send(packet).await?;
            bail!("Failed to login: Name already exists")
        }

        let new_id = Database::get_next_id();
        let new_user = User::new(tx.clone().downgrade(), new_id, name, session_nation);

        info!("User '{}' {} joined!", name, new_id);

        let packet = WormsPacket::create(PacketCode::Login)
            .with_value_1(new_id)
            .with_value_4(0)
            .with_name(name)
            .with_session(&new_user.session)
            .build()?;

        DATABASE.users.insert(new_id, new_user);
        Server::broadcast_all(packet).await?;

        let packet = WormsPacket::create(PacketCode::LoginReply)
            .with_value_1(new_id)
            .with_error_code(0)
            .build()?;
        tx.send(packet).await?;

        Ok(new_id)
    }

    async fn broadcast_all_with_filter<F>(packet: Arc<Bytes>, filter: F) -> Result<(), eyre::Error>
    where
        F: Fn(&u32) -> bool,
    {
        let futures = DATABASE.users.iter().filter_map(|entry| {
            if filter(entry.key()) {
                let packet = Arc::clone(&packet);
                Some(async move {
                    if let Err(e) = entry.value().send_packet(packet).await {
                        error!(
                            "Error sending packet to user {}: {:?}",
                            entry.value().name,
                            e
                        );
                    }
                })
            } else {
                None
            }
        });

        join_all(futures).await;

        Ok(())
    }

    pub async fn broadcast_all(packet: Arc<Bytes>) -> Result<()> {
        Self::broadcast_all_with_filter(packet, |_| true).await
    }
    pub async fn broadcast_all_except(packet: Arc<Bytes>, ignored: &u32) -> Result<()> {
        Self::broadcast_all_with_filter(packet, |user_id| *user_id != *ignored).await
    }

    pub async fn disconnect_user(client_id: u32) -> Result<()> {
        if client_id < Database::ID_START {
            return Ok(());
        }

        info!("Disconnecting User: '{}'", {
            DATABASE
                .users
                .get(&client_id)
                .map_or(client_id.to_string(), |u| u.name.to_string())
        });

        let mut left_id = client_id;
        let old_user = DATABASE.users.remove(&client_id);

        let (mut room_id, client_name) =
            old_user.map_or((0, String::new()), |(_, u)| (u.room_id, u.name.clone()));

        DATABASE.games.retain(|cur_id, cur_game| {
            if cur_game.name == client_name {
                room_id = cur_game.room_id;
                left_id = *cur_id;

                debug!("Removing Game '{}'", cur_game.name);
                let leave_packet = WormsPacket::create(PacketCode::Leave)
                    .with_value_2(left_id)
                    .with_value_10(client_id)
                    .build()
                    .expect("Packet should build without a worry");
                let close_packet = WormsPacket::create(PacketCode::Close)
                    .with_value_10(left_id)
                    .build()
                    .expect("Packet should build without a worry");

                tokio::spawn(async move {
                    Server::broadcast_all(leave_packet).await.unwrap();
                    Server::broadcast_all(close_packet).await.unwrap();
                });

                false // remove this entry
            } else {
                true // keep this entry
            }
        });

        Server::leave_room(room_id, left_id)
            .await
            .wrap_err_with(|| format!("Failed to leave room {room_id}"))?;

        let packet = WormsPacket::create(PacketCode::DisconnectUser)
            .with_value_10(client_id)
            .build()?;
        Server::broadcast_all(packet).await?;

        Ok(())
    }

    pub async fn leave_room(room_id: u32, left_id: u32) -> Result<()> {
        let room_exists = DATABASE.rooms.contains_key(&room_id);

        // Close an abandoned room.
        let room_abandoned = {
            if room_exists {
                let any_users_connected = DATABASE
                    .users
                    .iter()
                    .any(|u| u.id != left_id && u.room_id == room_id);

                let any_games_connected = DATABASE
                    .games
                    .iter()
                    .any(|g| g.id != left_id && g.room_id == room_id);

                !any_users_connected && !any_games_connected
            } else {
                false
            }
        };

        if room_abandoned {
            if let Some(room) = DATABASE.rooms.remove(&room_id) {
                debug!("Removed room '{}'", room.1.name);
            }
        }

        // Notify users
        if room_exists {
            let packet = WormsPacket::create(PacketCode::Leave)
                .with_value_2(room_id)
                .with_value_10(left_id)
                .build()?;
            Server::broadcast_all_except(packet, &left_id).await?;
        }

        if room_abandoned {
            let packet = WormsPacket::create(PacketCode::Close)
                .with_value_10(room_id)
                .build()?;
            Server::broadcast_all_except(packet, &left_id).await?;
        }

        Ok(())
    }
}

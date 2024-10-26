use eyre::{eyre, Error, Result};

/// Represents the description of the packet contents, as seen from client-side (thus a "reply" comes from the
/// server).
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum PacketCode {
    ListRooms = 200,
    ListItem = 350,
    ListEnd = 351,
    ListUsers = 400,
    ListGames = 500,
    Login = 600,
    LoginReply = 601,
    CreateRoom = 700,
    CreateRoomReply = 701,
    Join = 800,
    JoinReply = 801,
    Leave = 900,
    LeaveReply = 901,
    DisconnectUser = 1000,
    Close = 1100,
    CloseReply = 1101,
    CreateGame = 1200,
    CreateGameReply = 1201,
    ChatRoom = 1300,
    ChatRoomReply = 1301,
    ConnectGame = 1326,
    ConnectGameReply = 1327,
    #[default]
    Unknown = 0,
}

impl From<PacketCode> for u32 {
    fn from(value: PacketCode) -> Self {
        match value {
            PacketCode::ListRooms => 200,
            PacketCode::ListItem => 350,
            PacketCode::ListEnd => 351,
            PacketCode::ListUsers => 400,
            PacketCode::ListGames => 500,
            PacketCode::Login => 600,
            PacketCode::LoginReply => 601,
            PacketCode::CreateRoom => 700,
            PacketCode::CreateRoomReply => 701,
            PacketCode::Join => 800,
            PacketCode::JoinReply => 801,
            PacketCode::Leave => 900,
            PacketCode::LeaveReply => 901,
            PacketCode::DisconnectUser => 1000,
            PacketCode::Close => 1100,
            PacketCode::CloseReply => 1101,
            PacketCode::CreateGame => 1200,
            PacketCode::CreateGameReply => 1201,
            PacketCode::ChatRoom => 1300,
            PacketCode::ChatRoomReply => 1301,
            PacketCode::ConnectGame => 1326,
            PacketCode::ConnectGameReply => 1327,
            PacketCode::Unknown => 0,
        }
    }
}

impl TryFrom<u32> for PacketCode {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(PacketCode::ListRooms),
            350 => Ok(PacketCode::ListItem),
            351 => Ok(PacketCode::ListEnd),
            400 => Ok(PacketCode::ListUsers),
            500 => Ok(PacketCode::ListGames),
            600 => Ok(PacketCode::Login),
            601 => Ok(PacketCode::LoginReply),
            700 => Ok(PacketCode::CreateRoom),
            701 => Ok(PacketCode::CreateRoomReply),
            800 => Ok(PacketCode::Join),
            801 => Ok(PacketCode::JoinReply),
            900 => Ok(PacketCode::Leave),
            901 => Ok(PacketCode::LeaveReply),
            1000 => Ok(PacketCode::DisconnectUser),
            1100 => Ok(PacketCode::Close),
            1101 => Ok(PacketCode::CloseReply),
            1200 => Ok(PacketCode::CreateGame),
            1201 => Ok(PacketCode::CreateGameReply),
            1300 => Ok(PacketCode::ChatRoom),
            1301 => Ok(PacketCode::ChatRoomReply),
            1326 => Ok(PacketCode::ConnectGame),
            1327 => Ok(PacketCode::ConnectGameReply),
            _ => Err(eyre!("Invalid Packet Code {}", value)),
        }
    }
}
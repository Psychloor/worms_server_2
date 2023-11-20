#[derive(Debug, Default, PartialOrd, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum SessionType {
    #[default]
    Room = 1,
    Game = 4,
    User = 5,
}

impl From<SessionType> for u8 {
    fn from(value: SessionType) -> Self {
        match value {
            SessionType::Room => 1,
            SessionType::Game => 4,
            SessionType::User => 5,
        }
    }
}

impl TryFrom<u8> for SessionType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SessionType::Room),
            4 => Ok(SessionType::Game),
            5 => Ok(SessionType::User),
            _ => Err(()),
        }
    }
}
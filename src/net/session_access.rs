use eyre::{bail, Error};

#[derive(Debug, Default, PartialOrd, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum SessionAccess {
    #[default]
    Public = 1,
    Protected = 2,
}

impl From<SessionAccess> for u8 {
    fn from(value: SessionAccess) -> Self {
        match value {
            SessionAccess::Public => 1,
            SessionAccess::Protected => 2,
        }
    }
}

impl TryFrom<u8> for SessionAccess {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SessionAccess::Public),
            2 => Ok(SessionAccess::Protected),
            _ => bail!("Invalid session access value: {}", value),
        }
    }
}

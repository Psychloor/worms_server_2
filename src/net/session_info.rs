use crate::net::nation::Nation;
use crate::net::session_access::SessionAccess;
use crate::net::session_type::SessionType;
use std::sync::Arc;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct SessionInfo {
    pub nation: Nation,
    pub game_release: u8,
    pub session_type: SessionType,
    pub access: SessionAccess,
}

impl SessionInfo {
    pub fn new(nation: Nation, session_type: SessionType) -> Arc<Self> {
        Arc::new(Self {
            nation,
            session_type,
            ..Default::default()
        })
    }

    pub fn new_with_access(
        nation: Nation,
        session_type: SessionType,
        session_access: SessionAccess,
    ) -> Arc<Self> {
        Arc::new(Self {
            nation,
            session_type,
            access: session_access,
            ..Default::default()
        })
    }
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self {
            nation: Default::default(),
            game_release: 49,
            session_type: Default::default(),
            access: Default::default(),
        }
    }
}
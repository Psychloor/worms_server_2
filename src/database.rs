pub(crate) mod game;
pub(crate) mod room;
pub(crate) mod user;

use crate::database::game::Game;
use crate::database::room::Room;
use crate::database::user::User;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct Database {
    pub users: DashMap<u32, User>,
    pub rooms: DashMap<u32, Room>,
    pub games: DashMap<u32, Game>,
    pub user_to_game: DashMap<String, u32>,
    next_id: AtomicU32,
}

impl Database {
    pub(crate) const ID_START: u32 = 0x1000;
    const STARTING_CAPACITY: usize = 1024;

    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            users: DashMap::with_capacity(Database::STARTING_CAPACITY),
            rooms: DashMap::with_capacity(Database::STARTING_CAPACITY),
            games: DashMap::with_capacity(Database::STARTING_CAPACITY),
            user_to_game: DashMap::with_capacity(Database::STARTING_CAPACITY),
            next_id: AtomicU32::new(Database::ID_START),
        })
    }

    pub async fn get_next_id(db: &Arc<Database>) -> u32 {
        db.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn name_exists(db: &Arc<Database>, name: &str) -> bool {
        db.users.iter().any(|u| u.name.eq_ignore_ascii_case(name))
    }
}
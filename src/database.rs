pub(crate) mod game;
pub(crate) mod room;
pub(crate) mod user;

use crate::database::game::Game;
use crate::database::room::Room;
use crate::database::user::User;
use dashmap::DashMap;
use nohash_hasher::BuildNoHashHasher;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::LazyLock;
use tokio_util::sync::CancellationToken;

pub static DATABASE: LazyLock<Database> = LazyLock::new(Database::initialize);
pub static SHUTDOWN_TOKEN: LazyLock<CancellationToken> = LazyLock::new(CancellationToken::new);

pub struct Database {
    pub users: DashMap<u32, User, BuildNoHashHasher<u32>>,
    pub rooms: DashMap<u32, Room, BuildNoHashHasher<u32>>,
    pub games: DashMap<u32, Game, BuildNoHashHasher<u32>>,
    next_id: AtomicU32,
    reusable_ids: Mutex<Vec<u32>>,
}

impl Database {
    pub(crate) const ID_START: u32 = 0x1000;
    const STARTING_CAPACITY: usize = 1024;

    fn initialize() -> Self {
        Self {
            users: DashMap::with_capacity_and_hasher(
                Self::STARTING_CAPACITY,
                BuildNoHashHasher::default(),
            ),
            rooms: DashMap::with_capacity_and_hasher(
                Self::STARTING_CAPACITY,
                BuildNoHashHasher::default(),
            ),
            games: DashMap::with_capacity_and_hasher(
                Self::STARTING_CAPACITY,
                BuildNoHashHasher::default(),
            ),
            next_id: AtomicU32::new(Database::ID_START),
            reusable_ids: Mutex::new(Vec::new()),
        }
    }

    pub async fn get_next_id() -> u32 {
        if let Some(id) = DATABASE.reusable_ids.lock().pop() {
            return id;
        }

        DATABASE.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn recycle_id(id: u32) {
        if SHUTDOWN_TOKEN.is_cancelled() {
            return;
        }
        if id >= Database::ID_START {
            DATABASE.reusable_ids.lock().push(id);
        }
    }

    pub async fn check_user_exists(name: &str) -> bool {
        DATABASE
            .users
            .iter()
            .any(|u| u.name.eq_ignore_ascii_case(name))
    }
}

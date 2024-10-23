pub(crate) mod game;
pub(crate) mod room;
pub(crate) mod user;

use crate::database::game::Game;
use crate::database::room::Room;
use crate::database::user::User;
use dashmap::DashMap;
use nohash_hasher::BuildNoHashHasher;
use rustc_hash::FxBuildHasher;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;
use tokio::task;

pub struct Database {
    pub users: DashMap<u32, User, BuildNoHashHasher<u32>>,
    pub rooms: DashMap<u32, Room, BuildNoHashHasher<u32>>,
    pub games: DashMap<u32, Game, BuildNoHashHasher<u32>>,
    pub user_to_game: DashMap<String, u32, FxBuildHasher>,
    next_id: AtomicU32,
    reusable_ids: Arc<Mutex<Vec<u32>>>,
}

impl Database {
    pub(crate) const ID_START: u32 = 0x1000;
    const STARTING_CAPACITY: usize = 1024;

    pub fn new() -> Arc<Self> {
        Arc::new(Self {
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
            user_to_game: DashMap::with_capacity_and_hasher(Self::STARTING_CAPACITY, FxBuildHasher),
            next_id: AtomicU32::new(Database::ID_START),
            reusable_ids: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn get_next_id(db: &Arc<Database>) -> u32 {
        let mut lock = db.reusable_ids.lock().await;
        if let Some(id) = lock.pop() {
            return id;
        }

        db.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn recycle_id(db: Weak<Database>, id: u32) {
        if id >= Database::ID_START {
            task::spawn(async move {
                if let Some(db) = db.upgrade() {
                    db.reusable_ids.lock().await.push(id);
                }
            });
        }
    }

    pub async fn check_user_exists(db: &Arc<Database>, name: &str) -> bool {
        db.users.iter().any(|u| u.name.eq_ignore_ascii_case(name))
    }
}

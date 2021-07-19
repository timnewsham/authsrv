
use crate::db;

pub struct ServerState {
    pub db: db::DB,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            db: db::DB::new(),
        }
    }
}

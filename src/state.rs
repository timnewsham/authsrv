
use std::sync::Mutex;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub struct ServerState {
    pub rng: Mutex<StdRng>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            rng: Mutex::new(StdRng::from_entropy()),
        }
    }
}

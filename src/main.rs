
#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket;

mod api;
mod cache;
mod json;
mod model;
mod redis_support;

use std::sync::Mutex;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rocket::State;                                                              
use rocket::serde::Deserialize; 
use rocket_sync_db_pools::{database};                                   

pub type Result<T> = std::result::Result<T, String>;                                
                                                                                
pub fn errstr(x: impl ToString) -> String {                                         
    x.to_string()                                                               
} 

#[database("diesel")]                                                           
pub struct Db(diesel::PgConnection);  

#[database("redis")]                                                           
pub struct Cache(redis_support::Connection);

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")] 
struct AppConfig {
    use_tests: bool,
    use_cache: bool,
    cache_lifetime: u32,
    token_lifetime: u64,
}

pub type Server = State<ServerState>;
pub struct ServerState {
    pub rng: Mutex<StdRng>,
    pub use_cache: bool,
    pub cache_lifetime: u32,
    pub token_lifetime: u64,
}

impl ServerState {
    fn new(cfg: &AppConfig) -> Self {
        ServerState {
            rng: Mutex::new(StdRng::from_entropy()),
            use_cache: cfg.use_cache,
            cache_lifetime: cfg.cache_lifetime,
            token_lifetime: cfg.token_lifetime,
        }
    }
}

#[launch]
fn rocket() -> _ {
    let mut b = rocket::build();
    let conf: AppConfig = b.figment().extract().expect("config");

    println!("caching {}", if conf.use_cache { "enabled" } else { "disabled "});
    if conf.use_tests {
        b = b.mount("/test", routes![
                api::test::health,
                api::test::hasher,
                api::test::crasher])
    }

    b.manage(ServerState::new(&conf))
        .attach(Db::fairing())
        .attach(Cache::fairing())
        .mount("/auth", routes![api::auth::auth])
}

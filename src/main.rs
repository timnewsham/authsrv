
mod api;
mod cache;
mod model;

#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket;
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

#[database("memcache")]
pub struct Cache(memcache::Client);

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")] 
struct AppConfig {
    use_tests: bool,
    use_cache: bool,
}

pub type Server = State<ServerState>;
pub struct ServerState {
    pub rng: Mutex<StdRng>,
    pub use_cache: bool,
}

impl ServerState {
    fn new(cfg: &AppConfig) -> Self {
        ServerState {
            rng: Mutex::new(StdRng::from_entropy()),
            use_cache: cfg.use_cache,
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

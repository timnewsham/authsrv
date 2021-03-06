
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket;

mod api;
mod cache;
mod json;
mod model;
mod redis_support;
mod rocktypes;

use std::sync::Mutex;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rocket::{Rocket, State, Build};
use rocket::serde::Deserialize;
use rocket::fairing::AdHoc;

use crate::rocktypes::{Db, Cache};

pub type Result<T> = std::result::Result<T, String>;

pub fn errstr(x: impl ToString) -> String {
    x.to_string()
}

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

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!("migrations");
    let conn = Db::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c)).await.expect("diesel migrations");
    rocket
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
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
        .attach(Cache::fairing())
        .mount("/auth", routes![api::auth::auth,
                                api::auth::check_auth])
        .mount("/admin", routes![api::admin::create_user,
                                 api::admin::create_scope,
                                 api::admin::clean])
}

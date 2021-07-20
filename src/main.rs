
#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket;

mod api;
mod db;
//mod schema;
mod state;

use rocket::serde::Deserialize; 
use rocket_sync_db_pools::{database};                                   

#[database("diesel")]                                                           
pub struct Db(diesel::PgConnection);  

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")] 
struct AppConfig {
    use_tests: bool,
}

#[launch]
fn rocket() -> _ {
    let mut b = rocket::build();
    let conf: AppConfig = b.figment().extract().expect("config");

    if conf.use_tests {
        b = b.mount("/test", routes![
                api::test::health,
                api::test::hasher,
                api::test::crasher])
    }

    b.manage(state::ServerState::new())
        .attach(Db::fairing())
        .mount("/auth", routes![api::auth::auth])
}

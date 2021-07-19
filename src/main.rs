
#[macro_use] extern crate rocket;

mod db;
mod state;
mod api;

//use rocket::fairing::AdHoc;
use rocket::serde::Deserialize; 

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
        .mount("/auth", routes![api::auth::auth])
}

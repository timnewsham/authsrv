
#[macro_use] extern crate rocket;

mod db;
mod state;
mod api;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(state::ServerState::new())
        .mount("/", routes![api::root::health, api::root::hasher])
        .mount("/auth", routes![api::auth::auth])
}

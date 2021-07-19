
use argon2;
use rocket::State;
use rocket::serde::{Serialize, Deserialize, json::Json};

use crate::state::ServerState;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthReq<'r> {
    name: &'r str,
    secret: &'r str,
    scopes: Vec<&'r str>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthState {
    token: String,
    scopes: Vec<String>,
    // XXX expire time, scopes, etc..
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthResp {
    status: String,
    result: Option<AuthState>,
}

fn auth_logic(server: &ServerState, req: &AuthReq) -> AuthResp {
    // this should be cleaner with the right combinators...
    let mut authed = false;
    if let Some(u) = server.db.get_user(req.name) {
        if let Ok(ok) = argon2::verify_encoded(&u.hash, req.secret.as_bytes()) {
            // grant if the user has all requested scopes
            authed = ok && req.scopes.iter().all(|&s| u.scopes.contains(s));
        }
    }
    if authed {
        let token = "XXX"; // XXX create token
        // XXX insert token into db
        let astate = AuthState {
            token: token.to_owned(),
            scopes: req.scopes.iter().copied().map(str::to_owned).collect(),
        };
        AuthResp{ status: "ok".to_string(), 
                  result: Some(astate) }
    } else {
        AuthResp{ status: "error".to_string(),
                  result: None }
    }
}

#[post("/", format="json", data="<req>")]
pub fn auth(server: &State<ServerState>, req: Json<AuthReq<'_>>) -> Json<AuthResp> {
    Json(auth_logic(&server, &req))
}


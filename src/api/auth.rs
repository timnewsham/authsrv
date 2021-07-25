
use std::collections::HashSet;
use std::sync::Mutex;
use argon2;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::{Db, Cache, Server};
use crate::model::user::{get_user};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthReq<'r> {
    name: &'r str,
    secret: &'r str,
    scopes: HashSet<&'r str>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthState {
    token: String,
    scopes: HashSet<String>,
    // XXX expire time, scopes, etc..
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthResp {
    status: &'static str,
    result: Option<AuthState>,
}

fn gen_token(rng: &Mutex<StdRng>) -> String {
    // unwrap() here can only panic if another thread already paniced while holding the mutex
    let bytes: [u8; 20] = rng.lock().unwrap().gen(); // safe
    bytes.encode_hex()
}

fn password_valid(hash: &str, pw: &str) -> bool {
    argon2::verify_encoded(hash, pw.as_bytes()).unwrap_or(false)
}

#[post("/", format="json", data="<req>")]
pub async fn auth(db: Db, cache: Cache, serv: &Server, req: Json<AuthReq<'_>>) -> Json<AuthResp> {
    let err = Json(AuthResp{ status: "error", result: None });

    // XXX to owned
    let u = match get_user(&db, &cache, serv, req.name.to_owned()).await {
        Ok(x) => x,
        _ => return err,
    };

    if !u.is_enabled() || !password_valid(&u.hash, &req.secret) {
        return err;
    }

    let have_scopes: HashSet<_> = u.scopes.iter().map(|s| s.as_ref()).collect();
    // XXX remove any scopes that are no longer defined
    if !req.scopes.is_subset(&have_scopes) {
        return err;
    }

     // XXX insert token into db
    let astate = AuthState {
        token: gen_token(&serv.rng),
        scopes: req.scopes.iter().copied().map(str::to_owned).collect(),
    };
    Json(AuthResp{ 
        status: "ok",
        result: Some(astate) 
    })
}


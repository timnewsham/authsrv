
use std::collections::HashSet;
use std::sync::Mutex;
use argon2;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::{Db, Cache, Server};
use crate::model::{user, scopes};

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
    let u = match user::get_user(&db, &cache, serv, req.name.to_owned()).await {
        Ok(x) => x,
        _ => return err,
    };
    let all_scopes: Vec<String> = match scopes::get_scopes(&db, &cache, serv).await {
        Ok(x) => x,
        _ => return err,
    };

    // fail if disabled, expired, or if provided credentials are bad
    if !u.is_enabled() || !password_valid(&u.hash, &req.secret) {
        return err;
    }

    /*
    // filter out any scopes that are no longer valid
    // XXX this is ugly.. there has to be a cleaner way
    let all_scope_set: HashSet<&str> = all_scopes.iter().map(|s| s.as_ref()).collect();
    let req_scopes: HashSet<&str> = req.scopes.iter().map(|s| *s).collect();
    let have_scopes_pre: HashSet<&str> = u.scopes.iter().map(|s| s.as_ref()).collect();
    let have_scopes: HashSet<&str> = all_scope_set.intersection(&have_scopes_pre).map(|s| s.as_ref()).collect();

    // fail if any requested scopes don't belong to the user
    if !req_scopes.is_subset(&have_scopes) {
        return err;
    }
    */
    // fail if any requested scope is no longer active or doesnt belong to the user
    for want in req.scopes.iter() {
        if !all_scopes.iter().any(|active| want == active)
        || !u.scopes.iter().any(|have| want == have) {
            return err;
        }
    }

    // XXX insert token into db
    let astate = AuthState {
        token: gen_token(&serv.rng),
        scopes: req.scopes.iter().map(|s| (*s).to_owned()).collect(),
    };
    Json(AuthResp{ 
        status: "ok",
        result: Some(astate) 
    })
}


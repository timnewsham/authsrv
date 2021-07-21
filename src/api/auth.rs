
use std::sync::Arc;
use argon2;
use rocket::serde::{Serialize, Deserialize, json::Json};
use diesel::table;
use rocket_sync_db_pools::diesel::prelude::*;
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::{Db, Cache, Server};
use crate::cache;

table! {
    users (name) {
        name -> Varchar,
        hash -> Varchar,
        scopes -> Array<Text>,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[table_name="users"]
struct User {
    name: String,
    hash: String,
    scopes: Vec<String>,
}

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

type Result<T> = std::result::Result<T, String>;

fn errstr(x: impl ToString) -> String {
    x.to_string()
}

async fn get_user(db: &Db, cache: &Cache, serv: &Server, name: String) -> Result<User> {
    let key = Arc::new(format!("user_{}", name));
    if let Some(u) = cache::get(&cache, serv, key.clone()).await {
        return Ok(u);
    }

    let u = db.run(move |c| users::table.filter(users::name.eq(&name)).first(c)).await.map_err(errstr)?;
    cache::put(&cache, serv, key, &u).await;
    Ok(u)
}

pub fn gen_token(rng: &mut StdRng) -> String {
    let bytes: [u8; 20] = rng.gen();
    bytes.encode_hex()
}

#[post("/", format="json", data="<req>")]
pub async fn auth(db: Db, cache: Cache, serv: &Server, req: Json<AuthReq<'_>>) -> Json<AuthResp> {
    let err = Json(AuthResp{ status: "error".to_owned(), result: None });

    // XXX to owned
    let u = match get_user(&db, &cache, serv, req.name.to_owned()).await {
        Ok(x) => x,
        _ => return err,
    };
    if !argon2::verify_encoded(&u.hash, req.secret.as_bytes()).unwrap_or(false) {
        return err;
    }

    // XXX return failure if user account is expired
    // XXX remove any scopes that are no longer defined

    // make sure the user has all of the requested scopes
    if !req.scopes.iter().all(|&req| u.scopes.iter().any(|have| req == have)) {
        return err;
    }

     // XXX insert token into db
    let astate = AuthState {
        // unwrap() here can only panic if another thread already paniced while holding the mutex
        token: gen_token(&mut serv.rng.lock().unwrap()), // safe
        scopes: req.scopes.iter().copied().map(str::to_owned).collect(),
    };
    Json(AuthResp{ 
        status: "ok".to_string(),
        result: Some(astate) 
    })
}


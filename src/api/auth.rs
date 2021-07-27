
use std::collections::HashSet;
use std::time::{Duration, SystemTime};
use std::sync::Mutex;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::rocktypes::{BearerToken, CachedDb};
use crate::model::{user, scopes, token};
use crate::json::{StatusErr, StrRes, JsonRes, json_res, ERR_FAILED, ERR_BADAUTH, ERR_BADSCOPES};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthReq<'r> {
    name: &'r str,
    secret: &'r str,
    scopes: HashSet<&'r str>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthResp {
    token: String,
    scopes: Vec<String>,
    life: u64,
}

fn gen_token(rng: &Mutex<StdRng>) -> String {
    // unwrap() here can only panic if another thread already paniced while holding the mutex
    let bytes: [u8; 20] = rng.lock().unwrap().gen(); // safe
    bytes.encode_hex()
}

fn password_valid(hash: &str, pw: &str) -> bool {
    argon2::verify_encoded(hash, pw.as_bytes()).unwrap_or(false)
}

fn scopes_valid(req_scopes: &HashSet<&'_ str>, have_scopes: &Vec<String>, active_scopes: &Vec<String>) -> bool {
    // fail if any requested scope is no longer active or doesnt belong to the user
    for want in req_scopes.iter() {
        if !active_scopes.iter().any(|active| want == active)
        || !have_scopes.iter().any(|have| want == have) {
            return false;
        }
    }
    return true;
}

fn catch_notfound(msg: String) -> StatusErr {
    if msg == "NotFound" {
        ERR_BADAUTH
    } else {
        println!("error {}", msg);
        ERR_FAILED
    }
}

pub async fn auth_sr(cdb: CachedDb<'_>, req: Json<AuthReq<'_>>) -> StrRes<AuthResp> {
    // XXX to owned
    let u = user::get_user(&cdb, req.name.to_owned()).await.map_err(catch_notfound)?;
    let active_scopes: Vec<String> = scopes::get_scopes(&cdb).await.or(Err(ERR_FAILED))?;

    // fail if disabled, expired, or if provided credentials are bad
    if !u.is_enabled()
    || !password_valid(&u.hash, &req.secret) {
        return Err(ERR_BADAUTH);
    }
    if !scopes_valid(&req.scopes, &u.scopes, &active_scopes) {
        return Err(ERR_BADSCOPES);
    }

    let tokstr = gen_token(&cdb.serv.rng);
    let life = Duration::new(cdb.serv.token_lifetime, 0);
    let exp = SystemTime::now() + life;
    let granted_scopes: Vec<String> = req.scopes.iter().copied().map(|s| s.to_owned()).collect();

    // add session to our store
    let tok = token::Token {
        token: tokstr.clone(),
        username: u.name.clone(),
        expiration: exp,
        scopes: granted_scopes.clone(),
    };
    token::put_token(&cdb, &tok).await.or(Err(ERR_FAILED))?;

    // and send it back to the user
    let astate = AuthResp {
        token: tokstr,
        scopes: granted_scopes,
        life: tok.seconds_left(),
    };
    Ok(astate)
}

#[post("/", format="json", data="<req>")]
pub async fn auth(cdb: CachedDb<'_>, req: Json<AuthReq<'_>>) -> JsonRes<AuthResp> {
    json_res(auth_sr(cdb, req).await)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TokenResp {
    username: String,
    life: u64,
    scopes: Vec<String>,
}

pub async fn check_auth_sr(cdb: CachedDb<'_>, bearer: BearerToken) -> StrRes<TokenResp> {
    let tok = bearer.lookup(&cdb).await?;
    let scopes: Vec<String> = tok.scopes.iter().map(|s| s.clone()).collect();
    let resp = TokenResp {
        username: tok.username.clone(),
        life: tok.seconds_left(),
        scopes: scopes,
    };
    Ok(resp)
}

#[get("/", format="json")]
pub async fn check_auth(cdb: CachedDb<'_>, bearer: BearerToken) -> JsonRes<TokenResp> {
    json_res(check_auth_sr(cdb, bearer).await)
}


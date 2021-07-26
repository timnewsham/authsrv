
use std::collections::HashSet;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use std::sync::Mutex;
use argon2;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::json::{JsonRes, IntoJErr, json_err, ERR_FAILED, ERR_BADAUTH, ERR_BADSCOPES};
use crate::model::{user, scopes, token};
use crate::rocktypes::{BearerToken, CachedDb};

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
    status: &'static str,
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

#[post("/", format="json", data="<req>")]
pub async fn auth(cdb: CachedDb<'_>, req: Json<AuthReq<'_>>) -> JsonRes<AuthResp> {
    //let cdb = CachedDb::new(&cache, &db, serv);

    // XXX to owned
    let u = user::get_user(&cdb, req.name.to_owned()).await.map_jerr(ERR_FAILED)?;
    let active_scopes: Vec<String> = scopes::get_scopes(&cdb).await.map_jerr(ERR_FAILED)?;

    // fail if disabled, expired, or if provided credentials are bad
    if !u.is_enabled()
    || !password_valid(&u.hash, &req.secret) {
        return json_err(ERR_BADAUTH);
    }
    if !scopes_valid(&req.scopes, &u.scopes, &active_scopes) {
        return json_err(ERR_BADSCOPES);
    }

    let tokstr = gen_token(&cdb.serv.rng);
    let life = Duration::new(cdb.serv.token_lifetime, 0);
    let exp = SystemTime::now().add(life);
    let granted_scopes: Vec<String> = req.scopes.iter().copied().map(|s| s.to_owned()).collect();

    // add session to our store
    let tok = token::Token {
        token: tokstr.clone(),
        username: u.name.clone(),
        expiration: exp,
        scopes: granted_scopes.clone(),
    };
    token::put_token(&cdb, &tok).await.map_jerr(ERR_FAILED)?;

    // and send it back to the user
    let astate = AuthResp {
        status: "ok",
        token: tokstr,
        scopes: granted_scopes,
        life: tok.seconds_left(),
    };
    Ok(Json(astate))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TokenResp {
    status: &'static str,
    //token: String,
    username: String,
    life: u64,
    scopes: Vec<String>,
}

#[get("/", format="json")]
pub async fn check_auth(cdb: CachedDb<'_>, bearer: BearerToken) -> JsonRes<TokenResp> {
    let tok = bearer.lookup(&cdb).await?;
    let scopes: Vec<String> = tok.scopes.iter().map(|s| s.clone()).collect();
    let resp = TokenResp {
        status: "ok",
        //token: tok.token.clone(),
        username: tok.username.clone(),
        life: tok.seconds_left(),
        scopes: scopes,
    };
    Ok(Json(resp))
}



use std::collections::HashSet;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use std::sync::Mutex;
use argon2;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::{Db, Cache, Server};
use crate::json::{JsonRes, IntoJErr, json_err, JsonError, ERR_FAILED, ERR_BADAUTH, ERR_BADSCOPES};
use crate::model::{user, scopes, token};

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
    scopes: Vec<String>,
    life: u64,
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

fn scopes_valid<'r>(req_scopes: &HashSet<&'r str>, have_scopes: &Vec<String>, active_scopes: &Vec<String>) -> bool {
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
pub async fn auth(db: Db, cache: Cache, serv: &Server, req: Json<AuthReq<'_>>) -> JsonRes<AuthResp> {
    // XXX to owned
    let u = user::get_user(&db, &cache, serv, req.name.to_owned()).await.map_jerr(ERR_FAILED)?;
    let active_scopes: Vec<String> = scopes::get_scopes(&db, &cache, serv).await.map_jerr(ERR_FAILED)?;

    // fail if disabled, expired, or if provided credentials are bad
    if !u.is_enabled()
    || !password_valid(&u.hash, &req.secret) {
        return json_err(ERR_BADAUTH);
    }
    if !scopes_valid(&req.scopes, &u.scopes, &active_scopes) {
        return json_err(ERR_BADSCOPES);
    }

    let tokstr = gen_token(&serv.rng);
    let life = Duration::new(serv.token_lifetime, 0);
    let exp = SystemTime::now().add(life);
    let granted_scopes: Vec<String> = req.scopes.iter().copied().map(|s| s.to_owned()).collect();

    // add session to our store
    let sess = token::Token {
        token: tokstr.clone(),
        username: u.name.clone(),
        expiration: exp,
        scopes: granted_scopes.clone(),
    };
    token::put_token(&db, &cache, serv, &sess).await.map_jerr(ERR_FAILED)?;

    // and send it back to the user
    let astate = AuthState {
        token: tokstr,
        scopes: granted_scopes,
        life: life.as_secs(),
    };
    Ok(Json(AuthResp{
        status: "ok",
        result: Some(astate)
    }))
}


///-------------------
use std::array::IntoIter;
use std::iter::FromIterator;
use rocket::request::{self, FromRequest, Request};  
use rocket::outcome::IntoOutcome;
use rocket::http::Status;

// Authorization information from bearer token
pub struct Authed {
    pub scopes: HashSet<String>,
}

fn validate_auth_header<'a>(hdr: &'a str) -> Option<Authed> {
    let tok = hdr.strip_prefix("bearer ")?;
    if tok != "XXX" {
        return None;
    }

    Some(Authed {
        scopes: HashSet::<_>::from_iter(IntoIter::new(["test".to_owned(), "authadmin".to_owned()])),
    })
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authed {
    type Error = Json<JsonError>;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Authed, Self::Error> {
        request.headers()
            .get_one("Authorization")
            .and_then(validate_auth_header)
            // XXX how can we return 401 and have it be json?
            //.into_outcome((Status::Unauthorized, Json(JsonError::new(ERR_BADAUTH))))
            .into_outcome((Status::Ok, Json(JsonError::new(ERR_BADAUTH))))
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ScopesResp {
    status: &'static str,
    result: HashSet<String>,
}

#[get("/", format="json")]
pub async fn check_auth(db: Db, cache: Cache, authed: Authed) -> Json<ScopesResp> {
    Json(ScopesResp {
        status: "succes",
        result: authed.scopes.clone(),
    })
}


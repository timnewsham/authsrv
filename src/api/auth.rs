
use std::collections::HashSet;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use std::sync::Mutex;
use argon2;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rand::{Rng, rngs::StdRng};
use hex::ToHex;

use crate::{Db, Cache, Server};
use crate::json::{JsonRes, IntoJErr, json_err, JsonError, ERR_FAILED, ERR_BADAUTH, ERR_BADSCOPES, ERR_EXPIRED};
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
    let tok = token::Token {
        token: tokstr.clone(),
        username: u.name.clone(),
        expiration: exp,
        scopes: granted_scopes.clone(),
    };
    token::put_token(&db, &cache, serv, &tok).await.map_jerr(ERR_FAILED)?;

    // and send it back to the user
    let astate = AuthResp {
        status: "ok",
        token: tokstr,
        scopes: granted_scopes,
        life: tok.seconds_left(),
    };
    Ok(Json(astate))
}


///-------------------
use rocket::request::{self, FromRequest, Request};  
use rocket::outcome::IntoOutcome;
use rocket::http::Status;

// Authorization information from bearer token
pub struct BearerToken {
    pub header: String,
}

impl BearerToken {
    pub fn new(hdr: &str) -> Self {
        BearerToken{ header: hdr.to_owned() }
    }

    // Lookup the token data associated with the bearer token and return it or an auth error
    pub async fn lookup(&self, db: &Db, cache: &Cache, serv: &Server) -> Result<token::Token, Json<JsonError>> {
        let tok = token::get_token(db, cache, serv, self.header.clone()).await.map_jerr(ERR_BADAUTH)?;
        if tok.is_expired() {
            json_err(ERR_EXPIRED)
        } else {
            Ok(tok)
        }
    }

    // Return an auth error if scope isn't associated with the bearer token
    pub async fn require_scope(&self, db: &Db, cache: &Cache, serv: &Server, scope: &str) -> Result<(), Json<JsonError>> {
        let tok = self.lookup(db, cache, serv).await?;
        if tok.scopes.iter().any(|have| have == scope) {
            Ok(())
        } else {
            json_err(ERR_BADAUTH)
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = Json<JsonError>;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<BearerToken, Self::Error> {
        request.headers()
            .get_one("Authorization")
            .and_then(|h| h.strip_prefix("bearer "))
            .map(BearerToken::new)
            // XXX how can we return 401 and have it be json?
            //.into_outcome((Status::Unauthorized, Json(JsonError::new(ERR_BADAUTH))))
            .into_outcome((Status::Ok, Json(JsonError::new(ERR_BADAUTH))))
    }
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
pub async fn check_auth(db: Db, cache: Cache, serv: &Server, bearer: BearerToken) -> JsonRes<TokenResp> {
    let tok = bearer.lookup(&db, &cache, serv).await?;
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


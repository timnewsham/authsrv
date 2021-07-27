
use rand::{Rng, rngs::StdRng};
use std::collections::HashSet;
use std::ops::Add;
use std::sync::Mutex;
use std::time::{SystemTime, Duration};
use rocket::serde::{Deserialize, json::Json};

use crate::cache;
use crate::rocktypes::{BearerToken, CachedDb};
use crate::model::{user, scopes, token};
use crate::json::{StrRes, JsonRes, json_res, ERR_FAILED, ERR_BADSCOPES};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateReq<'r> {
    pub name: &'r str,
    pub secret: &'r str,
    pub life: u64,
    //pub enable: '&r str,
    scopes: HashSet<&'r str>,
}

fn hash_password(rng: &Mutex<StdRng>, secret: &str) -> String {
    let hash_config = argon2::Config::default(); // XXX config?
    let salt: [u8; 20] = rng.lock().unwrap().gen(); // safe
    let hash = argon2::hash_encoded(secret.as_bytes(), &salt, &hash_config).unwrap();
    hash
}

fn scopes_valid(req_scopes: &HashSet<&str>, active_scopes: &Vec<String>) -> bool {
    // fail if any requested scope is not an active scope
    for want in req_scopes.iter() {
        if !active_scopes.iter().any(|active| want == active) {
            return false;
        }
    }
    return true;
}

// XXX make some of the fields optional?

async fn create_user_sr(cdb: CachedDb<'_>, bearer: BearerToken, req: Json<CreateReq<'_>>) -> StrRes<&'static str> {
    bearer.require_scope(&cdb, "authadmin").await?;
    let active_scopes: Vec<String> = scopes::get_scopes(&cdb).await.or(Err(ERR_FAILED))?;
    if !scopes_valid(&req.scopes, &active_scopes) {
        return Err(ERR_BADSCOPES);
    }

    let expire = SystemTime::now().add(Duration::from_secs(req.life)); // XXX cant this fail?
    let hash = hash_password(&cdb.serv.rng, req.secret);
    let granted_scopes = req.scopes.iter().copied().map(|s| s.to_owned()).collect();
    let u = user::User {
        name: req.name.to_owned(),
        hash: hash,
        expiration: expire,
        enabled: true,
        scopes: granted_scopes,
    };

    // XXX do we have to check if the user already exists?
    // I expect db insert will fail if it already exists
    // XXX translate errors better.. need to know why it failed...
    user::put_user(&cdb, u).await.or(Err(ERR_FAILED))?;
    Ok("created")
}

#[post("/user", format="json", data="<req>")]
pub async fn create_user(cdb: CachedDb<'_>, bearer: BearerToken, req: Json<CreateReq<'_>>) -> JsonRes<&'static str> {
    json_res(create_user_sr(cdb, bearer, req).await)
}

async fn create_scope_sr(cdb: CachedDb<'_>, bearer: BearerToken, req: Json<String>) -> StrRes<&'static str> {
    bearer.require_scope(&cdb, "authadmin").await?;
    scopes::put_scope(&cdb, &req.to_owned()).await.or(Err(ERR_FAILED))?;
    Ok("created")
}

#[post("/scope", format="json", data="<req>")]
pub async fn create_scope(cdb: CachedDb<'_>, bearer: BearerToken, req: Json<String>) -> JsonRes<&'static str> {
    json_res(create_scope_sr(cdb, bearer, req).await)
}

async fn clean_sr(cdb: CachedDb<'_>, bearer: BearerToken) -> StrRes<&'static str> {
    bearer.require_scope(&cdb, "authadmin").await?;
    let _ = cache::clean(&cdb).await;
    let _ = token::clean(&cdb).await;
    Ok("cleaned")
}

#[post("/clean", format="json")]
pub async fn clean(cdb: CachedDb<'_>, bearer: BearerToken) -> JsonRes<&'static str> {
    json_res(clean_sr(cdb, bearer).await)
}


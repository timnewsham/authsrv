
use argon2;
//use rocket::State;
use rocket::serde::{Serialize, Deserialize, json::Json};
use serde_json;
//use rocket::response::{Debug};
use diesel::table;
//use memcache::{FromMemcacheValue, MemcacheError};
use rocket_sync_db_pools::diesel::prelude::*;

//use crate::state::ServerState;
use crate::{Db, Cache};

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

/*
impl FromMemcacheValue for User {
    fn from_memcache_value(value: Vec<u8>, flags: u32) -> std::result::Result<Self, MemcacheError> {
        let u = 
        
    }
    
}
*/

//type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;   

type Result<T> = std::result::Result<T, String>;

fn errstr(x: impl ToString) -> String {
    x.to_string()
}

const CACHETIME: u32 = 5 * 60;

/*
 * Fetch typ_key from cache and return it if there were no cache errors
 * or parse errors.
 */
async fn cache_get(cache: &Cache, typ: &str, key: &str) -> Option<User> {
    let key2 = format!("{}_{}", typ, key);
    let s: String = cache.run(move |c| c.get(&key2)).await.ok()??;
    // XXX isnt there some more compact and performant encoding we can use
    // that supports Serialize/Deserialize?
    serde_json::from_str(&s).ok()
}

async fn cache_put(cache: &Cache, typ: &str, key: &str, x: &impl Serialize) -> Option<()>{
    let key2 = format!("{}_{}", typ, key);
    let s = serde_json::to_string(x).ok()?;
    cache.run(move |c| c.set(&key2, &s, CACHETIME)).await.ok()
}

async fn get_user(db: &Db, cache: &Cache, name: String) -> Result<User> {
    if let Some(u) = cache_get(&cache, "user", &name).await {
        println!("fetched from cache");
        return Ok(u);
    }

    let namecopy = name.clone();
    let u = db.run(move |c| users::table.filter(users::name.eq(&namecopy)).first(c)).await.map_err(errstr)?;
    println!("fetched from db");
    cache_put(&cache, "user", &name, &u).await;
    Ok(u)
}

#[post("/", format="json", data="<req>")]
pub async fn auth(db: Db, cache: Cache, req: Json<AuthReq<'_>>) -> Json<AuthResp> {
    let err = Json(AuthResp{ status: "error".to_owned(), result: None });

    // XXX to owned
    let u = match get_user(&db, &cache, req.name.to_owned()).await {
        Ok(x) => x,
        _ => return err,
    };
    if !argon2::verify_encoded(&u.hash, req.secret.as_bytes()).unwrap_or(false) {
        return err;
    }

    // XXX return failure if user account is expired
    // XXX remove any scopes that are no longer defined

    // XXX to_owned
    if !req.scopes.iter().all(|&s| u.scopes.contains(&s.to_owned())) {
        return err;
    }

    let token = "XXX"; // XXX create token
     // XXX insert token into db
    let astate = AuthState {
        token: token.to_owned(),
        scopes: req.scopes.iter().copied().map(str::to_owned).collect(),
    };
    Json(AuthResp{ 
        status: "ok".to_string(),
        result: Some(astate) 
    })
}


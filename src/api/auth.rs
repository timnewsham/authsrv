
use argon2;
//use rocket::State;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::response::{Debug};
use diesel::table;
use rocket_sync_db_pools::diesel::prelude::*;

//use crate::state::ServerState;
use crate::Db;

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

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;   

async fn get_user(db: &Db, name: String) -> Result<User> {
    let u = db.run(|c| users::table.filter(users::name.eq(name)).first(c)).await?;
    Ok(u)
}

#[post("/", format="json", data="<req>")]
pub async fn auth(db: Db, req: Json<AuthReq<'_>>) -> Json<AuthResp> {
    let err = Json(AuthResp{ status: "error".to_owned(), result: None });

    // XXX to_owned!
    let u = match get_user(&db, req.name.to_owned()).await {
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


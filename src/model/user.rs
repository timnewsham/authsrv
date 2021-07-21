
use std::sync::Arc;
use rocket::serde::{Serialize, Deserialize};
use diesel::table;
use rocket_sync_db_pools::diesel::prelude::*;

use crate::{Db, Cache, Server, Result, errstr};
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
pub struct User {
    pub name: String,
    pub hash: String,
    pub scopes: Vec<String>,
}

pub async fn get_user(db: &Db, cache: &Cache, serv: &Server, name: String) -> Result<User> {
    let key = Arc::new(format!("user_{}", name));
    if let Some(u) = cache::get(&cache, serv, key.clone()).await {
        return Ok(u);
    }

    let u = db.run(move |c| users::table.filter(users::name.eq(&name)).first(c)).await.map_err(errstr)?;
    cache::put(&cache, serv, key, &u).await;
    Ok(u)
}


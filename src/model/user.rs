
use std::sync::Arc;
use rocket::serde::{Serialize, Deserialize};
use rocket_sync_db_pools::diesel::prelude::*;
use std::time::SystemTime;

use crate::{Server, Result, errstr};
use crate::rocktypes::{Db, Cache};
use crate::cache;
use crate::model::schema::users;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[table_name="users"]
pub struct User {
    pub name: String,
    pub hash: String,
    pub expiration: SystemTime,
    //pub expiration: chrono::NaiveDateTime,
    pub enabled: bool,
    pub scopes: Vec<String>,
}

impl User {
    pub fn is_expired(&self) -> bool {
        // errors if expiration is before now
        self.expiration.duration_since(SystemTime::now()).is_err()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled && !self.is_expired()
    }
}

fn cache_key(k: &str) -> Arc<String> {
    Arc::new(format!("user_{}", k))
}

pub async fn get_user(db: &Db, cache: &Cache, serv: &Server, name: String) -> Result<User> {
    let key = cache_key(&name);
    if let Some(u) = cache::get(&cache, serv, key.clone()).await {
        return Ok(u);
    }

    let u = db.run(move |c| users::table.filter(users::name.eq(&name)).first(c)).await.map_err(errstr)?;
    cache::put(&cache, serv, key, &u).await;

    Ok(u)
}


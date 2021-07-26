
use std::sync::Arc;
use rocket::serde::{Serialize, Deserialize};
use rocket_sync_db_pools::diesel::prelude::*;
use std::time::SystemTime;

use crate::{Result, errstr};
use crate::rocktypes::CachedDb;
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

pub async fn get_user<'r>(cdb: &CachedDb<'r>, name: String) -> Result<User> {
    let key = cache_key(&name);
    if let Some(u) = cache::get(cdb, key.clone()).await {
        return Ok(u);
    }

    let u = cdb.db.run(move |c| users::table.filter(users::name.eq(&name)).first(c)).await.map_err(errstr)?;
    cache::put(cdb, key, &u).await;

    Ok(u)
}



use std::sync::Arc;
use rocket::serde::{Serialize, Deserialize};
use rocket_sync_db_pools::diesel::prelude::*;
use std::time::SystemTime;

use crate::{Result, errstr};
use crate::rocktypes::CachedDb;
use crate::cache;
use crate::model::schema::tokens;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[table_name="tokens"]
pub struct Token {
    pub token: String,
    pub username: String,
    pub expiration: SystemTime,
    pub scopes: Vec<String>,
}

impl Token {
    pub fn is_expired(&self) -> bool {
        self.seconds_left() == 0
    }

    pub fn seconds_left(&self) -> u64 {
        self.expiration.duration_since(SystemTime::now()).map(|d| d.as_secs()).unwrap_or(0)
    }
}

fn cache_key(k: &str) -> Arc<String> {
    Arc::new(format!("token_{}", k))
}

pub async fn get_token(cdb: &CachedDb<'_>, name: String) -> Result<Token> {
    let key = cache_key(&name);
    if let Some(x) = cache::get(cdb, key.clone()).await {
        return Ok(x);
    }

    let x = cdb.db.run(move |c| tokens::table.filter(tokens::token.eq(&name)).first(c)).await.map_err(errstr)?;
    cache::put(cdb, key, &x).await;

    Ok(x)
}

pub async fn put_token(cdb: &CachedDb<'_>, tok: &Token) -> Result<()> {
    let tok2 = tok.clone();
    cdb.db.run(|c| diesel::insert_into(tokens::table).values(tok2).execute(c)).await.map_err(errstr)?;
    let key = cache_key(&tok.token);
    let _ = cache::put(cdb, key, tok).await; // ignore any errors
    Ok(())
}

pub async fn clean(cdb: &CachedDb<'_>) -> Result<usize> {
    use diesel::dsl::now;
    use self::tokens::dsl::*; // XXX figure out exactly what we need
    let cnt = cdb.db.run(|c| 
        diesel::delete(self::tokens::table)
                .filter(expiration.lt(now))
                .execute(c)
            ).await.map_err(errstr)?;
    Ok(cnt)
}


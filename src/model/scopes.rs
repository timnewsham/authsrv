
use std::sync::Arc;
use rocket::serde::{Serialize, Deserialize};
use rocket_sync_db_pools::diesel::prelude::*;

use crate::{Result, errstr};
use crate::rocktypes::CachedDb;
use crate::cache;
use crate::model::schema::scopes;

// XXX do we need a separate type here?
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[table_name="scopes"]
pub struct Scope {
    pub name: String,
}

fn cache_key() -> Arc<String> {
    Arc::new("scopes".to_string())
}

pub async fn get_scopes(cdb: &CachedDb<'_>) -> Result<Vec<String>> {
    let key = cache_key();
    if let Some(u) = cache::get(cdb, key.clone()).await {
        return Ok(u);
    }

    let scopes: Vec<Scope> = cdb.db.run(move |c| scopes::table.load(c)).await.map_err(errstr)?;
    let names = scopes.into_iter().map(|sc| sc.name).collect();
    cache::put(cdb, key, &names).await;
    Ok(names)
}

pub async fn put_scope(cdb: &CachedDb<'_>, newscope: &String) -> Result<()> {
    let key = cache_key();
    cache::del(cdb, key).await;
    let scope = Scope{ name: newscope.clone() };
    cdb.db.run(move |c| diesel::insert_into(scopes::table).values(scope).execute(c)).await.map_err(errstr)?;
    Ok(())
}

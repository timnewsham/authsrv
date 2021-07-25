
use std::sync::Arc;
use rocket::serde::{Serialize, Deserialize};
use rocket_sync_db_pools::diesel::prelude::*;

use crate::{Db, Cache, Server, Result, errstr};
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

pub async fn get_scopes(db: &Db, cache: &Cache, serv: &Server) -> Result<Vec<String>> {
    let key = cache_key();
    if let Some(u) = cache::get(&cache, serv, key.clone()).await {
        return Ok(u);
    }

    let scopes: Vec<Scope> = db.run(move |c| scopes::table.load(c)).await.map_err(errstr)?;
    let names = scopes.into_iter().map(|sc| sc.name).collect();
    cache::put(&cache, serv, key, &names).await;
    Ok(names)
}


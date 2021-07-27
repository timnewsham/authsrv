
use std::sync::Arc;
use rocket::serde::{Serialize, DeserializeOwned};
use redis;
use redis::Commands;
use rmp_serde;

use crate::rocktypes::CachedDb;


/*
 * Fetch key from cache and return it if there were no cache errors
 * or parse errors.
 */
pub async fn get<T: DeserializeOwned + Send>(cdb: &CachedDb<'_>, key: Arc<String>) -> Option<T> {
    if !cdb.serv.use_cache { return None; }
    let v: Vec<u8> = cdb.cache.run(move |c| c.0.get(&*key)).await.ok()?;
    rmp_serde::from_read_ref(&v).ok()
}

pub async fn put(cdb: &CachedDb<'_>, key: Arc<String>, x: &impl Serialize) -> Option<()>{
    if !cdb.serv.use_cache { return None; }
    let v: Vec<u8> = rmp_serde::to_vec(x).ok()?;
    let lifetime = cdb.serv.cache_lifetime as usize;
    cdb.cache.run(move |c| c.0.set_ex(&*key, &*v, lifetime)).await.ok()
}

pub async fn del(cdb: &CachedDb<'_>, key: Arc<String>) -> Option<()> {
    if !cdb.serv.use_cache { return None; }
    cdb.cache.run(move |c| c.0.del(&*key)).await.ok()
}


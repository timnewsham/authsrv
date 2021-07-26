
use std::sync::Arc;
use rocket::serde::{Serialize, DeserializeOwned};
use redis;
use redis::Commands;
use rmp_serde;

use crate::Server;
use crate::rocktypes::Cache;


/*
 * Fetch key from cache and return it if there were no cache errors
 * or parse errors.
 */
pub async fn get<T: DeserializeOwned + Send>(cache: &Cache, serv: &Server, key: Arc<String>) -> Option<T> {
    if !serv.use_cache { return None; }
    let v: Vec<u8> = cache.run(move |c| c.0.get(&*key)).await.ok()?;
    rmp_serde::from_read_ref(&v).ok()
}

pub async fn put(cache: &Cache, serv: &Server, key: Arc<String>, x: &impl Serialize) -> Option<()>{
    if !serv.use_cache { return None; }
    let v: Vec<u8> = rmp_serde::to_vec(x).ok()?;
    let lifetime = serv.cache_lifetime as usize;
    cache.run(move |c| c.0.set_ex(&*key, &*v, lifetime)).await.ok()
}


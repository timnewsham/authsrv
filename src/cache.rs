
use std::sync::Arc;
use rocket::serde::{Serialize, DeserializeOwned};
use rmp_serde;                                                                  
use crate::{Cache, Server};

/*                                                                              
 * Fetch key from cache and return it if there were no cache errors             
 * or parse errors.                                                             
 */                                                                             
pub async fn get<T: DeserializeOwned>(cache: &Cache, serv: &Server, key: Arc<String>) -> Option<T> {
    if !serv.use_cache { return None; }                                         
    let s: Vec<u8> = cache.run(move |c| c.get(&key)).await.ok()??;              
    rmp_serde::from_read_ref(&s).ok()                                           
}                                                                               
                                                                                
pub async fn put(cache: &Cache, serv: &Server, key: Arc<String>, x: &impl Serialize) -> Option<()>{
    if !serv.use_cache { return None; }                                         
    let s: Vec<u8> = rmp_serde::to_vec(x).ok()?;                                
    let lifetime = serv.cache_lifetime;
    cache.run(move |c| c.set(&key, &*s, lifetime)).await.ok()                  
} 

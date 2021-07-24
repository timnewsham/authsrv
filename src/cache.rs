
use std::sync::Arc;
use rocket::serde::{Serialize, DeserializeOwned};
use crate::{Cache, Server};

/*                                                                              
 * Fetch key from cache and return it if there were no cache errors             
 * or parse errors.                                                             
 */                                                                             
pub async fn get<T: DeserializeOwned>(cache: &Cache, serv: &Server, key: Arc<String>) -> Option<T> {
    
    if !serv.use_cache { return None; }                                         
    return None;
}                                                                               
                                                                                
pub async fn put(cache: &Cache, serv: &Server, key: Arc<String>, x: &impl Serialize) -> Option<()>{
    if !serv.use_cache { return None; }                                         
    return None;
} 

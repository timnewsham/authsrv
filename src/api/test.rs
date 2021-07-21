
use argon2;
use crate::Server;

#[get("/")]
pub fn health(serv: &Server) -> String {
    format!("alive. caching is {}. cache lifetime {}\n", 
        if serv.use_cache { "enabled" } else { "disabled" },
        serv.cache_lifetime)
}

// XXX for devel. remove me!
#[get("/hash/<secret>")]
pub fn hasher(secret: &str) -> String {
    // XXX should config be global state?
    let hash_config = argon2::Config::default();
    let salt = b"randomsalt";
    let hash = argon2::hash_encoded(secret.as_bytes(), salt, &hash_config).unwrap();
    format!("Your hash is \"{}\"\n", hash)
}

// XXX for devel remove me
#[get("/crash")]
pub fn crasher() -> String {
    let x : &str = None.expect("cant get value");
    format!("Your value is \"{}\"\n", x)
}



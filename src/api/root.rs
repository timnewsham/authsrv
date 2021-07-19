
use argon2;

#[get("/")]
pub fn health() -> &'static str {
    "healthy\n"
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


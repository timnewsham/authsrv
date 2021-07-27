
use rocket_sync_db_pools::database;
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::IntoOutcome;

use crate::json::{StrRes, true_or_err, ERR_BADAUTH, ERR_EXPIRED};
use crate::model::token;
use crate::redis_support;

pub use crate::Server;

#[database("diesel")]
pub struct Db(diesel::PgConnection);

#[database("redis")]
pub struct Cache(redis_support::Connection);

/*
 * Authorization information from bearer token.
 * Implements FromRequest so routes can receive this as an argument.
 * Has methods for performing authentication.
 */
pub struct BearerToken {
    header: Option<String>,
}

#[allow(dead_code)]
impl BearerToken {
    fn new(hdr: Option<&str>) -> Self {
        BearerToken{ header: hdr.map(|s| s.to_owned()) }
    }

    // Lookup the token data associated with the bearer token and return it or an auth error
    pub async fn lookup(&self, cdb: &CachedDb<'_>) -> StrRes<token::Token> {
        let header = self.header.clone().ok_or(ERR_BADAUTH)?;
        let tok = token::get_token(cdb, header).await.or(Err(ERR_BADAUTH))?;
        let valid = !tok.is_expired();
        true_or_err(valid, tok, ERR_EXPIRED)
    }

    // Return an auth error if scope isn't associated with the bearer token
    pub async fn require_scope(&self, cdb: &CachedDb<'_>, scope: &str) -> StrRes<()> {
        let tok = self.lookup(cdb).await?;
        let valid = tok.scopes.iter().any(|have| have == scope);
        println!("require {:?}, have {:?} status {:?}", scope, tok.scopes, valid);
        true_or_err(valid, (), ERR_BADAUTH)
    }

    // Return an auth error unless the bearer token is associated with the user or the scope
    pub async fn require_user_or_scope(&self, cdb: &CachedDb<'_>, user: &str, scope: &str) -> StrRes<()> {
        let tok = self.lookup(cdb).await?;
        let valid = tok.username == user || tok.scopes.iter().any(|have| have == scope);
        true_or_err(valid, (), ERR_BADAUTH)
    }
}

// Automatically pull BearerTokens out from requests when asked for
#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<BearerToken, Self::Error> {
        let opthdr = request.headers()
                        .get_one("Authorization")
                        .and_then(|s| s.strip_prefix("bearer "));
        let bt = BearerToken::new(opthdr);
        // XXX never forwards.. is there a better way to do this?
        Some(bt).or_forward(())
    }
}

// Wraps up Cache and Db and Server, since they're all needed together
pub struct CachedDb<'r> {
    pub cache: Cache,
    pub db: Db,
    pub serv: &'r Server,
}

// Automatically provide wrapped CacheDb when asked for
#[rocket::async_trait]
impl <'r> FromRequest<'r> for CachedDb<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let cache = request.guard::<Cache>().await.expect("cant get cache pool");
        let db = request.guard::<Db>().await.expect("cant get db pool");
        let serv = request.guard::<&Server>().await.expect("cant get server state");
        Ok(CachedDb{ cache: cache, db: db, serv: serv })
            .or_forward(())
    }
}


use rocket_sync_db_pools::database;
use rocket::serde::json::Json;
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::IntoOutcome;

use crate::json::{IntoJErr, JsonError, true_or_jerr, ERR_BADAUTH, ERR_EXPIRED};
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
    pub async fn lookup(&self, db: &Db, cache: &Cache, serv: &Server) -> Result<token::Token, Json<JsonError>> {
        let header = self.header.clone().map_jerr(ERR_BADAUTH)?;
        let tok = token::get_token(db, cache, serv, header).await.map_jerr(ERR_BADAUTH)?;
        let valid = !tok.is_expired();
        true_or_jerr(valid, tok, ERR_EXPIRED)
    }

    // Return an auth error if scope isn't associated with the bearer token
    pub async fn require_scope(&self, db: &Db, cache: &Cache, serv: &Server, scope: &str) -> Result<(), Json<JsonError>> {
        let tok = self.lookup(db, cache, serv).await?;
        let valid = tok.scopes.iter().any(|have| have == scope);
        true_or_jerr(valid, (), ERR_BADAUTH)
    }

    // Return an auth error unless the bearer token is associated with the user or the scope
    pub async fn require_user_or_scope(&self, db: &Db, cache: &Cache, serv: &Server, user: &str, scope: &str) -> Result<(), Json<JsonError>> {
        let tok = self.lookup(db, cache, serv).await?;
        let valid = tok.username == user || tok.scopes.iter().any(|have| have == scope);
        true_or_jerr(valid, (), ERR_BADAUTH)
    }
}

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


use rocket_sync_db_pools::database;
use rocket::serde::json::Json;
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::IntoOutcome;
use rocket::http::Status;

use crate::json::{IntoJErr, json_err, JsonError, ERR_BADAUTH, ERR_EXPIRED};
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
    header: String,
}

#[allow(dead_code)]
impl BearerToken {
    fn new(hdr: &str) -> Self {
        BearerToken{ header: hdr.to_owned() }
    }

    // Lookup the token data associated with the bearer token and return it or an auth error
    pub async fn lookup(&self, db: &Db, cache: &Cache, serv: &Server) -> Result<token::Token, Json<JsonError>> {
        let tok = token::get_token(db, cache, serv, self.header.clone()).await.map_jerr(ERR_BADAUTH)?;
        if tok.is_expired() {
            json_err(ERR_EXPIRED)
        } else {
            Ok(tok)
        }
    }

    // Return an auth error if scope isn't associated with the bearer token
    pub async fn require_scope(&self, db: &Db, cache: &Cache, serv: &Server, scope: &str) -> Result<(), Json<JsonError>> {
        let tok = self.lookup(db, cache, serv).await?;
        if tok.scopes.iter().any(|have| have == scope) {
            Ok(())
        } else {
            json_err(ERR_BADAUTH)
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = Json<JsonError>;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<BearerToken, Self::Error> {
        request.headers()
            .get_one("Authorization")
            .and_then(|h| h.strip_prefix("bearer "))
            .map(BearerToken::new)
            // XXX how can we return 401 and have it be json?
            //.into_outcome((Status::Unauthorized, Json(JsonError::new(ERR_BADAUTH))))
            .into_outcome((Status::Ok, Json(JsonError::new(ERR_BADAUTH))))
    }
}


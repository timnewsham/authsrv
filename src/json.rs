
use rocket::serde::{Serialize, json::Json};

pub const ERR_FAILED: &'static str = "failed";
pub const ERR_BADAUTH: &'static str = "auth failure";
pub const ERR_BADSCOPES: &'static str = "bad scopes";
pub const ERR_EXPIRED: &'static str = "expired";

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct JsonError {
    status: &'static str,
    result: &'static str,
}

impl JsonError {
    pub fn new(msg: &'static str) -> Self {
        JsonError{
            status: "error",
            result: msg,
        }
    }
}

// A Json<T> result or a Json error
pub type JsonRes<T> = Result<Json<T>, Json<JsonError>>;

pub trait IntoJErr<T> {
    // map an error result into a JsonRes with the provided json error message
    fn map_jerr(self, errmsg: &'static str) -> Result<T, Json<JsonError>>;
}

pub fn json_err<T>(msg: &'static str) -> Result<T, Json<JsonError>> {
    Err(Json(JsonError::new(msg)))
}

impl <T, E: std::fmt::Debug> IntoJErr<T> for Result<T, E> {
    fn map_jerr(self, errmsg: &'static str) -> Result<T, Json<JsonError>> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                // log error XXX something better than println
                println!("got error! {:?}", e);
                json_err(errmsg)
            },
        }
    }
}

impl <T> IntoJErr<T> for Option<T> {
    fn map_jerr(self, errmsg: &'static str) -> Result<T, Json<JsonError>> {
        match self {
            Some(x) => Ok(x),
            None => json_err(errmsg),
        }
    }
}

pub fn true_or_jerr<T>(ok: bool, okval: T, errmsg: &'static str) -> Result<T, Json<JsonError>> {
    if ok {
        Ok(okval)
    } else {
        json_err(errmsg)
    }
}



use rocket::serde::{Serialize, json::Json};

pub const ERR_FAILED: &'static str = "failed";
pub const ERR_BADAUTH: &'static str = "auth failure";
pub const ERR_BADSCOPES: &'static str = "bad scopes";

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonError {
    status: &'static str,
    result: &'static str,
}

// A Json<T> result or a Json error
pub type JsonRes<T> = Result<Json<T>, Json<JsonError>>;

pub trait IntoJErr<T> {
    // map an error result into a JsonRes with the provided json error message
    fn map_jerr(self, errmsg: &'static str) -> Result<T, Json<JsonError>>;
}

pub fn json_err<T>(msg: &'static str) -> Result<T, Json<JsonError>> {
    Err(Json(JsonError {
        status: "error",
        result: msg,
    }))
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

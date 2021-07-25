
use rocket::serde::{Serialize, json::Json};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonError {
    status: &'static str,
    result: &'static str,
}

// Its either a json result for T or a json error message
pub type JsonRes<T> = Result<Json<T>, Json<JsonError>>;

pub fn json_err<T>(msg: &'static str) -> Result<T, Json<JsonError>> {
    Err(Json(JsonError {
        status: "error",
        result: msg,
    }))
}

pub const ERR_FAILED: &'static str = "failed";
pub const ERR_BADAUTH: &'static str = "auth failure";
pub const ERR_BADSCOPES: &'static str = "bad scopes";


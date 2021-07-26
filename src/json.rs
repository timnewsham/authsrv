
use rocket::serde::{Serialize, json::Json};

pub const ERR_FAILED: &'static str = "failed";
pub const ERR_BADAUTH: &'static str = "auth failure";
pub const ERR_BADSCOPES: &'static str = "bad scopes";
pub const ERR_EXPIRED: &'static str = "expired";

// A result with a status message
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct WithStatus<T: Serialize> {
    status: &'static str,
    result: T,
}

// Results wrapped up as JSON
type JsonError = Json<WithStatus<&'static str>>;
type JsonWithStatus<T> = Json<WithStatus<T>>;

// A JsonRes<T> is success or error wrapped in a Json message with a status field.
pub type JsonRes<T> = Result<JsonWithStatus<T>, JsonError>;

// A StrRes is success or an error string.
pub type StrRes<T> = Result<T, &'static str>;

// Convert a StrRes<T> into a JsonRes<T>
pub fn json_res<T: Serialize>(res: StrRes<T>) -> JsonRes<T> {
    // XXX in the future would be nice to set HTTP response codes with these, too...
    match res {
        Ok(v) => Ok(Json(WithStatus{ status: "ok", result: v, })),
        Err(msg) => Err(Json(WithStatus{ status: "error", result: msg, }))
    }
}

// XXX move elsewhere
pub fn true_or_err<T>(ok: bool, okval: T, errmsg: &'static str) -> Result<T, &'static str> {
    if ok {
        Ok(okval)
    } else {
        Err(errmsg)
    }
}


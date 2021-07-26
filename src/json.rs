
use rocket::http::Status;
use rocket::serde::{Serialize, json::Json};

pub struct StatusErr(&'static str, Status);

pub const ERR_FAILED: StatusErr = StatusErr("failed", Status::Ok);
pub const ERR_BADAUTH: StatusErr = StatusErr("auth failure", Status::Unauthorized);
pub const ERR_BADSCOPES: StatusErr = StatusErr("bad scopes", Status::Unauthorized);
pub const ERR_EXPIRED: StatusErr = StatusErr("expired", Status::Unauthorized);

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
pub type JsonRes<T> = (Status, Result<JsonWithStatus<T>, JsonError>);

// A StrRes is success or an error string and a status code.
pub type StrRes<T> = Result<T, StatusErr>;

// Convert a StrRes<T> into a JsonRes<T> with a status code
pub fn json_res<T: Serialize>(res: StrRes<T>) -> JsonRes<T> {
    match res {
        Ok(v) => 
            (Status::Ok, 
             Ok(Json(WithStatus{ status: "ok", result: v, }))),
        Err(StatusErr(msg, code)) => 
            (code, 
             Err(Json(WithStatus{ status: "error", result: msg, }))),
    }
}

// XXX move elsewhere
pub fn true_or_err<T, E>(ok: bool, okval: T, errval: E) -> Result<T, E> {
    if ok { Ok(okval) } else { Err(errval) }
}


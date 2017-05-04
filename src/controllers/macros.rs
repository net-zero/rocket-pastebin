// use rocket::http::Status;
// use rocket::response::status::Custom;
// use rocket_contrib::{JSON, Value};
// use helpers::error::Error;

macro_rules! call_ctrl {
    ($perm: expr, $ctrl_fn: expr) => (
        match $ctrl_fn($perm) {
            Ok(result) => Custom(Status::Ok, JSON(json!(result))),
            Err(err) => Custom::from(err),
        }
    )
}

macro_rules! call_serv {
    ($diesel_serv_call: expr) => (
        $diesel_serv_call.or_else(|err| Err(Error::from(err)))
    )
}

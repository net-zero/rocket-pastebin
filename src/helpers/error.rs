use std::fmt;
use std::error;
use std::convert::Into;

use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub code: u16,
    pub msg: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.msg)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl Into<Custom<JSON<Value>>> for Error {
    fn into(self) -> Custom<JSON<Value>> {
        let code = self.code;
        let status = Status::from_code(code).unwrap_or(Status::new(code, "custom code"));
        Custom(status, JSON(json!(self)))
    }
}

pub fn badrequest(msg: &str) -> Error {
    Error {
        code: Status::BadRequest.code,
        msg: msg.to_string(),
    }
}

pub fn unauthorized(msg: &str) -> Error {
    Error {
        code: Status::Unauthorized.code,
        msg: msg.to_string(),
    }
}

pub fn forbidden(msg: &str) -> Error {
    Error {
        code: Status::Forbidden.code,
        msg: msg.to_string(),
    }
}

pub fn notfound(msg: &str) -> Error {
    Error {
        code: Status::NotFound.code,
        msg: msg.to_string(),
    }
}

pub fn internal_server_error(msg: &str) -> Error {
    Error {
        code: Status::InternalServerError.code,
        msg: msg.to_string(),
    }
}

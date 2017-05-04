use std::fmt;
use std::error;
use std::convert::From;

use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use diesel::result::Error as DieselError;

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

impl From<Error> for Custom<JSON<Value>> {
    fn from(err: Error) -> Custom<JSON<Value>> {
        let status = Status::from_code(err.code).unwrap_or(Status::new(err.code, "custom code"));
        Custom(status, JSON(json!(err)))
    }
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Error {
        match err {
            DieselError::NotFound => notfound("data not found"),
            _ => internal_server_error("database operation failure"),
        }
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

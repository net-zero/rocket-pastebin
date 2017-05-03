use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::users::*;

use helpers::db::DB;
use helpers::guard::{NormalUser, AdminUser};
use helpers::error;


#[get("/users/me")]
pub fn me(normal_user: Result<NormalUser, error::Error>, db: DB) -> Custom<JSON<Value>> {
    match normal_user.and_then(|normal_user| {
                                   get_user_by_id(normal_user.user_id, db.conn())
                                       .or(Err(error::badrequest("user not found")))
                               }) {
        Ok(user) => Custom(Status::Ok, JSON(json!(user))),
        Err(err) => err.into(),
    }
}

#[get("/users")]
pub fn get_users(admin: Result<AdminUser, error::Error>, db: DB) -> Custom<JSON<Value>> {
    match admin.and_then(|_| {
        get_user_list(db.conn()).or(Err(error::internal_server_error("fail to get users")))
    }) {
        Ok(users) => Custom(Status::Ok, JSON(json!(users))),
        Err(err) => err.into(),
    }
}

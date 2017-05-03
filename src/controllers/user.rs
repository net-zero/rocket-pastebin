use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::users as user_serv;

use helpers::db::DB;
use helpers::guard::{NormalUser, AdminUser};
use helpers::error;


#[get("/users/me")]
pub fn me(normal_user: Result<NormalUser, error::Error>, db: DB) -> Custom<JSON<Value>> {
    match normal_user.and_then(|normal_user| {
                                   user_serv::get_user_by_id(normal_user.user_id, db.conn())
                                       .or(Err(error::badrequest("user not found")))
                               }) {
        Ok(user) => Custom(Status::Ok, JSON(json!(user))),
        Err(err) => err.into(),
    }
}

#[get("/users")]
pub fn get_users(admin: Result<AdminUser, error::Error>, db: DB) -> Custom<JSON<Value>> {
    match admin.and_then(|_| {
                             user_serv::get_user_list(db.conn())
                                 .or(Err(error::internal_server_error("fail to get users")))
                         }) {
        Ok(users) => Custom(Status::Ok, JSON(json!(users))),
        Err(err) => err.into(),
    }
}

#[derive(FromForm)]
pub struct UserPayload {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}


#[post("/users", data = "<payload>")]
pub fn create_user(payload: Form<UserPayload>, db: DB) -> Custom<JSON<Value>> {
    let payload = payload.into_inner();
    if payload.password != payload.confirm_password {
        return Custom(Status::BadRequest,
                      JSON(json!(error::badrequest("password mismatch"))));
    }

    // TODO: validation
    let new_user = user_serv::NewUser {
        username: &payload.username,
        email: &payload.email,
        password: &payload.password,
    };

    // TODO: detail error instead of InternalServerError, for example, duplicate username
    match user_serv::create_user(&new_user, db.conn()) {
        Ok(user) => Custom(Status::Ok, JSON(json!(user))),
        Err(_) => {
            Custom(Status::InternalServerError,
                   JSON(json!(error::internal_server_error("fail to create user"))))
        }
    }
}

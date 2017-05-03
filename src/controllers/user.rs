use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::users as user_serv;

use helpers::db::DB;
use helpers::guard::{NormalUser, AdminUser, has_permission};
use helpers::error;
use self::error::Error;


#[get("/users/me")]
pub fn me(normal_user: Result<NormalUser, Error>, db: DB) -> Custom<JSON<Value>> {
    match normal_user.and_then(|normal_user| {
                                   user_serv::get_user_by_id(normal_user.user_id, db.conn())
                                       .or(Err(error::badrequest("user not found")))
                               }) {
        Ok(user) => Custom(Status::Ok, JSON(json!(user))),
        Err(err) => err.into(),
    }
}

#[get("/users")]
pub fn get_users(admin: Result<AdminUser, Error>, db: DB) -> Custom<JSON<Value>> {
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

#[get("/users/<id>")]
pub fn get_user_by_id(id: i32,
                      user: Result<NormalUser, Error>,
                      admin: Result<AdminUser, Error>,
                      db: DB)
                      -> Custom<JSON<Value>> {
    match has_permission(id, user, admin).and_then(|_| user_serv::get_user_by_id(id, db.conn()).or(Err(error::notfound("user not found")))) {
        Ok(user) => Custom(Status::Ok, JSON(json!(user))),
        Err(err) => err.into(),
    }
}

#[derive(FromForm)]
pub struct UpdatePayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

#[put("/users/<id>", data = "<payload>")]
pub fn update_user_by_id(id: i32,
                         payload: Form<UpdatePayload>,
                         user: Result<NormalUser, Error>,
                         admin: Result<AdminUser, Error>,
                         db: DB)
                         -> Custom<JSON<Value>> {
    match has_permission(id, user, admin).and_then(|_| {
        let payload = payload.into_inner();
        if payload.password.as_ref().is_some() && ( payload.confirm_password.as_ref().is_none() || payload.confirm_password.as_ref().unwrap() != payload.password.as_ref().unwrap() ) {
            return Err(error::badrequest("password mismatch"));
        }

        let updated_user = user_serv::UpdatedUser {
            username: payload.username.as_ref().map(|name| name.as_ref()),
            email: payload.email.as_ref().map(|email| email.as_ref()),
            password: payload.password.as_ref().map(|password| password.as_ref()),
        };

        user_serv::update_user(id, &updated_user, db.conn()).or(Err(error::internal_server_error("fail to update user")))
    }) {
            Ok(user) => Custom(Status::Ok, JSON(json!(user))),
            Err(err) => err.into()
    }
}

#[delete("/users/<id>")]
pub fn delete_user_by_id(id: i32,
                         user: Result<NormalUser, Error>,
                         admin: Result<AdminUser, Error>,
                         db: DB)
                         -> Custom<JSON<Value>> {
    match has_permission(id, user, admin).and_then(|_| user_serv::delete_user(id, db.conn()).or(Err(error::internal_server_error("fail to delete user")))) {
        Ok(del_num) => Custom(Status::Ok, JSON(json!(del_num))),
        Err(err) => err.into(),
    }
}

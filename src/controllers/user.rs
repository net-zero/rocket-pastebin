use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::user as user_serv;

use helpers::db::DB;
use helpers::guard::{NormalUser, AdminUser, check_perm};
use helpers::error;
use self::error::Error;


#[get("/users/me")]
pub fn me(user: Result<NormalUser, Error>, db: DB) -> Custom<JSON<Value>> {
    call_ctrl!(user, |perm: Result<NormalUser, Error>| {
        perm.and_then(|user| call_serv!(user_serv::get_user_by_id(user.user_id, db.conn())))
    })
}

#[get("/users")]
pub fn get_users(admin: Result<AdminUser, Error>, db: DB) -> Custom<JSON<Value>> {
    call_ctrl!(admin, |perm: Result<AdminUser, Error>| {
        perm.and_then(|_| call_serv!(user_serv::get_user_list(db.conn())))
    })
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
    call_ctrl!(Ok(()), |_: Result<(), Error>| {
        let payload = payload.into_inner();
        if payload.password != payload.confirm_password {
            return Err(error::badrequest("password mismatch"));
        }

        // TODO: validation
        let new_user = user_serv::NewUser {
            username: &payload.username,
            email: &payload.email,
            password: &payload.password,
        };

        // TODO: detail error instead of InternalServerError, for example, duplicate username
        call_serv!(user_serv::create_user(&new_user, db.conn()))
    })
}

#[get("/users/<id>")]
pub fn get_user_by_id(id: i32,
                      user: Result<NormalUser, Error>,
                      admin: Result<AdminUser, Error>,
                      db: DB)
                      -> Custom<JSON<Value>> {
    call_ctrl!(check_perm(id, user, admin), |perm: Result<(), Error>| {
        perm.and_then(|_| call_serv!(user_serv::get_user_by_id(id, db.conn())))
    })
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
    call_ctrl!(check_perm(id, user, admin), |perm: Result<(), Error>| {
        perm.and_then(|_| {
            let payload = payload.into_inner();
            if payload.password.as_ref().is_some() &&
               (payload.confirm_password.as_ref().is_none() ||
                payload.confirm_password.as_ref().unwrap() != payload.password.as_ref().unwrap()) {
                return Err(error::badrequest("password mismatch"));
            }

            let updated_user = user_serv::UpdatedUser {
                username: payload.username.as_ref().map(|name| name.as_ref()),
                email: payload.email.as_ref().map(|email| email.as_ref()),
                password: payload
                    .password
                    .as_ref()
                    .map(|password| password.as_ref()),
            };

            call_serv!(user_serv::update_user(id, &updated_user, db.conn()))
        })
    })
}

#[delete("/users/<id>")]
pub fn delete_user_by_id(id: i32,
                         user: Result<NormalUser, Error>,
                         admin: Result<AdminUser, Error>,
                         db: DB)
                         -> Custom<JSON<Value>> {
    call_ctrl!(check_perm(id, user, admin), |perm: Result<(), Error>| {
        perm.and_then(|_| call_serv!(user_serv::delete_user(id, db.conn())))
    })
}

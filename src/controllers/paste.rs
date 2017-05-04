use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::paste as paste_serv;
use models::paste::{Paste, NewPaste};

use helpers::db::DB;
use helpers::guard::{NormalUser, AdminUser, has_permission};
use helpers::error;
use self::error::Error;

#[get("/pastes")]
pub fn get_pastes(admin: Result<AdminUser, Error>, db: DB) -> Custom<JSON<Value>> {
    match admin.and_then(|_| {
                             paste_serv::get_pastes(db.conn())
                                 .or(Err(error::internal_server_error("fail to get pastes")))
                         }) {
        Ok(pastes) => Custom(Status::Ok, JSON(json!(pastes))),
        Err(err) => err.into(),
    }
}

#[post("/pastes", data = "<payload>")]
pub fn create_paste(payload: Form<NewPaste>,
                    user: Result<NormalUser, Error>,
                    db: DB)
                    -> Custom<JSON<Value>> {
    match user.and_then(|user| {
        let payload = payload.into_inner();
        if user.user_id != payload.user_id {
            return Err(error::badrequest("user_id doesn't match jwt token"));
        }

        paste_serv::create_paste(&payload, db.conn()).or(Err(error::internal_server_error("fail to create paste")))
        }) {
            Ok(paste) => Custom(Status::Ok, JSON(json!(paste))),
            Err(err) => err.into(),
    }
}

#[get("/pastes/<id>")]
pub fn get_paste_by_id(id: i32, db: DB) -> Custom<JSON<Value>> {
    match paste_serv::get_paste_by_id(id, db.conn()) {
        Ok(paste) => Custom(Status::Ok, JSON(json!(paste))),
        Err(_) => {
            Custom(Status::InternalServerError,
                   JSON(json!(error::internal_server_error("fail to get paste"))))
        }
    }
}

#[get("/users/<user_id>/pastes")]
pub fn get_pastes_by_user_id(user_id: i32,
                             user: Result<NormalUser, Error>,
                             admin: Result<AdminUser, Error>,
                             db: DB)
                             -> Custom<JSON<Value>> {
    match has_permission(user_id, user, admin).and_then(|_| {
        paste_serv::get_pastes_by_user_id(user_id, db.conn()).or(Err(error::internal_server_error("fail to get pastes")))
    }) {
        Ok(pastes) => Custom(Status::Ok, JSON(json!(pastes))),
        Err(err) => err.into(),
    }
}

#[put("/users/<user_id>/pastes/<id>", data = "<payload>")]
pub fn update_paste_by_id(id: i32,
                          user_id: i32,
                          payload: Form<Paste>,
                          user: Result<NormalUser, Error>,
                          admin: Result<AdminUser, Error>,
                          db: DB)
                          -> Custom<JSON<Value>> {
    match has_permission(user_id, user, admin).and_then(|_| {
        let payload = payload.into_inner();
        if payload.user_id != user_id || payload.id != id {
            return Err(error::badrequest("user_id or paste id doesn't match"));
        }

        paste_serv::update_paste(payload, db.conn()).or(Err(error::internal_server_error("fail to update paste")))
    }) {
        Ok(paste) => Custom(Status::Ok, JSON(json!(paste))),
        Err(err) => err.into(),
    }
}

#[delete("/users/<user_id>/pastes/<id>")]
pub fn delete_paste_by_id(id: i32,
                          user_id: i32,
                          user: Result<NormalUser, Error>,
                          admin: Result<AdminUser, Error>,
                          db: DB)
                          -> Custom<JSON<Value>> {
    match has_permission(user_id, user, admin).and_then(|_| {
        paste_serv::delete_paste(id, db.conn()).or(Err(error::internal_server_error("fail to delete paste")))
    }) {
        Ok(del_num) => Custom(Status::Ok, JSON(json!(del_num))),
        Err(err) => err.into(),
    }
}

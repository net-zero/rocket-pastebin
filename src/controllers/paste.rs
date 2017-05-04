use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::paste as paste_serv;
use models::paste::{Paste, NewPaste};

use helpers::db::DB;
use helpers::guard::{NormalUser, AdminUser, check_perm};
use helpers::error;
use self::error::Error;

#[get("/pastes")]
pub fn get_pastes(admin: Result<AdminUser, Error>, db: DB) -> Custom<JSON<Value>> {
    call_ctrl!(admin, |perm: Result<AdminUser, Error>| {
        perm.and_then(|_| call_serv!(paste_serv::get_pastes(db.conn())))
    })
}

#[post("/pastes", data = "<payload>")]
pub fn create_paste(payload: Form<NewPaste>,
                    user: Result<NormalUser, Error>,
                    db: DB)
                    -> Custom<JSON<Value>> {
    call_ctrl!(user, |perm: Result<NormalUser, Error>| {
        perm.and_then(|user| {
                          let payload = payload.into_inner();
                          if user.user_id != payload.user_id {
                              return Err(error::badrequest("user_id doesn't match jwt token"));
                          }

                          call_serv!(paste_serv::create_paste(&payload, db.conn()))
                      })
    })
}

#[get("/pastes/<id>")]
pub fn get_paste_by_id(id: i32, db: DB) -> Custom<JSON<Value>> {
    call_ctrl!(Ok(()),
               |_: Result<(), Error>| call_serv!(paste_serv::get_paste_by_id(id, db.conn())))
}

#[get("/users/<user_id>/pastes")]
pub fn get_pastes_by_user_id(user_id: i32,
                             user: Result<NormalUser, Error>,
                             admin: Result<AdminUser, Error>,
                             db: DB)
                             -> Custom<JSON<Value>> {
    call_ctrl!(check_perm(user_id, user, admin),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| {
                                     call_serv!(paste_serv::get_pastes_by_user_id(user_id,
                                                                                  db.conn()))
                                 })
               })
}

#[put("/users/<user_id>/pastes/<id>", data = "<payload>")]
pub fn update_paste_by_id(id: i32,
                          user_id: i32,
                          payload: Form<Paste>,
                          user: Result<NormalUser, Error>,
                          admin: Result<AdminUser, Error>,
                          db: DB)
                          -> Custom<JSON<Value>> {
    call_ctrl!(check_perm(user_id, user, admin),
               |perm: Result<(), Error>| {
        perm.and_then(|_| {
                          let payload = payload.into_inner();
                          if payload.user_id != user_id || payload.id != id {
                              return Err(error::badrequest("user_id or paste id doesn't match"));
                          }

                          call_serv!(paste_serv::update_paste(payload, db.conn()))
                      })
    })
}

#[delete("/users/<user_id>/pastes/<id>")]
pub fn delete_paste_by_id(id: i32,
                          user_id: i32,
                          user: Result<NormalUser, Error>,
                          admin: Result<AdminUser, Error>,
                          db: DB)
                          -> Custom<JSON<Value>> {
    call_ctrl!(check_perm(user_id, user, admin),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| call_serv!(paste_serv::delete_paste(id, db.conn())))
               })
}

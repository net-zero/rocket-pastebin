use rocket::State;
use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::paste as paste_serv;
use models::paste::{Paste, NewPaste};

use DBPool;

use helpers::guard::{UserToken, User, Admin};
use helpers::error;
use self::error::Error;

#[get("/pastes")]
pub fn get_pastes(token: Result<UserToken<Admin>, Error>,
                  db_pool: State<DBPool>)
                  -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm()),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| get_conn!(db_pool))
                       .and_then(|conn| call_serv!(paste_serv::get_pastes(&conn)))
               })
}

#[post("/pastes", data = "<payload>")]
pub fn create_paste(payload: Form<NewPaste>,
                    user: Result<UserToken<User>, Error>,
                    db_pool: State<DBPool>)
                    -> Custom<JSON<Value>> {
    call_ctrl!(user, |perm: Result<UserToken<User>, Error>| {
        perm.and_then(|user| {
                          let payload = payload.into_inner();
                          if user.user_id != payload.user_id {
                              return Err(error::badrequest("user_id doesn't match jwt token"));
                          }

                          get_conn!(db_pool).and_then(|conn| call_serv!(paste_serv::create_paste(&payload, &conn)))
                      })
    })
}

#[get("/pastes/<id>")]
pub fn get_paste_by_id(id: i32, db_pool: State<DBPool>) -> Custom<JSON<Value>> {
    call_ctrl!(Ok(()), |_: Result<(), Error>| {
        get_conn!(db_pool).and_then(|conn| call_serv!(paste_serv::get_paste_by_id(id, &conn)))
    })
}

#[get("/users/<user_id>/pastes")]
pub fn get_pastes_by_user_id(user_id: i32,
                             token: Result<UserToken<User>, Error>,
                             db_pool: State<DBPool>)
                             -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm(user_id)),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| {
                                     get_conn!(db_pool).and_then(|conn|
                                     call_serv!(paste_serv::get_pastes_by_user_id(user_id, &conn)))
                                 })
               })
}

#[put("/users/<user_id>/pastes/<id>", data = "<payload>")]
pub fn update_paste_by_id(id: i32,
                          user_id: i32,
                          payload: Form<Paste>,
                          token: Result<UserToken<User>, Error>,
                          db_pool: State<DBPool>)
                          -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm(user_id)),
               |perm: Result<(), Error>| {
        perm.and_then(|_| {
                          let payload = payload.into_inner();
                          if payload.user_id != user_id || payload.id != id {
                              return Err(error::badrequest("user_id or paste id doesn't match"));
                          }

                          get_conn!(db_pool).and_then(|conn| call_serv!(paste_serv::update_paste(payload, &conn)))
                      })
    })
}

#[delete("/users/<user_id>/pastes/<id>")]
pub fn delete_paste_by_id(id: i32,
                          user_id: i32,
                          token: Result<UserToken<User>, Error>,
                          db_pool: State<DBPool>)
                          -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm(user_id)),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| {
                                     get_conn!(db_pool).and_then(|conn|
                                 call_serv!(paste_serv::delete_paste(id, &conn)))
                                 })
               })
}

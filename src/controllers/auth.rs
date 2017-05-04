use rocket::State;
use rocket::request::Form;
use rocket::response::status::Custom;
use rocket::http::Status;
use rocket_contrib::{JSON, Value};

use DBPool;

use services::user as user_serv;
use services::auth;
use helpers::error;
use helpers::error::Error;

#[derive(FromForm)]
pub struct LoginPayload {
    username: String,
    password: String,
}

#[post("/login", data = "<payload>")]
pub fn login(payload: Form<LoginPayload>, db_pool: State<DBPool>) -> Custom<JSON<Value>> {
    let user_error = error::badrequest("wrong username or password");
    let jwt_error = error::internal_server_error("fail to generate jwt token");
    let payload = payload.into_inner();

    call_ctrl!(Ok(()), |_: Result<(), Error>| {
        get_conn!(db_pool)
            .and_then(|conn| call_serv!(user_serv::get_user_by_name(&payload.username, &conn)))
            .or_else(|err| {
                         if err.code == Status::NotFound.code {
                             return Err(user_error.clone());
                         }
                         Err(err)
                     })
            .and_then(|user| {
                          if !user.verify_password(&payload.password) {
                              return Err(user_error.clone());
                          }
                          Ok(user)
                      })
            .and_then(|user| {
                          auth::login(&user)
                              .or_else(|_| Err(jwt_error))
                              .and_then(|token| Ok(format!("token: {}", token)))
                      })
    })
}

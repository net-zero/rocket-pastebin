use rocket::request::Form;
use rocket::response::status;
use rocket::http::Status;
use rocket_contrib::{JSON, Value};

use services::users;
use services::auth;
use helpers::db::DB;
use helpers::error;

#[derive(FromForm)]
pub struct LoginPayload {
    username: String,
    password: String,
}

#[post("/login", data = "<payload>")]
pub fn login(payload: Form<LoginPayload>, db: DB) -> status::Custom<JSON<Value>> {
    let user_error = error::badrequest("wrong username or password");
    let jwt_error = error::internal_server_error("fail to generate jwt token");
    let username = &payload.get().username;
    let password = &payload.get().password;

    let result = users::get_user_by_name(username, db.conn())
        .or(Err(&user_error))
        .and_then(|user| match user.verify_password(password) {
                      true => Ok(user),
                      false => Err(&user_error),
                  })
        .and_then(|user| auth::login(&user).or(Err(&jwt_error)));

    match result {
        Ok(token) => {
            let res = JSON(json!({
                                     "token": token
                                 }));
            status::Custom(Status::Ok, res)
        }
        Err(err) => status::Custom(Status::BadRequest, JSON(json!(err))),
    }
}

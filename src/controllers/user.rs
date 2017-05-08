use rocket::State;
use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use DBPool;

use services::user as user_serv;

use helpers::guard::{User, Admin, UserToken};
use helpers::error;
use self::error::Error;


#[get("/users/me")]
pub fn me(token: Result<UserToken<User>, Error>, db_pool: State<DBPool>) -> Custom<JSON<Value>> {
    call_ctrl!(token, |perm: Result<UserToken<User>, Error>| {
        perm.and_then(|user| get_conn!(db_pool).and_then(|conn| Ok((user, conn))))
            .and_then(|(user, conn)| call_serv!(user_serv::get_user_by_id(user.user_id, &conn)))
    })
}

#[get("/users")]
pub fn get_users(token: Result<UserToken<Admin>, Error>,
                 db_pool: State<DBPool>)
                 -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm()),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| get_conn!(db_pool))
                       .and_then(|conn| call_serv!(user_serv::get_user_list(&conn)))
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
pub fn create_user(payload: Form<UserPayload>, db_pool: State<DBPool>) -> Custom<JSON<Value>> {
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

        get_conn!(db_pool).and_then(|conn| call_serv!(user_serv::create_user(&new_user, &conn)))
    })
}

#[get("/users/<id>")]
pub fn get_user_by_id(id: i32,
                      token: Result<UserToken<User>, Error>,
                      db_pool: State<DBPool>)
                      -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm(id)),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| get_conn!(db_pool))
                       .and_then(|conn| call_serv!(user_serv::get_user_by_id(id, &conn)))
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
                         token: Result<UserToken<User>, Error>,
                         db_pool: State<DBPool>)
                         -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm(id)),
               |perm: Result<(), Error>| {
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

            get_conn!(db_pool).and_then(|conn| {
                                            call_serv!(user_serv::update_user(id,
                                                                              &updated_user,
                                                                              &conn))
                                        })
        })
    })
}

#[delete("/users/<id>")]
pub fn delete_user_by_id(id: i32,
                         token: Result<UserToken<User>, Error>,
                         db_pool: State<DBPool>)
                         -> Custom<JSON<Value>> {
    call_ctrl!(token.and_then(|token| token.has_perm(id)),
               |perm: Result<(), Error>| {
                   perm.and_then(|_| get_conn!(db_pool))
                       .and_then(|conn| call_serv!(user_serv::delete_user(id, &conn)))
               })
}

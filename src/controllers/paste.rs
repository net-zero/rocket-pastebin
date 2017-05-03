use rocket::request::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::{JSON, Value};

use services::pastes as paste_serv;
use models::pastes::NewPaste;

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

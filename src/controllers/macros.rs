// use rocket::http::Status;
// use rocket::response::status::Custom;
// use rocket_contrib::{JSON, Value};
// use helpers::error::Error;

macro_rules! call_ctrl {
    ($ctrl_fn: expr) => (
        match $ctrl_fn() {
            Ok(result) => Custom(Status::Ok, JSON(json!(result))),
            Err(err) => Custom::from(err),
        }
    )
}

macro_rules! call_serv {
    ($diesel_serv_call: expr) => (
        $diesel_serv_call.or_else(|err| Err(Error::from(err)))
    )
}

macro_rules! match_or_has_roles {
    ($token: expr, $user_id: expr, $roles: expr) => ({
        $token.and_then(|token| {
            if !token.match_user_id($user_id) && $roles.into_iter().filter(|role| token.has_role(role)).count() == 0 {
                return Err(error::forbidden("permission denied"));
            }
            Ok(())
        })
    })
}

use std::env;
use jwt::{encode, Header};
use jwt::errors;
use super::models::users::User;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    user_id: i32,
    username: String,
    admin: bool,
}

pub fn login(user: &User) -> Result<String, errors::Error> {
    let claims = Claims {
        user_id: user.id,
        username: user.username.clone(),
        admin: user.admin,
    };
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    encode(&Header::default(), &claims, jwt_secret.as_bytes())
}

use jwt::{encode, Header};
use jwt::errors;
use super::models::users::User;

use super::super::ENV;

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
    let jwt_secret: &str = ENV.jwt_secret.as_ref();

    encode(&Header::default(), &claims, jwt_secret.as_bytes())
}

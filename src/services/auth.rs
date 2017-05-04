use jwt::{encode, Header};
use jwt::errors;

use services::user::User;
use ENV;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: i32,
    pub username: String,
    pub admin: bool,
}

pub fn login(user: &User) -> Result<String, errors::Error> {
    let claims = JwtClaims {
        user_id: user.id,
        username: user.username.clone(),
        admin: user.admin,
    };
    let jwt_secret: &str = ENV.jwt_secret.as_ref();

    encode(&Header::default(), &claims, jwt_secret.as_bytes())
}

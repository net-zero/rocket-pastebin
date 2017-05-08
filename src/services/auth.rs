use jwt::{encode, Header};
use jwt::errors;

use time;

use services::user::User;
use ENV;

const DAY: i64 = 60 * 60 * 24;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    pub user_id: i32,
    pub username: String,
    pub roles: Vec<String>,
}

impl JwtClaims {
    pub fn is_expired(&self) -> bool {
        let now = time::get_time().sec;
        now >= self.exp
    }
}

pub fn login(user: &User) -> Result<String, errors::Error> {
    let now = time::get_time().sec;
    let claims = JwtClaims {
        iat: now,
        exp: now + 7 * DAY,
        user_id: user.id,
        username: user.username.clone(),
        roles: user.roles.clone(),
    };
    let jwt_secret: &str = ENV.jwt_secret.as_ref();

    encode(&Header::default(), &claims, jwt_secret.as_bytes())
}

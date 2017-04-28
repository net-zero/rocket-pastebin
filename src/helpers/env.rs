use dotenv::dotenv;
use std::env;

pub struct Env {
    pub database_url: String,
    pub digest_salt: String,
    pub jwt_secret: String,
}

pub fn load() -> Env {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let digest_salt = env::var("DIGEST_SALT").expect("DIGEST_SALT must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    Env {
        database_url,
        digest_salt,
        jwt_secret,
    }
}

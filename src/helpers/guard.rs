use std::marker::PhantomData;

use rocket::Request;
use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::{Success, Failure};
use rocket::http::Status;

use jwt::{decode, Validation};

use ENV;
use services::auth::JwtClaims;

use helpers::error;

macro_rules! get_claims {
    ($req: expr) => (
        $req.headers()
        .get_one("Authorization")
        .ok_or((Status::Unauthorized, error::unauthorized("token not found")))
        .and_then(|bearer_token| {
            let mut validation = Validation::default();
            // relax 'exp' validation by 10 seconds, so we can use 'exp' in past
            // to test code.
            if ENV.test_expired_token {
                validation.leeway = 1000 * 10;
            }
            decode::<JwtClaims>(&bearer_token.trim_left_matches("Bearer "),
                                ENV.jwt_secret.as_ref(),
                                &validation)
                .or(Err((Status::Unauthorized, error::unauthorized("invalid token"))))
                .and_then(|data| {
                    if data.claims.is_expired() {
                        return Err((Status::Unauthorized, error::unauthorized("expired token")));
                    }
                    Ok(data.claims)
                })
        });
    )
}

pub enum User {}
pub enum Admin {}

pub struct UserToken<Perm> {
    pub user_id: i32,
    pub username: String,
    admin_flag: bool,

    perm: PhantomData<Perm>,
}

impl<'a, 'r, Perm> FromRequest<'a, 'r> for UserToken<Perm> {
    type Error = error::Error;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match get_claims!(req) {
            Ok(claims) => {
                Success(UserToken {
                            user_id: claims.user_id,
                            username: claims.username,
                            admin_flag: claims.admin,
                            perm: PhantomData,
                        })
            }
            Err(err) => Failure(err),
        }
    }
}

impl UserToken<Admin> {
    pub fn has_perm(&self) -> Result<(), error::Error> {
        if !self.admin_flag {
            return Err(error::forbidden("permission denied"));
        }
        Ok(())
    }
}

impl UserToken<User> {
    pub fn has_perm(&self, user_id: i32) -> Result<(), error::Error> {
        if self.user_id != user_id && !self.admin_flag {
            return Err(error::forbidden("permission denied"));
        }
        Ok(())
    }
}

use std::marker::PhantomData;

use rocket::Request;
use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::{Success, Failure};
use rocket::http::Status;

use jwt::{decode, Validation};

use ENV;
use services::auth::JwtClaims;

use helpers::error;
use self::error::Error;

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
    roles: Vec<String>,

    perm: PhantomData<Perm>,
}

impl<'a, 'r> FromRequest<'a, 'r> for UserToken<User> {
    type Error = Error;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match get_claims!(req) {
            Ok(claims) => {
                Success(UserToken {
                            user_id: claims.user_id,
                            username: claims.username,
                            roles: claims.roles,
                            perm: PhantomData,
                        })
            }
            Err(err) => Failure(err),
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserToken<Admin> {
    type Error = Error;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match get_claims!(req) {
            Ok(claims) => {
                if !claims.roles.contains(&"admin".to_owned()) {
                    return Failure((Status::Forbidden, error::forbidden("permission denied")));
                }

                Success(UserToken {
                            user_id: claims.user_id,
                            username: claims.username,
                            roles: claims.roles,
                            perm: PhantomData,
                        })
            }
            Err(err) => Failure(err),
        }
    }
}

impl<Perm> UserToken<Perm> {
    pub fn match_user_id(&self, id: i32) -> bool {
        self.user_id == id
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_owned())
    }
}

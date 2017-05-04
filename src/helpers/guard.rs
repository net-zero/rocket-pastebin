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
            decode::<JwtClaims>(&bearer_token.trim_left_matches("Bearer "),
                                ENV.jwt_secret.as_ref(),
                                &Validation::default())
                .or(Err((Status::Unauthorized, error::unauthorized("invalid token"))))
                .and_then(|data| Ok(data.claims))
        });
    )
}

pub struct NormalUser {
    pub user_id: i32,
}

pub struct AdminUser {
    pub user_id: i32,
    pub username: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for NormalUser {
    type Error = error::Error;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match get_claims!(req) {
            Ok(claims) => Success(NormalUser { user_id: claims.user_id }),
            Err(err) => Failure(err),
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AdminUser {
    type Error = error::Error;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match get_claims!(req) {
            Ok(claims) => {
                if claims.admin {
                    return Success(AdminUser {
                                       user_id: claims.user_id,
                                       username: claims.username,
                                   });
                }
                Failure((Status::Forbidden, error::forbidden("permission denied")))
            }
            Err(err) => Failure(err),
        }
    }
}

// return true when user_id match user.user_id or admin
pub fn check_perm(user_id: i32,
                  user: Result<NormalUser, error::Error>,
                  admin: Result<AdminUser, error::Error>)
                  -> Result<(), error::Error> {

    // if user_id doesn't match, we also return Forbidden with
    // "permission denied", this is same as AdminUser.
    if user.is_ok() && user.unwrap().user_id == user_id {
        return Ok(());
    }
    admin.and(Ok(()))
}

use ring::pbkdf2;
use std::env;
use std::convert::From;

static PBKDF2_PRF: &'static pbkdf2::PRF = &pbkdf2::HMAC_SHA256;
static PBKDF2_ITER: u32 = 1000;
const DIGEST_CREDENTIAL_LEN: usize = 32;

pub fn digest_password(username: &str, password: &str) -> Vec<u8> {
    let salt = salt(username);
    let mut credential = [0u8; DIGEST_CREDENTIAL_LEN];
    pbkdf2::derive(PBKDF2_PRF,
                   PBKDF2_ITER,
                   &salt,
                   password.as_bytes(),
                   &mut credential);
    Vec::from(&credential as &[u8])
}

pub fn verify_password(username: &str,
                       stored_password: &Vec<u8>,
                       attempted_password: &str)
                       -> bool {
    let salt = salt(username);
    pbkdf2::verify(PBKDF2_PRF,
                   PBKDF2_ITER,
                   &salt,
                   attempted_password.as_bytes(),
                   stored_password)
            .is_ok()
}

fn salt(username: &str) -> Vec<u8> {
    let digest_salt = env::var("DIGEST_SALT").expect("DIGEST_SALT must be set");
    let mut salt = Vec::with_capacity(digest_salt.as_bytes().len() + username.as_bytes().len());
    salt.extend(digest_salt.as_bytes().as_ref());
    salt.extend(username.as_bytes());
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;


    #[test]
    fn password() {
        dotenv().ok();

        const USERNAME: &str = "test";
        const PASSWORD: &str = "testpassword";
        const WRONG_PASSWORD: &str = "wrongpassword";

        let credential = digest_password(&USERNAME, &PASSWORD);
        assert_eq!(verify_password(&USERNAME, &credential, &PASSWORD), true);
        assert_eq!(verify_password(&USERNAME, &credential, &WRONG_PASSWORD),
                   false);
    }
}

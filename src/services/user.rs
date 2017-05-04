use std::convert::From;

use diesel;
use diesel::result;
// Must have this, otherwise .get_result is not available.
use diesel::prelude::*;
use diesel::pg::PgConnection;

use helpers::digest;
use models::schema;
use models::user::{User as ModelUser, NewUser as ModelNewUser};

use self::schema::users;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub admin: bool,
    #[serde(skip_serializing, skip_deserializing)]
    password_digest: Vec<u8>,
}

impl From<ModelUser> for User {
    fn from(user: ModelUser) -> User {
        User {
            id: user.id,
            username: user.username,
            email: user.email,
            admin: user.admin,
            password_digest: user.password_digest,
        }
    }
}

impl User {
    pub fn verify_password(&self, attempted_password: &str) -> bool {
        digest::verify_password(&self.username, &self.password_digest, attempted_password)
    }
}

pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

pub struct UpdatedUser<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password: Option<&'a str>,
}

pub fn create_user<'a>(user: &'a NewUser, conn: &'a PgConnection) -> Result<User, result::Error> {
    let new_user = ModelNewUser {
        username: user.username,
        email: &user.email.to_lowercase(),
        password_digest: digest::digest_password(user.username, user.password),
    };

    diesel::insert(&new_user)
        .into(users::table)
        .get_result::<ModelUser>(conn)
        .and_then(|user| Ok(user.into()))
}

pub fn update_user<'a>(id: i32,
                       updated_user: &'a UpdatedUser,
                       conn: &'a PgConnection)
                       -> Result<User, result::Error> {
    let mut user = users::table.find(id).get_result::<ModelUser>(conn)?;

    if updated_user.username.is_some() {
        user.username = updated_user.username.unwrap().into();
    }
    if updated_user.email.is_some() {
        user.email = updated_user.email.unwrap().to_lowercase();
    }
    if updated_user.password.is_some() {
        user.password_digest = digest::digest_password(user.username.as_ref(),
                                                       updated_user.password.unwrap());
    }

    diesel::update(users::table.find(id))
        .set((users::username.eq(user.username),
              users::email.eq(user.email),
              users::password_digest.eq(user.password_digest)))
        .get_result::<ModelUser>(conn)
        .and_then(|user| Ok(user.into()))
}

pub fn get_user_by_id(id: i32, conn: &PgConnection) -> Result<User, result::Error> {
    users::table
        .find(id)
        .get_result::<ModelUser>(conn)
        .and_then(|user| Ok(user.into()))
}

pub fn get_user_by_name(username: &str, conn: &PgConnection) -> Result<User, result::Error> {
    users::table
        .filter(users::username.eq(username))
        .get_result::<ModelUser>(conn)
        .and_then(|user| Ok(user.into()))
}

// TODO: paging
pub fn get_user_list(conn: &PgConnection) -> Result<Vec<User>, result::Error> {
    users::table
        .limit(20)
        .load::<ModelUser>(conn)
        .and_then(|users| Ok(users.into_iter().map(|user| user.into()).collect()))
}

pub fn delete_user(id: i32, conn: &PgConnection) -> Result<usize, result::Error> {
    diesel::delete(users::table.filter(users::id.eq(id))).execute(conn)
}

// NOTE: cannot run tests concurrently
// env RUST_TEST_THREADS=1 cargo test
#[cfg(test)]
mod tests {
    use super::*;
    use diesel::pg::PgConnection;
    use r2d2::Pool;
    use r2d2_diesel::ConnectionManager;

    use DB_POOL;
    use tests::helpers::testdata;

    #[test]
    fn test_create_user() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let mut new_user = NewUser {
            username: "test",
            email: "TEST@example.com",
            password: "password",
        };

        testdata::clear();
        let user = create_user(&new_user, conn).unwrap();

        assert_eq!(user.username, new_user.username);
        assert_eq!(user.email, new_user.email.to_lowercase().as_ref());
        // admin flag should be false by default
        assert_eq!(user.admin, false);
        assert_eq!(user.verify_password(new_user.password), true);

        // test duplicate email
        new_user.username = "test1";
        assert_eq!(create_user(&new_user, conn).is_err(), true);

        // test duplicate username
        new_user.username = "test";
        new_user.email = "test1@example.com";
        assert_eq!(create_user(&new_user, conn).is_err(), true);
    }

    #[test]
    fn test_update_user() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let mut updated_user = UpdatedUser {
            username: Some("test22"),
            email: Some("TEST22@example.com"),
            password: Some("password22"),
        };
        let duplicate_user = NewUser {
            username: "test222",
            email: "test222@example.com",
            password: "password222",
        };

        let user_id = testdata::recreate().user.id;

        // username, case-sensitive email and password update
        let user = update_user(user_id, &updated_user, conn).unwrap();
        assert_eq!(user.username, updated_user.username.unwrap());
        assert_eq!(user.email, updated_user.email.unwrap().to_lowercase());
        assert_eq!(user.verify_password(updated_user.password.unwrap()), true);

        // create user for duplicate username and email test
        assert_eq!(create_user(&duplicate_user, conn).is_ok(), true);

        // duplicate username update
        updated_user.username = Some(duplicate_user.username);
        assert_eq!(update_user(user_id, &updated_user, conn).is_err(), true);

        // duplicate email update
        updated_user.username = Some("NotDuplicateName");
        updated_user.email = Some(duplicate_user.email);
        assert_eq!(update_user(user_id, &updated_user, conn).is_err(), true);
    }

    #[test]
    fn test_get_user_by_id() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();

        let user = testdata::recreate().user;
        let fetched_user = get_user_by_id(user.id, conn).unwrap();

        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.email, user.email);
        assert_eq!(fetched_user.verify_password(testdata::TEST_USER.password),
                   true);
    }

    #[test]
    fn test_get_user_by_name() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();

        let user = testdata::recreate().user;
        let fetched_user = get_user_by_name(user.username.as_ref(), conn).unwrap();

        assert_eq!(fetched_user.id, user.id);
        assert_eq!(fetched_user.email, user.email);
        assert_eq!(fetched_user.verify_password(testdata::TEST_USER.password),
                   true);
    }

    #[test]
    fn test_delete_user() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        // test user from test data is bound to test paste, it
        // cannot be delete without delete paste first
        let new_user = NewUser {
            username: "test2",
            email: "TEST2@example.com",
            password: "password2",
        };

        testdata::clear();
        let user_id = create_user(&new_user, conn).unwrap().id;
        assert_eq!(delete_user(user_id, conn), Ok(1));
    }
}

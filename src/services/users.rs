use diesel;
use diesel::result;
// Must have this, otherwise .get_result is not available.
use diesel::prelude::*;
use diesel::pg::PgConnection;

use super::helpers::digest;
use super::models::schema;
use super::models::users::*;

use self::schema::users;

pub fn create_user<'a>(user: &'a RawUser, conn: &'a PgConnection) -> Result<User, result::Error> {
    let new_user = NewUser {
        username: user.username,
        email: &user.email.to_lowercase(),
        password_digest: digest::digest_password(user.username, user.password),
    };

    diesel::insert(&new_user)
        .into(users::table)
        .get_result(conn)
}

pub fn update_user<'a>(id: i32,
                       updated_user: &'a RawUpdatedUser,
                       conn: &'a PgConnection)
                       -> Result<User, result::Error> {
    let mut user = users::table.find(id).get_result::<User>(conn)?;

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
        .get_result(conn)
}

pub fn get_user_by_id(id: i32, conn: &PgConnection) -> Result<User, result::Error> {
    users::table.find(id).get_result::<User>(conn)
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

    use super::super::helpers;
    use super::super::testdata;

    lazy_static! {
        pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = helpers::db::create_db_pool();
    }

    #[test]
    fn test_create_user() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let mut new_user = RawUser {
            username: "test",
            email: "TEST@example.com",
            password: "password",
        };

        testdata::clear(conn);
        let user = create_user(&new_user, conn).unwrap();

        assert_eq!(user.username, new_user.username);
        assert_eq!(user.email, new_user.email.to_lowercase().as_ref());
        // admin flag should be false by default
        assert_eq!(user.admin, false);

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
        let mut updated_user = RawUpdatedUser {
            username: Some("test22"),
            email: Some("TEST22@example.com"),
            password: Some("password22"),
        };
        let duplicate_user = RawUser {
            username: "test222",
            email: "test222@example.com",
            password: "password222",
        };

        let user_id = testdata::recreate(conn).user.id;

        // username, case-sensitive email and password update
        let user = update_user(user_id, &updated_user, conn).unwrap();
        assert_eq!(user.username, updated_user.username.unwrap());
        assert_eq!(user.email, updated_user.email.unwrap().to_lowercase());
        assert_eq!(user.password_digest,
                   digest::digest_password(user.username.as_ref(), updated_user.password.unwrap()));

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

        let user = testdata::recreate(conn).user;
        let fetched_user = get_user_by_id(user.id, conn).unwrap();

        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.email, user.email);
        assert_eq!(fetched_user.password_digest, user.password_digest);
    }

    #[test]
    fn test_delete_user() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        // test user from test data is bound to test paste, it
        // cannot be delete without delete paste first
        let new_user = RawUser {
            username: "test2",
            email: "TEST2@example.com",
            password: "password2",
        };

        testdata::clear(conn);
        let user_id = create_user(&new_user, conn).unwrap().id;
        assert_eq!(delete_user(user_id, conn), Ok(1));
    }
}

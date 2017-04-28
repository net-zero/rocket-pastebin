use super::helpers;
use super::models;

pub mod users;
pub mod pastes;

pub mod testdata {
    use diesel;
    use diesel::prelude::*;
    use diesel::pg::PgConnection;

    // services
    use super::users::*;
    use super::pastes::*;

    // models
    use super::models::users::*;
    use super::models::pastes::*;
    use super::models::schema::*;

    pub const test_user: RawUser = RawUser {
        username: "test",
        email: "test@example.com",
        password: "password",
    };
    pub const test_paste_data: &str = "test paste data";

    pub struct Data {
        pub user: User,
        pub paste: Paste,
    }

    pub fn create(conn: &PgConnection) -> Data {
        let user = create_user(&test_user, conn).expect("Fail to create test user");
        let test_paste = NewPaste {
            user_id: user.id,
            data: test_paste_data.to_string(),
        };
        let paste = create_paste(&test_paste, conn).expect("Fail to create test paste");
        Data { user, paste }
    }

    pub fn clear(conn: &PgConnection) {
        // Paste has user_id, should be deleted first
        diesel::delete(pastes::table)
            .execute(conn)
            .expect("Fail to clear pastes table");
        diesel::delete(users::table)
            .execute(conn)
            .expect("Fail to clear users table");
    }

    pub fn recreate(conn: &PgConnection) -> Data {
        clear(conn);
        create(conn)
    }
}

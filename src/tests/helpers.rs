macro_rules! run_test {
    ($rocket: expr, $req: expr, $test_fn: expr) => ({
        let mut req = $req;
        $test_fn(req.dispatch_with($rocket));
    })
}

pub mod testdata {
    use diesel;
    use diesel::prelude::*;
    use diesel::pg::PgConnection;

    // services
    use services::users::*;
    use services::pastes::*;

    // models
    use models::pastes::*;
    use models::schema::*;

    use DB_POOL;

    pub const test_user: NewUser = NewUser {
        username: "test",
        email: "test@example.com",
        password: "password",
    };
    pub const test_paste_data: &str = "test paste data";

    pub struct Data {
        pub user: User,
        pub paste: Paste,
    }

    pub fn create() -> Data {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let user = create_user(&test_user, conn).expect("Fail to create test user");
        let test_paste = NewPaste {
            user_id: user.id,
            data: test_paste_data.to_string(),
        };
        let paste = create_paste(&test_paste, conn).expect("Fail to create test paste");
        Data { user, paste }
    }

    pub fn clear() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        // Paste has user_id, should be deleted first
        diesel::delete(pastes::table)
            .execute(conn)
            .expect("Fail to clear pastes table");
        diesel::delete(users::table)
            .execute(conn)
            .expect("Fail to clear users table");
    }

    pub fn recreate() -> Data {
        clear();
        create()
    }
}

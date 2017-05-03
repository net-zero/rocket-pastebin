macro_rules! run_test {
    ($rocket: expr, $req: expr, $test_fn: expr) => ({
        let mut req = $req;
        $test_fn(req.dispatch_with($rocket));
    })
}

macro_rules! body_string {
    ($response: expr) => (
        $response.body().unwrap().into_string().unwrap();
    )
}

pub mod testdata {
    use diesel;
    use diesel::prelude::*;
    use diesel::pg::PgConnection;

    use jwt::{encode, Header};

    // services
    use services::users::*;
    use services::pastes::*;
    use services::auth::JwtClaims;

    // models
    use models::pastes::*;
    use models::schema::*;

    use DB_POOL;
    use ENV;

    pub const TEST_USER: NewUser = NewUser {
        username: "test",
        email: "test@example.com",
        password: "password",
    };
    pub const TEST_PASTE_DATA: &str = "test paste data";

    pub struct Data {
        pub user: User,
        pub paste: Paste,
    }

    pub fn create() -> Data {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let user = create_user(&TEST_USER, conn).expect("Fail to create test user");
        let test_paste = NewPaste {
            user_id: user.id,
            data: TEST_PASTE_DATA.to_string(),
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

    pub fn normal_user_auth_token(user_id: i32, username: &str) -> String {
        let claims = JwtClaims {
            user_id,
            username: username.to_string(),
            admin: false,
        };
        let jwt_secret: &str = ENV.jwt_secret.as_ref();
        encode(&Header::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }

    pub fn admin_user_auth_token(user_id: i32, username: &str) -> String {
        let claims = JwtClaims {
            user_id,
            username: username.to_string(),
            admin: true,
        };
        let jwt_secret: &str = ENV.jwt_secret.as_ref();
        encode(&Header::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }
}

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

    use jwt::{encode, Header as JwtHeader};

    use rocket::http::Header;

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
    pub const TEST_USER_ALT: NewUser = NewUser {
        username: "test_alt",
        email: "test_alt@example.com",
        password: "password",
    };
    pub const TEST_PASTE_DATA: &str = "test paste data";

    pub struct Data<'a> {
        pub user: User,
        pub user_alt: User,
        pub paste: Paste,
        pub admin_header: Header<'a>,
        pub normal_header: Header<'a>,
    }

    pub fn create<'a>() -> Data<'a> {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let user = create_user(&TEST_USER, conn).expect("Fail to create test user");
        let user_alt = create_user(&TEST_USER_ALT, conn).expect("Fail to create test user alt");
        let test_paste = NewPaste {
            user_id: user.id,
            data: TEST_PASTE_DATA.to_string(),
        };
        let paste = create_paste(&test_paste, conn).expect("Fail to create test paste");

        let normal_token = normal_user_auth_token(user.id, &user.username);
        let normal_header = Header::new("Authorization", "Bearer ".to_string() + &normal_token);
        let admin_token = admin_user_auth_token(1, "admin");
        let admin_header = Header::new("Authorization", "Bearer ".to_string() + &admin_token);

        Data {
            user,
            user_alt,
            paste,
            normal_header,
            admin_header,
        }
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

    pub fn recreate<'a>() -> Data<'a> {
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
        encode(&JwtHeader::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }

    pub fn admin_user_auth_token(user_id: i32, username: &str) -> String {
        let claims = JwtClaims {
            user_id,
            username: username.to_string(),
            admin: true,
        };
        let jwt_secret: &str = ENV.jwt_secret.as_ref();
        encode(&JwtHeader::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }
}

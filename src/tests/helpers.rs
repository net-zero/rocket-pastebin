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

macro_rules! req {
    ($method: expr, $endpoint: expr, $header: expr) => ({
        let mut req = MockRequest::new($method, $endpoint);
        req.add_header($header);
        req
    })
}

macro_rules! trivial_token_tests {
    ($rocket: expr, $req: expr) => (
        // empty token test
        run_test!($rocket, $req, |mut response: Response| {
            let body = body_string!(response);
            let err: Error = serde_json::from_str(&body).unwrap();
            assert_eq!(err.code, Status::Unauthorized.code);
            assert_eq!(err.msg, "token not found");
        });

        // invalid token test
        let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
        let mut req = $req;
        req.add_header(wrong_token);
        run_test!($rocket, req, |mut response: Response| {
            let body = body_string!(response);
            let err: Error = serde_json::from_str(&body).unwrap();
            assert_eq!(err.code, Status::Unauthorized.code);
            assert_eq!(err.msg, "invalid token");
        });

        // expired token test
        let expired_token = Header::new("Authorization", format!("Bearer {}", testdata::expired_token()));
        let mut req = $req;
        req.add_header(expired_token);
        run_test!($rocket, req, |mut response: Response| {
            let body = body_string!(response);
            let err: Error = serde_json::from_str(&body).unwrap();
            assert_eq!(err.code, Status::Unauthorized.code);
            assert_eq!(err.msg, "expired token");
        });
    )
}

macro_rules! trivial_perm_tests {
    ($rocket: expr, $normal_req: expr, $admin_req: expr, $admin_test_fn: expr) => (
        // without permission
        run_test!($rocket, $normal_req, |mut response: Response| {
            let body = body_string!(response);
            let err: Error = serde_json::from_str(&body).unwrap();
            assert_eq!(err.code, Status::Forbidden.code);
            assert_eq!(err.msg, "permission denied");
        });

        // id with admin token
        run_test!($rocket, $admin_req, |response: Response| {
            assert!(response.status().code != Status::Forbidden.code);
            $admin_test_fn(response);
        });
    )
}

pub mod testdata {
    use diesel;
    use diesel::prelude::*;
    use diesel::pg::PgConnection;

    use jwt::{encode, Header as JwtHeader};

    use rocket::http::Header;

    use time;

    // services
    use services::user::*;
    use services::paste::*;
    use services::auth::JwtClaims;

    // models
    use models::paste::*;
    use models::schema::*;

    use DB_POOL;
    use ENV;

    const DAY: i64 = 60 * 60 * 24;

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
        pub normal_header_alt: Header<'a>,
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
        let normal_token_alt = normal_user_auth_token(user_alt.id, &user_alt.username);
        let normal_header_alt = Header::new("Authorization",
                                            "Bearer ".to_string() + &normal_token_alt);
        let admin_token = admin_user_auth_token(1, "admin");
        let admin_header = Header::new("Authorization", "Bearer ".to_string() + &admin_token);

        Data {
            user,
            user_alt,
            paste,
            normal_header,
            normal_header_alt,
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
        let now = time::get_time().sec;
        let claims = JwtClaims {
            iat: now,
            exp: now + 7 * DAY,
            user_id,
            username: username.to_string(),
            admin: false,
        };
        let jwt_secret: &str = ENV.jwt_secret.as_ref();
        encode(&JwtHeader::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }

    pub fn admin_user_auth_token(user_id: i32, username: &str) -> String {
        let now = time::get_time().sec;
        let claims = JwtClaims {
            iat: now,
            exp: now + 7 * DAY,
            user_id,
            username: username.to_string(),
            admin: true,
        };
        let jwt_secret: &str = ENV.jwt_secret.as_ref();
        encode(&JwtHeader::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }

    pub fn expired_token() -> String {
        let now = time::get_time().sec;
        let claims = JwtClaims {
            iat: now,
            exp: now - 1,
            user_id: 1,
            username: "test user".to_string(),
            admin: false,
        };
        let jwt_secret: &str = ENV.jwt_secret.as_ref();
        encode(&JwtHeader::default(), &claims, jwt_secret.as_bytes()).unwrap()
    }
}

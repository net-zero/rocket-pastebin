use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, Header, ContentType};
use rocket::Response;

use serde_json;

use helpers::error::Error;

use services::users::User;
use controllers::user::UserPayload;

use tests::helpers;
use self::helpers::testdata;

#[test]
fn test_me() {
    let test_user = testdata::recreate().user;
    let auth_token = testdata::normal_user_auth_token(test_user.id, &test_user.username);
    let auth_header = Header::new("Authorization", "Bearer ".to_string() + &auth_token);
    let rocket = rocket();

    let mut req = MockRequest::new(Get, "/users/me");
    req.add_header(auth_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user.id, test_user.id);
        assert_eq!(user.username, test_user.username);
        assert_eq!(user.email, test_user.email);
    });

    // without token
    let req = MockRequest::new(Get, "/users/me");
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "token not found");
    });

    // invalid token
    let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
    let mut req = MockRequest::new(Get, "/users/me");
    req.add_header(wrong_token);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "invalid token");
    });
}

#[test]
fn test_get_users() {
    let test_user = testdata::recreate().user;
    let auth_token = testdata::admin_user_auth_token(1, "admin");
    let auth_header = Header::new("Authorization", "Bearer ".to_string() + &auth_token);
    let rocket = rocket();

    let mut req = MockRequest::new(Get, "/users");
    req.add_header(auth_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let users: Vec<User> = serde_json::from_str(&body).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, test_user.id);
        assert_eq!(users[0].username, test_user.username);
        assert_eq!(users[0].email, test_user.email);
    });

    // without token
    let req = MockRequest::new(Get, "/users");
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "token not found");
    });

    // invalid token
    let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
    let mut req = MockRequest::new(Get, "/users");
    req.add_header(wrong_token);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "invalid token");
    });

    // normal user token
    let auth_token = testdata::normal_user_auth_token(test_user.id, &test_user.username);
    let auth_header = Header::new("Authorization", "Bearer ".to_string() + &auth_token);
    let mut req = MockRequest::new(Get, "/users");
    req.add_header(auth_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Forbidden.code);
        assert_eq!(err.msg, "permission denied");
    });
}

macro_rules! create_user_req {
    ($new_user: expr) => (
        MockRequest::new(Post, "/users")
        .header(ContentType::Form)
        .body(&format!("username={}&email={}&password={}&confirm_password={}",
                       $new_user.username,
                       $new_user.email,
                       $new_user.password,
                       $new_user.confirm_password));
    )
}

#[test]
fn test_create_user() {
    testdata::recreate();

    let mut new_user = UserPayload {
        username: "test_new_user".to_string(),
        email: "test_new_user@example.com".to_string(),
        password: "new_user_password".to_string(),
        confirm_password: "new_user_password".to_string(),
    };
    let rocket = rocket();

    let req = create_user_req!(new_user);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user.username, new_user.username);
        assert_eq!(user.email, new_user.email);
    });

    // mismatch password
    new_user.confirm_password = "wrong password".to_string();
    let req = create_user_req!(new_user);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "password mismatch");
    });

    // duplicate username
    new_user.confirm_password = new_user.password.clone();
    new_user.username = testdata::TEST_USER.username.to_string();
    let req = create_user_req!(new_user);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::InternalServerError.code);
        assert_eq!(err.msg, "fail to create user");
    });
}


#[test]
fn test_get_user_by_id() {
    let test_user = testdata::recreate().user;
    let mut endpoint = format!("/users/{}", test_user.id);
    let normal_token = testdata::normal_user_auth_token(test_user.id, &test_user.username);
    let normal_header = Header::new("Authorization", "Bearer ".to_string() + &normal_token);
    let admin_token = testdata::admin_user_auth_token(1, "admin");
    let admin_header = Header::new("Authorization", "Bearer ".to_string() + &admin_token);
    let rocket = rocket();

    let mut req = MockRequest::new(Get, &endpoint);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user.id, test_user.id);
        assert_eq!(user.username, test_user.username);
        assert_eq!(user.email, test_user.email);
    });

    // without token
    let req = MockRequest::new(Get, &endpoint);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "token not found");
    });

    // invalid token
    let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
    let mut req = MockRequest::new(Get, &endpoint);
    req.add_header(wrong_token);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "invalid token");
    });

    // id without permission
    endpoint = format!("/users/{}", -1);
    let mut req = MockRequest::new(Get, &endpoint);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Forbidden.code);
        assert_eq!(err.msg, "permission denied");
    });

    // id with admin token
    let mut req = MockRequest::new(Get, &endpoint);
    req.add_header(admin_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::NotFound.code);
        assert_eq!(err.msg, "user not found");
    })
}

macro_rules! update_user_req {
    ($updated_user: expr, $user_id: expr) => (
        MockRequest::new(Put, format!("/users/{}", $user_id))
        .header(ContentType::Form)
        .body(&format!("username={}&email={}&password={}&confirm_password={}",
                       $updated_user.0,
                       $updated_user.1,
                       $updated_user.2,
                       $updated_user.3));
    )
}

#[test]
fn test_update_user_by_id() {
    let test_user = testdata::recreate().user;
    let mut endpoint = format!("/users/{}", test_user.id);
    let normal_token = testdata::normal_user_auth_token(test_user.id, &test_user.username);
    let normal_header = Header::new("Authorization", "Bearer ".to_string() + &normal_token);
    let admin_token = testdata::admin_user_auth_token(1, "admin");
    let admin_header = Header::new("Authorization", "Bearer ".to_string() + &admin_token);
    let mut updated_user =
        ("update_user", "update_user@exmaple.com", "updated_password", "updated_password");
    let rocket = rocket();

    let mut req = update_user_req!(updated_user, test_user.id);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user.username, updated_user.0);
        assert_eq!(user.email, updated_user.1);
    });

    // without token
    let req = update_user_req!(updated_user, test_user.id);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "token not found");
    });

    // invalid token
    let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
    let mut req = update_user_req!(updated_user, test_user.id);
    req.add_header(wrong_token);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "invalid token");
    });

    // id without permission
    let mut req = update_user_req!(updated_user, -1);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Forbidden.code);
        assert_eq!(err.msg, "permission denied");
    });

    // id with admin token
    let mut req = update_user_req!(updated_user, -1);
    req.add_header(admin_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::InternalServerError.code);
        assert_eq!(err.msg, "fail to update user");
    })
}

#[test]
fn test_delete_user_by_id() {
    let test_user = testdata::recreate().user_alt;
    let mut endpoint = format!("/users/{}", test_user.id);
    let normal_token = testdata::normal_user_auth_token(test_user.id, &test_user.username);
    let normal_header = Header::new("Authorization", "Bearer ".to_string() + &normal_token);
    let admin_token = testdata::admin_user_auth_token(1, "admin");
    let admin_header = Header::new("Authorization", "Bearer ".to_string() + &admin_token);
    let rocket = rocket();

    let mut req = MockRequest::new(Delete, &endpoint);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        assert!(body.contains("1"));
    });

    // without token
    let req = MockRequest::new(Delete, &endpoint);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "token not found");
    });

    // invalid token
    let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
    let mut req = MockRequest::new(Delete, &endpoint);
    req.add_header(wrong_token);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "invalid token");
    });

    // id without permission
    endpoint = format!("/users/{}", -1);
    let mut req = MockRequest::new(Delete, &endpoint);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Forbidden.code);
        assert_eq!(err.msg, "permission denied");
    });

    // id with admin token
    let mut req = MockRequest::new(Delete, &endpoint);
    req.add_header(admin_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        assert!(body.contains("0"));
    })
}

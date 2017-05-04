use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, Header, ContentType};
use rocket::Response;

use serde_json;

use helpers::error::Error;

use services::user::User;
use controllers::user::UserPayload;

use tests::helpers;
use self::helpers::testdata;

#[test]
fn test_me() {
    let testdata::Data {
        user: test_user,
        normal_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();

    let req = req!(Get, "/users/me", normal_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user.id, test_user.id);
        assert_eq!(user.username, test_user.username);
        assert_eq!(user.email, test_user.email);
    });

    trivial_token_tests!(&rocket, MockRequest::new(Get, "/users/me"));
}

#[test]
fn test_get_users() {
    let testdata::Data {
        user: test_user,
        admin_header,
        normal_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();

    let req = req!(Get, "/users", admin_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let users: Vec<User> = serde_json::from_str(&body).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, test_user.id);
        assert_eq!(users[0].username, test_user.username);
        assert_eq!(users[0].email, test_user.email);
    });

    trivial_token_tests!(&rocket, MockRequest::new(Get, "/users"));

    // normal user token
    let req = req!(Get, "/users", normal_header);
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
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "duplicate username");
    });

    // duplicate email
    new_user.username = "justnewuser".to_string();
    new_user.email = testdata::TEST_USER.email.to_string();
    let req = create_user_req!(new_user);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "duplicate email");
    });
}


#[test]
fn test_get_user_by_id() {
    let testdata::Data {
        user: test_user,
        admin_header,
        normal_header,
        ..
    } = testdata::recreate();
    let mut endpoint = format!("/users/{}", test_user.id);
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

    trivial_token_tests!(&rocket, MockRequest::new(Get, &endpoint));

    endpoint = format!("/users/{}", -1);
    let normal_req = req!(Get, &endpoint, normal_header.clone());
    let admin_req = req!(Get, &endpoint, admin_header.clone());
    trivial_perm_tests!(&rocket, normal_req, admin_req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::NotFound.code);
        assert_eq!(err.msg, "data not found");
    });
}

macro_rules! update_user_req {
    ($updated_user: expr, $user_id: expr, $header: expr) => ({
        let mut req = MockRequest::new(Put, format!("/users/{}", $user_id))
        .header(ContentType::Form)
        .body(&format!("username={}&email={}&password={}&confirm_password={}",
                       $updated_user.0,
                       $updated_user.1,
                       $updated_user.2,
                       $updated_user.3));
        req.add_header($header);
        req
    })
}

#[test]
fn test_update_user_by_id() {
    let testdata::Data {
        user: test_user,
        admin_header,
        normal_header,
        ..
    } = testdata::recreate();
    let updated_user =
        ("update_user", "update_user@exmaple.com", "updated_password", "updated_password");
    let rocket = rocket();

    let req = update_user_req!(updated_user, test_user.id, normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user.username, updated_user.0);
        assert_eq!(user.email, updated_user.1);
    });

    let dummy_header = Header::new("dummy", "dummy");
    trivial_token_tests!(&rocket,
                         update_user_req!(updated_user, test_user.id, dummy_header.clone()));

    let normal_req = update_user_req!(updated_user, -1, normal_header.clone());
    let admin_req = update_user_req!(updated_user, -1, admin_header.clone());
    trivial_perm_tests!(&rocket, normal_req, admin_req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::NotFound.code);
        assert_eq!(err.msg, "data not found");
    });
}

#[test]
fn test_delete_user_by_id() {
    let testdata::Data {
        user_alt: test_user,
        admin_header,
        normal_header_alt: normal_header,
        ..
    } = testdata::recreate();
    let mut endpoint = format!("/users/{}", test_user.id);
    let rocket = rocket();

    let mut req = MockRequest::new(Delete, &endpoint);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        println!("{}", &body);
        assert!(body.contains("1"));
    });

    trivial_token_tests!(&rocket, MockRequest::new(Delete, &endpoint));

    endpoint = format!("/users/{}", -1);
    let normal_req = req!(Delete, &endpoint, normal_header.clone());
    let admin_req = req!(Delete, &endpoint, admin_header.clone());
    trivial_perm_tests!(&rocket, normal_req, admin_req, |mut response: Response| {
        let body = body_string!(response);
        assert!(body.contains("0"));
    });
}

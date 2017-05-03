use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, ContentType};
use rocket::Response;

use serde_json;

use helpers::error::Error;

use tests::helpers;
use self::helpers::testdata;

macro_rules! login_req {
    ($username: expr, $password: expr) => (
        MockRequest::new(Post, "/login")
        .header(ContentType::Form)
        .body(&format!("username={}&password={}",
                       $username,
                       $password));
    )
}

#[test]
fn test_login() {
    testdata::recreate();

    let test_user = testdata::TEST_USER;
    let rocket = rocket();

    let req = login_req!(test_user.username, test_user.password);
    run_test!(&rocket, req, |mut response: Response| {
        let body = response.body().unwrap().into_string().unwrap();
        assert_eq!(response.status(), Status::Ok);
        assert!(body.contains("token"));
    });

    // wrong username
    let req = login_req!("wrong_user", test_user.password);
    run_test!(&rocket, req, |mut response: Response| {
        let body = response.body().unwrap().into_string().unwrap();
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "wrong username or password");
    });

    // wrong password
    let req = login_req!(test_user.username, "wrong password");
    run_test!(&rocket, req, |mut response: Response| {
        let body = response.body().unwrap().into_string().unwrap();
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "wrong username or password");
    });
}

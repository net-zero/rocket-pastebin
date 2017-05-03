use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, Header, ContentType};
use rocket::Response;

use serde_json;

use helpers::error::Error;

use models::pastes::{Paste, NewPaste};

use tests::helpers;
use self::helpers::testdata;

#[test]
fn test_get_pastes() {
    let testdata::Data {
        paste: test_paste,
        admin_header,
        normal_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();

    let mut req = MockRequest::new(Get, "/pastes");
    req.add_header(admin_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let pastes: Vec<Paste> = serde_json::from_str(&body).unwrap();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0].id, test_paste.id);
        assert_eq!(pastes[0].user_id, test_paste.user_id);
        assert_eq!(pastes[0].data, test_paste.data);
    });

    // normal user token
    let mut req = MockRequest::new(Get, "/pastes");
    req.add_header(normal_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Forbidden.code);
        assert_eq!(err.msg, "permission denied");
    });

    // without token
    let req = MockRequest::new(Get, "/pastes");
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "token not found");
    });

    // invalid token
    let wrong_token = Header::new("Authorization", "Bearer wrongtoken");
    let mut req = MockRequest::new(Get, "/pastes");
    req.add_header(wrong_token);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Unauthorized.code);
        assert_eq!(err.msg, "invalid token");
    });
}

macro_rules! create_paste_req {
    ($new_paste: expr) => (
        MockRequest::new(Post, "/pastes")
        .header(ContentType::Form)
        .body(&format!("user_id={}&data={}",$new_paste.user_id, $new_paste.data));
    )
}

#[test]
fn test_create_paste() {
    let testdata::Data {
        user,
        normal_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();
    let mut new_paste = NewPaste {
        user_id: user.id,
        data: "test new paste".to_string(),
    };

    let mut req = create_paste_req!(new_paste);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let paste: Paste = serde_json::from_str(&body).unwrap();
        assert_eq!(paste.user_id, user.id);
        assert_eq!(paste.data, new_paste.data);
    });

    // user_id doesn't match with token
    new_paste.user_id = -1;
    let mut req = create_paste_req!(new_paste);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "user_id doesn't match jwt token");
    });
}

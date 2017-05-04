use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, Header, ContentType};
use rocket::Response;

use serde_json;

use helpers::error::Error;

use models::paste::{Paste, NewPaste};

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

    let req = req!(Get, "/pastes", admin_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let pastes: Vec<Paste> = serde_json::from_str(&body).unwrap();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0], test_paste);
    });

    // normal user token
    let req = req!(Get, "/pastes", normal_header);
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::Forbidden.code);
        assert_eq!(err.msg, "permission denied");
    });

    trivial_token_tests!(&rocket, MockRequest::new(Get, "/pastes"));
}

macro_rules! create_paste_req {
    ($new_paste: expr, $header: expr) => ({
        let mut req = MockRequest::new(Post, "/pastes")
            .header(ContentType::Form)
            .body(&format!("user_id={}&data={}",$new_paste.user_id, $new_paste.data));
        req.add_header($header);
        req
    })
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

    let req = create_paste_req!(new_paste, normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let paste: Paste = serde_json::from_str(&body).unwrap();
        assert_eq!(paste.user_id, user.id);
        assert_eq!(paste.data, new_paste.data);
    });

    // user_id doesn't match with token
    new_paste.user_id = -1;
    let req = create_paste_req!(new_paste, normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "user_id doesn't match jwt token");
    });

    let dummy_header = Header::new("dummy", "dummy");
    trivial_token_tests!(&rocket, create_paste_req!(new_paste, dummy_header.clone()));
}

#[test]
fn test_get_paste_by_id() {
    let test_paste = testdata::recreate().paste;
    let rocket = rocket();

    let req = MockRequest::new(Get, format!("/pastes/{}", test_paste.id));
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let paste: Paste = serde_json::from_str(&body).unwrap();
        assert_eq!(paste, test_paste);
    });

    // invalid paste id
    let req = MockRequest::new(Get, format!("/pastes/{}", -1));
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::NotFound.code);
        assert_eq!(err.msg, "data not found");
    });
}

macro_rules! update_paste_req {
    ($updated_paste: expr, $endpoint: expr, $header: expr) => ({
        let mut req = MockRequest::new(Put, $endpoint)
            .header(ContentType::Form)
            .body(&format!("id={}&user_id={}&data={}",
                           $updated_paste.id,
                           $updated_paste.user_id,
                           $updated_paste.data));
        req.add_header($header);
        req
    })
}

#[test]
fn test_update_paste_by_id() {
    let testdata::Data {
        paste: test_paste,
        normal_header,
        admin_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();
    let mut updated_paste = Paste {
        id: test_paste.id,
        user_id: test_paste.user_id,
        data: "test updated paste".to_string(),
    };

    let endpoint = format!("/users/{}/pastes/{}", test_paste.user_id, test_paste.id);
    let req = update_paste_req!(updated_paste, &endpoint, normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let paste: Paste = serde_json::from_str(&body).unwrap();
        assert_eq!(paste, updated_paste);
    });

    // update using admin permission
    updated_paste.data = "update paste by admin".to_string();
    let req = update_paste_req!(updated_paste, &endpoint, admin_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let paste: Paste = serde_json::from_str(&body).unwrap();
        assert_eq!(paste, updated_paste);
    });

    // user_id doesn't match
    updated_paste.user_id = -1;
    let req = update_paste_req!(updated_paste, &endpoint, admin_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "user_id or paste id doesn't match");
    });

    // paste id doesn't match
    updated_paste.user_id = test_paste.id;
    updated_paste.id = -1;
    let req = update_paste_req!(updated_paste, &endpoint, admin_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let err: Error = serde_json::from_str(&body).unwrap();
        assert_eq!(err.code, Status::BadRequest.code);
        assert_eq!(err.msg, "user_id or paste id doesn't match");
    });

    let dummy_header = Header::new("dummy", "dummy");
    trivial_token_tests!(&rocket,
                         update_paste_req!(updated_paste, &endpoint, dummy_header.clone()));
}

#[test]
fn test_delete_paste_by_id() {
    let testdata::Data {
        paste: test_paste,
        normal_header,
        admin_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();

    let endpoint = format!("/users/{}/pastes/{}", test_paste.user_id, test_paste.id);
    let mut req = MockRequest::new(Delete, &endpoint);
    req.add_header(normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        body.contains("1");
    });

    trivial_token_tests!(&rocket, MockRequest::new(Delete, "/users/-1/pastes/-1"));

    let admin_req = req!(Delete, "/users/-1/pastes/-1", admin_header.clone());
    let normal_req = req!(Delete, "/users/-1/pastes/-1", normal_header.clone());
    trivial_perm_tests!(&rocket, normal_req, admin_req, |mut response: Response| {
        let body = body_string!(response);
        body.contains("0");
    });
}

#[test]
fn test_get_pastes_by_user_id() {
    let testdata::Data {
        user,
        paste: test_paste,
        admin_header,
        normal_header,
        ..
    } = testdata::recreate();
    let rocket = rocket();

    let endpoint = format!("/users/{}/pastes", user.id);
    let req = req!(Get, &endpoint, normal_header.clone());
    run_test!(&rocket, req, |mut response: Response| {
        let body = body_string!(response);
        let pastes: Vec<Paste> = serde_json::from_str(&body).unwrap();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0], test_paste);
    });

    trivial_token_tests!(&rocket, MockRequest::new(Get, &endpoint));

    let admin_req = req!(Get, &endpoint, admin_header.clone());
    let normal_req = req!(Get, "/users/-1/pastes", normal_header.clone());
    trivial_perm_tests!(&rocket, normal_req, admin_req, |mut response: Response| {
        let body = body_string!(response);
        let pastes: Vec<Paste> = serde_json::from_str(&body).unwrap();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0], test_paste);
    });
}

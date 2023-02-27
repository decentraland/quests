mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_server::routes::ErrorResponse;

#[actix_web::test]
async fn get_quests_should_be_200() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let quest_definition = quest_samples::grab_some_pies();

    let req = TestRequest::post()
        .uri("/quests")
        .set_json(quest_definition)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let req = TestRequest::get()
        .uri("/quests?offset=0&limit=2")
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());
}

#[actix_web::test]
async fn get_quests_should_be_400() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::get().uri("/quests?offset=0aa").to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());

    let req = TestRequest::get().uri("/quests?limit=0aa").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));

    let req = TestRequest::get()
        .uri("/quests?offset=10a&limit=0aa")
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}

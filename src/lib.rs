use actix_web::{get, http::StatusCode, post, web, HttpRequest, HttpResponse, Responder};
use std::sync::Mutex;
use tracing::{info, instrument};

mod utils;

#[get("/")]
pub async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("If you see this, it means that the web server is working correctly!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(format!("Ok good, echoing: {}", req_body))
}

#[post("/login")]
pub async fn login(token: web::Query<utils::auth::IdToken>) -> impl Responder {
    let claims = utils::auth::VALIDATOR.validate_token(&token.id_token);

    match claims {
        Some(claims) => HttpResponse::Ok().body("Good token!"),
        None => HttpResponse::build(StatusCode::FORBIDDEN).body("Bad token!"),
    }
}

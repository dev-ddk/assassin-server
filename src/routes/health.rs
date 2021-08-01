use actix_web::{get, web, HttpResponse, Responder};
use tracing::instrument;

#[get("/")]
#[instrument]
pub async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("If you see this, it means that the web server is working correctly!!")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello_world);
}

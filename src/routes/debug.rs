use actix_web::{get, http::StatusCode, web, HttpRequest, HttpResponse, Responder};
use tracing::instrument;

use crate::models::player;
use crate::utils::auth;

#[get("/player/{id}")]
#[instrument]
pub async fn search(req: HttpRequest, web::Path(id): web::Path<i32>) -> impl Responder {
    let ext = req.extensions();

    let claims = ext.get::<auth::UserClaims>().unwrap();

    let account = player::Player::find(id);

    match account {
        Ok(account) => {
            let response = format!(
                "[Authenticated as {}] Player info: {}",
                claims.email,
                serde_json::to_string(&account).expect("Could not serialize user")
            );

            HttpResponse::Ok().body(response)
        }
        Err(_) => HttpResponse::build(StatusCode::NOT_FOUND).body("User not found"),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(search);
}

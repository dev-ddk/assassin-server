use actix_web::{http::StatusCode, post, web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{info, instrument};

use crate::models::player;
use crate::utils::auth;

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    nickname: String,
}

#[post("/register")]
#[instrument]
pub async fn register(claims: auth::UserClaims, info: web::Json<RegisterInfo>) -> impl Responder {
    info!("Looking for a player with uid {}", &claims.user_id);

    let res = player::Player::register(
        info.nickname.clone(),
        claims.email.clone(),
        claims.user_id.clone(),
    );

    match res {
        Ok(account) => {
            let response = format!(
                "[Authenticated as {}] Registered player info: {}",
                claims.email,
                serde_json::to_string(&account).expect("Could not serialize user")
            );

            HttpResponse::Ok().body(response)
        }
        Err(e) => {
            info!("Error during registration: {}", e);
            info!("Wrapped error: {}", e.root_cause());
            HttpResponse::build(StatusCode::BAD_REQUEST)
                .body("An error occurred during registration")
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}

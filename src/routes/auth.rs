use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use tracing::{info, instrument};

use crate::models::player;
use crate::utils::auth;
use crate::models::api_errors::ApiError;

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    nickname: String,
}

#[post("/register")]
#[instrument]
pub async fn register(claims: auth::UserClaims, info: web::Json<RegisterInfo>) -> Result<HttpResponse, ApiError> {
    info!("Looking for a player with uid {}", &claims.user_id);

    player::Player::register(
        info.nickname.clone(),
        claims.email.clone(),
        claims.user_id.clone(),
    )?;

    Ok(HttpResponse::Ok().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}

use actix_web::{http::StatusCode, post, web, HttpRequest, HttpResponse, Responder};
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
pub async fn register(req: HttpRequest, info: web::Json<RegisterInfo>) -> impl Responder {
    let ext = req.extensions();

    let claims = ext.get::<auth::UserClaims>().unwrap();

    info!("Looking for a player with uid {}", &claims.user_id);

    if player::Account::find_by_uid(&claims.user_id).is_err() {
        info!("No such player found: registering new player");

        let account = player::Account::register(
            info.nickname.clone(),
            claims.email.clone(),
            claims.user_id.clone(),
        )
        .unwrap();

        let response = format!(
            "[Authenticated as {}] Registered player info: {}",
            claims.email,
            serde_json::to_string(&account).expect("Could not serialize user")
        );

        HttpResponse::Ok().body(response)
    } else {
        HttpResponse::build(StatusCode::BAD_REQUEST).body("Already registered")
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}

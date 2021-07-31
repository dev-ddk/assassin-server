use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{error, info, instrument};

use crate::models::game::Game;
use crate::models::player::Player;

#[derive(Debug, Deserialize)]
pub struct GameCreationInfo {
    game_name: String,
}

#[post("/create")]
#[instrument]
pub async fn create(player: Player, info: web::Json<GameCreationInfo>) -> impl Responder {
    let game = Game::new(info.game_name.clone(), player.id);
    match game {
        Ok(game) => {
            info!("Created new game: {:?}", game);
            HttpResponse::Created()
        }
        Err(e) => {
            error!("Failed to create game: {:?}", e);
            HttpResponse::BadRequest()
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create);
}

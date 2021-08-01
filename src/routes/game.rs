use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};

use crate::models::enums::GameStatus;
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

#[derive(Debug, Deserialize)]
pub struct GameInfo {
    game_code: String,
}

#[post("/join")]
#[instrument]
pub async fn join(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::join(&info.game_code, player.id);

    match res {
        Ok(_) => {
            info!("Joined game: {:?}", info.game_code);
            HttpResponse::Ok()
        }
        Err(e) => {
            error!("Failed to join game: {:?}", e);
            HttpResponse::BadRequest()
        }
    }
}

#[post("/start")]
#[instrument]
pub async fn start(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::start_game(&info.game_code, player.id);

    match res {
        Ok(_) => {
            info!("Started game: {:?}", info.game_code);
            HttpResponse::Ok()
        }
        Err(e) => {
            error!("Failed to start game: {:?}", e);
            HttpResponse::BadRequest()
        }
    }
}

#[derive(Serialize)]
pub struct StatusResult {
    game_status: GameStatus,
}

//TODO: only people inside the lobby should be able to query this
#[get("/status")]
#[instrument]
pub async fn get_status(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let status = Game::get_game_status(&info.game_code);
    match status {
        Ok(status) => HttpResponse::Ok().json(StatusResult {
            game_status: status,
        }),
        Err(e) => {
            info!("Failed to fetch game status: {:?}", e);
            HttpResponse::BadRequest().finish()
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(join)
        .service(start)
        .service(get_status);
}

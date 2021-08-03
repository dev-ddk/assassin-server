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

#[derive(Debug, Serialize)]
pub struct GameCreationResult {
    #[serde(rename(serialize = "gameCode"))]
    game_code: String,
}

#[post("/create")]
#[instrument]
pub async fn create(player: Player, info: web::Json<GameCreationInfo>) -> impl Responder {
    let game = Game::new(info.game_name.clone(), player.id);
    match game {
        Ok(game) => {
            info!("Created new game: {:?}", game);
            HttpResponse::Created().json(GameCreationResult {
                game_code: game.code,
            })
        }
        Err(e) => {
            error!("Failed to create game: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
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
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to join game: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
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
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to start game: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
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
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[get("/agent_info")]
#[instrument]
pub async fn get_agent_info(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let agent_info = player.get_agent_info(&info.game_code);
    match agent_info {
        Ok(agent_info) => HttpResponse::Ok().json(agent_info),
        Err(e) => {
            info!("Failed to get agent info: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[post("/kill")]
#[instrument]
pub async fn kill(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::kill_player(&info.game_code, player.id);
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            info!("Failed to kill target: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[get("/game_info")]
#[instrument]
pub async fn get_game_info(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::get_game_info(&info.game_code, player.id);
    match res {
        Ok(game_info) => HttpResponse::Ok().json(game_info),
        Err(e) => {
            info!("Failed to get game info: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[get("/user_info")]
#[instrument]
pub async fn get_user_info(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = player.get_user_info();
    match res {
        Ok(user_info) => HttpResponse::Ok().json(user_info),
        Err(e) => {
            info!("Failed to get user_info: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[get("/codenames")]
#[instrument]
pub async fn get_codenames(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::get_codenames(&info.game_code, player.id);
    match res {
        Ok(codenames) => HttpResponse::Ok().json(codenames),
        Err(e) => {
            info!("Failed to get codenames: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[get("/end_game")]
#[instrument]
pub async fn get_end_time(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::get_end_time(&info.game_code, player.id);
    match res {
        Ok(end_time) => HttpResponse::Ok().body(format!("{}", end_time)),
        Err(e) => {
            info!("Failed to get end time: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

#[post("/end_game")]
#[instrument]
pub async fn end_game(player: Player, info: web::Json<GameInfo>) -> impl Responder {
    let res = Game::stop_game(&info.game_code, player.id);
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            info!("Failed to get end time: {:?}", e);
            HttpResponse::BadRequest().body(format!("{:?}", e.root_cause()))
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(join)
        .service(start)
        .service(get_status)
        .service(get_agent_info)
        .service(kill)
        .service(get_game_info)
        .service(get_user_info)
        .service(get_codenames)
        .service(get_end_time)
        .service(end_game);
}

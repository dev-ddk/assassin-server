use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::models::enums::GameStatus;
use crate::models::game::Game;
use crate::models::player::Player;
use crate::models::api_errors::ApiError;

type HttpResult = std::result::Result<HttpResponse, ApiError>;

#[derive(Debug, Deserialize)]
pub struct GameCreationInfo {
    game_name: String,
}

#[post("/create_game")]
#[instrument]
pub async fn create(player: Player, info: web::Json<GameCreationInfo>) -> HttpResult {
    let game = Game::new(info.game_name.clone(), player.id)?;
    info!("Succesfully created game {}", game.code);
    Ok(HttpResponse::Created().json(GameInfo{ game_code: game.code }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameInfo {
    #[serde(rename(deserialize = "gameCode", serialize = "gameCode"))]
    game_code: String,
}

#[post("/join_game")]
#[instrument]
pub async fn join(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let res = Game::join(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/start_game")]
#[instrument]
pub async fn start(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let res = Game::start_game(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
pub struct StatusResult {
    game_status: GameStatus,
}

//TODO: only people inside the lobby should be able to query this
#[get("/game_status")]
#[instrument]
pub async fn get_status(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let status = Game::get_game_status(&info.game_code)?;
    Ok(HttpResponse::Ok().json(StatusResult{ game_status: status }))
}

#[get("/agent_info")]
#[instrument]
pub async fn get_agent_info(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let agent_info = player.get_agent_info(&info.game_code)?;
    Ok(HttpResponse::Ok().json(agent_info))
}

#[post("/kill")]
#[instrument]
pub async fn kill(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let res = Game::kill_player(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/game_info")]
#[instrument]
pub async fn get_game_info(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let game_info = Game::get_game_info(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().json(game_info))
}

#[get("/user_info")]
#[instrument]
pub async fn get_user_info(player: Player) -> HttpResult {
    let user_info = player.get_user_info()?;
    Ok(HttpResponse::Ok().json(user_info))
}

#[get("/codenames")]
#[instrument]
pub async fn get_codenames(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let codenames = Game::get_codenames(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().json(codenames))
}

#[get("/end_game")]
#[instrument]
pub async fn get_end_time(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let end_time = Game::get_end_time(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().body(format!("{}", end_time)))
}

#[post("/end_game")]
#[instrument]
pub async fn end_game(player: Player, info: web::Query<GameInfo>) -> HttpResult {
    let res = Game::stop_game(&info.game_code, player.id)?;
    Ok(HttpResponse::Ok().finish())
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

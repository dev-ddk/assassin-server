#![allow(non_camel_case_types)]
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, DbEnum, Serialize, Deserialize, PartialEq)]
#[PgType = "game_status_t"]
#[DieselType = "Game_status_t"]
#[DbValueStyle = "verbatim"]
pub enum GameStatus {
    WAITING_FOR_PLAYERS,
    ACTIVE,
    FINISHED,
    PAUSED,
}

#[derive(Debug, Clone, DbEnum, Serialize, Deserialize, PartialEq)]
#[PgType = "player_status_t"]
#[DieselType = "Player_status_t"]
#[DbValueStyle = "verbatim"]
pub enum PlayerStatus {
    DEAD,
    ALIVE,
    LEFT_GAME,
}

#[derive(Debug, Clone, DbEnum, Serialize, Deserialize)]
#[PgType = "target_status_t"]
#[DieselType = "Target_status_t"]
#[DbValueStyle = "verbatim"]
pub enum TargetStatus {
    CURRENT,
    KILL_SUCCESS,
    TARGET_LEFT,
    REASSIGNED,
    GAME_END,
}

#[derive(Debug, Clone, DbEnum, Serialize, Deserialize)]
#[PgType = "role_t"]
#[DieselType = "Role_t"]
#[DbValueStyle = "verbatim"]
pub enum Role {
    USER,
    ADMIN,
}

impl Default for Role {
    fn default() -> Self {
        Role::USER
    }
}

impl Default for GameStatus {
    fn default() -> Self {
        GameStatus::WAITING_FOR_PLAYERS
    }
}

use chrono::NaiveDateTime;
use color_eyre::Result;
use diesel::prelude::*;
use diesel::{
    result::DatabaseErrorKind::UniqueViolation, result::Error::DatabaseError, Associations,
    Identifiable, Insertable, Queryable,
};
use lazy_static::lazy_static;
use rand::{prelude::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::db;
use crate::models::enums::{GameStatus, PlayerStatus, TargetStatus};
use crate::models::player::Player;

use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "game"]
pub struct NewGame {
    name: String,
    owner: i32,
    code: String,
    status: GameStatus,
}

#[derive(Debug, Serialize, Associations, Deserialize, Queryable)]
#[table_name = "game"]
#[belongs_to(Player, foreign_key = "owner")]
pub struct Game {
    pub id: i32,
    pub name: Option<String>,
    pub owner: i32,
    pub code: String,
    pub status: GameStatus,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Identifiable, Insertable)]
#[primary_key(player, game)]
#[table_name = "playergame"]
pub struct NewPlayerGame {
    player: i32,
    game: i32,
    target: Option<i32>,
    codename: String,
    status: PlayerStatus,
    target_status: TargetStatus,
}

const GAME_CODE_LEN: u32 = 8; //To be moved in some config file

lazy_static! {
    static ref CODE_CHARS: Vec<char> = ('A'..='Z').chain('0'..='9').collect();
}

fn get_game_code() -> String {
    let mut rng = thread_rng();
    (0..GAME_CODE_LEN)
        .map(|_| CODE_CHARS.iter().choose(&mut rng).unwrap())
        .collect()
}

impl Game {
    pub fn find(id: i32) -> Result<Self> {
        let conn = db::connection()?;
        let game = game::table.filter(game::id.eq(id)).first(&conn)?;

        Ok(game)
    }

    pub fn find_by_code(code: String) -> Result<Self> {
        let conn = db::connection()?;
        let game = game::table.filter(game::code.eq(code)).first(&conn)?;

        Ok(game)
    }

    pub fn new(game_name: String, game_owner: i32) -> Result<Self> {
        let conn = db::connection()?;
        let code = get_game_code();

        let mut new_game = NewGame {
            name: game_name,
            owner: game_owner,
            code,
            status: GameStatus::WAITING_FOR_PLAYERS,
        };

        conn.transaction(|| {
            loop {

                let res = diesel::insert_into(game::table)
                    .values(new_game.clone())
                    .get_result(&conn);

                match res {
                    Ok(game) => {
                        return Ok(game);
                    }
                    Err(DatabaseError(UniqueViolation, e)) => {
                        info!("Got unique violation while creating new game (probably due to duplicate game code): {:?}", e);
                        let code = get_game_code();
                        new_game.code = code;
                    }
                    Err(e) => {
                        let err_str = format!(
                            "Failed in creating game {}",
                            serde_json::to_string_pretty(&new_game).unwrap()
                        );
                        return Err(color_eyre::Report::new(e).wrap_err(err_str));
                    }
                }
            }
        })
    }

    // pub fn join(&self, code: String, account_id: i32) -> Result
}

use crate::db;
use crate::models::enums::{GameStatus, PlayerStatus};
use crate::models::player::Player;
use crate::utils::genstring::{get_agent_name, get_game_code};
use chrono::{DateTime, Utc};
use color_eyre::Result;
use diesel::prelude::*;
use diesel::{
    result::DatabaseErrorKind::UniqueViolation, result::Error::DatabaseError,
    result::Error::RollbackTransaction, Associations, Identifiable, Insertable, Queryable,
};
use serde::{Deserialize, Serialize};
use tracing::info;

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
    pub created_at: DateTime<Utc>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Identifiable, Insertable)]
#[primary_key(player, game)]
#[table_name = "playergame"]
pub struct NewPlayerGame {
    player: i32,
    game: i32,
    codename: String,
    status: PlayerStatus,
}

#[derive(Debug, Serialize, Associations, Deserialize, Queryable)]
#[belongs_to(Player, foreign_key = "player")]
#[belongs_to(Game, foreign_key = "game")]
#[table_name = "playergame"]
pub struct PlayerGame {
    pub player: i32,
    pub game: i32,
    pub codename: String,
    pub status: PlayerStatus,
    pub joined_at: DateTime<Utc>,
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

    pub fn join(code: &String, player_id: i32) -> Result<()> {
        let conn = db::connection()?;
        let codename = get_agent_name();

        let res = conn.transaction(|| {
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            //Find if player is already in a game, if so reject request
            let active_game_count = playergame::table
                .filter(playergame::player.eq(player_id))
                .inner_join(game::table)
                .filter(game::status.eq(GameStatus::ACTIVE))
                .count()
                .execute(&conn)?;

            if active_game_count > 0 {
                return Err(RollbackTransaction);
            }

            let new_player_game = NewPlayerGame {
                player: player_id,
                game: requested_game.id,
                codename,
                status: PlayerStatus::ALIVE,
            };

            diesel::insert_into(playergame::table)
                .values(new_player_game.clone())
                .execute(&conn)?;

            Ok(())
        });

        let err_str = format!("User {} couldn't join game {}", player_id, code);
        res.map_err(|e| color_eyre::Report::new(e).wrap_err(err_str))
    }

    pub fn start_game(code: &String, player_id: i32) -> Result<()> {
        let conn = db::connection()?;
        diesel::update(
            game::table
                .filter(game::code.eq(code))
                .filter(game::owner.eq(player_id)),
        )
        .set((
            game::status.eq(GameStatus::ACTIVE),
            game::start_time.eq(chrono::offset::Utc::now()),
            game::end_time.eq(chrono::offset::Utc::now() + chrono::Duration::days(3)), //TODO: de-hardcode this
        ))
        .execute(&conn)
        .map_err(|e| {
            color_eyre::Report::new(e)
                .wrap_err(format!("User {} couldn't start game {}", player_id, code))
        })?;

        Ok(())
    }

    pub fn get_game_status(code: &String) -> Result<GameStatus> {
        let conn = db::connection()?;
        let requested_game = game::table
            .filter(game::code.eq(code))
            .first::<Game>(&conn)
            .map_err(|e| {
                color_eyre::Report::new(e)
                    .wrap_err(format!("Couldn't fetch game status for code {}", code))
            })?;
        Ok(requested_game.status)
    }
}

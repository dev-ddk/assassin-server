use crate::db;
use crate::models::enums::{GameStatus, PlayerStatus, TargetStatus};
use crate::models::player::{AgentStats, Player};
use crate::utils::genstring::{get_agent_name, get_game_code};
use crate::models::model_errors::{ModelError, Result};

use chrono::{DateTime, Utc};
use color_eyre::Report;
use diesel::prelude::*;
use diesel::{
    result::DatabaseErrorKind::UniqueViolation, result::Error::DatabaseError,
    result::QueryResult, Associations, Identifiable,
    Insertable, Queryable,
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

#[derive(Debug, Serialize, Associations, Deserialize, Queryable, Identifiable)]
#[belongs_to(Player, foreign_key = "owner")]
#[table_name = "game"]
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

#[derive(Debug, Serialize, Queryable)]
pub struct GamePlayerInfo {
    pub nickname: String,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GameInfo {
    pub game_name: String,
    pub admin_nickname: String,
    pub players: Vec<GamePlayerInfo>,
}

#[derive(Debug, Serialize)]
pub struct GameStats {
    pub winner: AgentStats,
    pub ranking: Vec<AgentStats>,
}

#[derive(Debug, Serialize)]
pub struct Codenames {
    pub codenames: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct PlayerCount {
    assassin: i32,
    count: usize,
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
            name: game_name.clone(),
            owner: game_owner,
            code,
            status: GameStatus::WAITING_FOR_PLAYERS,
        };

        conn.transaction(|| {
            let active_game_count = playergame::table
                .inner_join(game::table)
                .filter(playergame::player.eq(game_owner))
                .filter(playergame::status.ne(PlayerStatus::LEFT_GAME))
                .filter(game::status.ne(GameStatus::FINISHED))
                .count()
                .get_result::<i64>(&conn)?;

            if active_game_count > 0 {
                info!("User has active game: rolling back! {}", active_game_count);
                return Err(ModelError::AlreadyInGame);
            }

            // Keep on generating codes until we get a unique one
            // Since the number of possible codes is around 2 trillion, we expect this
            // to run only once almost always
            loop {
                let res: QueryResult<Game> = diesel::insert_into(game::table)
                    .values(new_game.clone())
                    .get_result(&conn);

                match res {
                    // Creation of new game was successful
                    Ok(game) => {
                        //Insert the game owner into the game automatically
                        let new_player_game = NewPlayerGame {
                            player: game_owner.clone(),
                            game: game.id,
                            codename: get_agent_name(),
                            status: PlayerStatus::ALIVE
                        };

                        diesel::insert_into(playergame::table)
                            .values(new_player_game)
                            .execute(&conn)?;

                        return Ok(game);
                    },

                    // Creation of new game failed because of duplicated game code
                    Err(DatabaseError(UniqueViolation, e)) => {
                        info!("Got unique violation while creating new game (probably due to duplicate game code): {:?}", e);
                        let code = get_game_code();
                        new_game.code = code;
                    },

                    // Creation of new game failed for other reasons
                    Err(e) => {
                        return Err(ModelError::UnknownError(Report::new(e)));
                    }
                }
            }
        })
    }

    pub fn join(code: &String, player_id: i32) -> Result<()> {
        let conn = db::connection()?;
        let codename = get_agent_name();

        conn.transaction(|| {
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            //Find if player is already in a game, if so reject request
            let active_game_count = playergame::table
                .inner_join(game::table)
                .filter(playergame::player.eq(player_id))
                .filter(game::status.eq(GameStatus::ACTIVE))
                .count()
                .get_result::<i64>(&conn)?;

            if active_game_count > 0 {
                return Err(ModelError::AlreadyInGame);
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
        })
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
        .execute(&conn)?;

        Ok(())
    }

    pub fn stop_game(code: &String, player_id: i32) -> Result<()> {
        let conn = db::connection()?;

        conn.transaction(|| {
            let requested_game: Game = game::table
                .filter(game::code.eq(code))
                .first(&conn)?;

            let is_user_in_game = playergame::table
                .filter(playergame::game.eq(requested_game.id))
                .filter(playergame::player.eq(player_id))
                .count()
                .get_result::<i64>(&conn)? > 0;

            if !is_user_in_game {
                info!("User {} is currently not in the requested game {}. Cannot stop game", player_id, code);
                return Err(ModelError::NotInGame);
            }

            if requested_game.end_time.is_some() && requested_game.end_time.unwrap() > chrono::offset::Utc::now() {
                diesel::update(&requested_game)
                    .set(game::status.eq(GameStatus::FINISHED))
                    .execute(&conn)?;
                Ok(())
            } else {
                info!("Couldn't stop game (Either end time hasn't been set yet or the end time hasn't arrived yet)");
                Err(ModelError::GameNotStarted)
            }
        })
    }

    pub fn get_game_status(code: &String) -> Result<GameStatus> {
        let conn = db::connection()?;
        let requested_game = game::table
            .filter(game::code.eq(code))
            .first::<Game>(&conn)?;

        Ok(requested_game.status)
    }

    pub fn leave_game(code: &String, player_id: i32) -> Result<()> {
        let conn = db::connection()?;

        conn.transaction(|| {
            // Fetch the game, as long as it's not already finished
            // If it is finished, throw an error
            let requested_game: Game = game::table
                .filter(game::code.eq(code))
                .filter(game::status.ne(GameStatus::FINISHED))
                .first(&conn)?;

            // Update and leave the game
            diesel::update(
                playergame::table
                    .filter(playergame::game.eq(requested_game.id))
                    .filter(playergame::player.eq(player_id))
                    .filter(playergame::status.ne(PlayerStatus::LEFT_GAME)),
            )
            .set(playergame::status.eq(PlayerStatus::LEFT_GAME))
            .execute(&conn)?;

            Ok(())
        })
    }

    pub fn get_game_info(code: &String, player_id: i32) -> Result<GameInfo> {
        let conn = db::connection()?;

        conn.transaction(|| {
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            let is_user_in_game = playergame::table
                .filter(playergame::game.eq(requested_game.id))
                .filter(playergame::player.eq(player_id))
                .count()
                .get_result::<i64>(&conn)?
                > 0;

            if !is_user_in_game {
                info!(
                    "User {} is currently not in the requested game {}. Cannot get game info",
                    player_id, code
                );
                return Err(ModelError::NotInGame);
            }

            let owner: Player = player::table
                .filter(player::id.eq(requested_game.owner))
                .first(&conn)?;

            let players: Vec<GamePlayerInfo> = playergame::table
                .inner_join(player::table)
                .filter(playergame::game.eq(requested_game.id))
                .select((player::nickname, player::picture))
                .load::<GamePlayerInfo>(&conn)?;

            let game_info = GameInfo {
                //TODO: right now the game name is nullable, debate whether we should require it?
                game_name: requested_game.name.unwrap(),
                admin_nickname: owner.nickname,
                players,
            };

            Ok(game_info)
        })
    }

    //TODO: it's inconvenient to check everytime manually if the user is in the game
    // We should refactor it and make a common check.
    // Maybe even retrieve the game automatically?
    pub fn get_codenames(code: &String, player_id: i32) -> Result<Codenames> {
        let conn = db::connection()?;

        conn.transaction(|| {
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            //TODO: should we check whether the game is active or not?
            let is_user_in_game = playergame::table
                .filter(playergame::game.eq(requested_game.id))
                .filter(playergame::player.eq(player_id))
                .count()
                .get_result::<i64>(&conn)?
                > 0;

            if !is_user_in_game {
                info!(
                    "User {} is currently not in the requested game {}. Cannot get codenames",
                    player_id, code
                );
                return Err(ModelError::NotInGame);
            }

            let codenames = playergame::table
                .filter(playergame::game.eq(requested_game.id))
                .select(playergame::codename)
                .load(&conn)?;

            Ok(Codenames { codenames })
        })
    }

    pub fn get_end_time(code: &String, player_id: i32) -> Result<DateTime<Utc>> {
        let conn = db::connection()?;

        conn.transaction(|| {
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            //TODO: should we check whether the game is active or not?
            let is_user_in_game = playergame::table
                .filter(playergame::game.eq(requested_game.id))
                .filter(playergame::player.eq(player_id))
                .count()
                .get_result::<i64>(&conn)?
                > 0;

            if !is_user_in_game {
                info!(
                    "User {} is currently not in the requested game {}. Cannot get end time",
                    player_id, code
                );
                return Err(ModelError::NotInGame);
            }

            //TODO: all these requests will give back a server error.
            //We should create an enum of errors which the functions may return
            //and map them to a helpful error response
            if requested_game.end_time.is_none() {
                info!("Attempted to fetch time for a game which hasn't set it yet");
                return Err(ModelError::GameNotStarted);
            }

            Ok(requested_game.end_time.unwrap())
        })
    }

    pub fn kill_player(code: &String, player_id: i32) -> Result<()> {
        let conn = db::connection()?;

        conn.transaction(|| {
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            //TODO: should we check whether the game is active or not?
            let is_user_in_game = playergame::table
                .filter(playergame::game.eq(requested_game.id))
                .filter(playergame::player.eq(player_id))
                .count()
                .get_result::<i64>(&conn)?
                > 0;

            if !is_user_in_game {
                info!(
                    "User {} is currently not in the requested game {}. Cannot get kill",
                    player_id, code
                );
                return Err(ModelError::NotInGame);
            }

            let has_target = assignment::table
                .filter(assignment::game.eq(requested_game.id))
                .filter(assignment::assassin.eq(player_id))
                .filter(assignment::status.eq(TargetStatus::CURRENT))
                .count()
                .get_result::<i64>(&conn)?
                > 0;

            if !has_target {
                info!("User {} is has no target {}. Cannot kill", player_id, code);
                return Err(ModelError::NoCurrentTarget);
            }

            diesel::update(
                assignment::table
                    .filter(assignment::game.eq(requested_game.id))
                    .filter(assignment::assassin.eq(player_id))
                    .filter(assignment::status.eq(TargetStatus::CURRENT)),
            )
            .set(assignment::status.eq(TargetStatus::KILL_SUCCESS))
            .execute(&conn)?;

            Ok(())
        })
    }

    //    pub fn game_stats(code: &String, player_id: i32) -> Result<()> {
    //        //Diesel doesn't support GROUP BY queries in a many-to-many setting
    //        //This means we have to dirty our hands with raw SQL queries...
    //        let conn = db::connection()?;

    //        conn.transaction(|| {
    //            // let kill_counts: HashMap<i32, usize> = diesel::sql_query(
    //            //     "SELECT (assassin, COUNT(*))
    //            //     FROM assignment
    //            //     WHERE status = 'KILL_SUCCESS'
    //            //     GROUP BY assassin"
    //            // ).load::<PlayerCount>(&conn).into_iter().collect();

    //            // let death_counts: HashMap<i32, usize> = diesel::sql_query(
    //            //     "SELECT (assassin, COUNT(*))
    //            //     FROM assignment
    //            //     WHERE status = 'KILL_SUCCESS'
    //            //     GROUP BY target"
    //            // ).load(&conn).into_iter.collect();
    //            //
    //            let kill_counts = assignment::table
    //                .group_by(assignment::assassin)
    //                .filter(assignment::status.eq(TargetStatus::KILL_SUCCESS))
    //                .select((assignment::assassin, diesel::dsl::count(assignment::target)));

    //            // info!("Got the following query: {}", diesel::debug_query::<DB, _>(&kill_counts));

    //            // let requested_game: Game = game::table
    //            //     .filter(game::code.eq(code))
    //            //     .first(&conn)?;

    //            // let all_players = playergame::table
    //            //     .inner_join(player::table)
    //            //     .filter(playergame::game.eq(requested_game.id))
    //            //     .select((player::id, player::nickname, playergame::codename, player::picture))
    //            //     .load(&conn);

    //            // let game_stats = GameStats {

    //            // };

    //            Ok(())
    //            // Ok(game_stats)

    //        })

    //    }
}

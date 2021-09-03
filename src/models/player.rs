use actix_web::{dev::Payload, FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Identifiable, Insertable, Queryable};
use futures_util::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tracing::error;

use crate::db;
use crate::models::api_errors::ApiError;
use crate::models::enums::{GameStatus, PlayerStatus, Role, TargetStatus};
use crate::models::game::Game;
use crate::models::model_errors::{ModelError, Result};
use crate::utils::auth;

use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "player"]
pub struct NewPlayer {
    nickname: String,
    email: String,
    uid: String,
    role: Role,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[table_name = "player"]
pub struct Player {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub uid: String,
    pub role: Role,
    pub picture: Option<String>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub nickname: String,
    pub email: String,
    pub picture: Option<String>,
    pub active: bool,
    pub curr_lobby_code: Option<String>,
    pub total_kills: usize,
}

#[derive(Debug, Serialize)]
pub struct AgentInfo {
    codename: String,
    target: Option<String>,
    target_picture: Option<String>, //See discussion on nullable picture
    alive: bool,
    kills: usize,
}

#[derive(Debug, Serialize)]
pub struct AgentStats {
    nickname: String,
    codename: String,
    picture: Option<String>,
    kills: usize,
    deaths: usize,
    score: usize,
}

impl Player {
    pub fn find(id: i32) -> Result<Self> {
        let conn = db::connection()?;

        let player = player::table.filter(player::id.eq(id)).first(&conn)?;

        Ok(player)
    }

    pub fn find_by_uid(uid: &String) -> Result<Self> {
        let conn = db::connection()?;

        let account = player::table.filter(player::uid.eq(uid)).first(&conn)?;

        Ok(account)
    }

    pub fn register(nickname: String, email: String, uid: String) -> Result<Self> {
        let conn = db::connection()?;

        let new_account = NewPlayer {
            nickname,
            email,
            uid,
            role: Role::USER,
        };

        let res = diesel::insert_into(player::table)
            .values(new_account.clone())
            .get_result(&conn)
            .map_err(|_| ModelError::AlreadyRegistered)?;

        Ok(res)
    }

    pub fn get_user_info(&self) -> Result<UserInfo> {
        //Info about email, username, propic are already in `self`, so we query only the remaining

        let conn = db::connection()?;
        conn.transaction(|| {
            let active_games: Vec<String> = playergame::table
                .inner_join(game::table)
                .filter(playergame::player.eq(self.id))
                .filter(game::status.ne(GameStatus::FINISHED))
                .select(game::code)
                .distinct()
                .load(&conn)?;

            if active_games.len() > 1 {
                error!("User {:?} has a more than one active game", &self);
                return Err(ModelError::AlreadyInAnotherGame);
            }

            let has_active_game = active_games.len() > 0;

            let active_game_code = if active_games.len() > 0 {
                Some(active_games[0].clone())
            } else {
                None
            };

            let total_kills = assignment::table
                .inner_join(game::table)
                .filter(game::status.eq(GameStatus::FINISHED))
                .filter(assignment::assassin.eq(self.id))
                .filter(assignment::status.eq(TargetStatus::KILL_SUCCESS))
                .count()
                .get_result::<i64>(&conn)?;

            let user_info = UserInfo {
                email: self.email.clone(),
                nickname: self.nickname.clone(),
                picture: self.picture.clone(),
                active: has_active_game,
                curr_lobby_code: active_game_code,
                total_kills: usize::try_from(total_kills).unwrap(),
            };

            Ok(user_info)
        })
    }

    pub fn get_agent_info(&self, code: &String) -> Result<AgentInfo> {
        let conn = db::connection()?;
        conn.transaction(|| {
            //TODO: Should check here whether the game is finished or not?
            let requested_game: Game = game::table.filter(game::code.eq(code)).first(&conn)?;

            let (codename, status): (String, PlayerStatus) = playergame::table
                .filter(playergame::player.eq(self.id))
                .filter(playergame::game.eq(requested_game.id))
                .select((playergame::codename, playergame::status))
                .first(&conn)?;

            let alive = status == PlayerStatus::ALIVE;

            let target_info = assignment::table
                .inner_join(player::table.on(assignment::target.eq(player::id)))
                .filter(assignment::assassin.eq(self.id))
                .filter(assignment::status.eq(TargetStatus::CURRENT))
                .select((player::nickname, player::picture))
                .first(&conn);

            let (target_nickname, target_picture) = match target_info {
                Ok((nick, pic)) => (Some(nick), pic),
                Err(_) => (None, None),
            };

            let kills = assignment::table
                .filter(assignment::game.eq(requested_game.id))
                .filter(assignment::assassin.eq(self.id))
                .filter(assignment::status.eq(TargetStatus::KILL_SUCCESS))
                .count()
                .get_result::<i64>(&conn)?;

            let agent_info = AgentInfo {
                codename,
                target: target_nickname,
                target_picture,
                alive,
                kills: usize::try_from(kills).unwrap(),
            };

            Ok(agent_info)
        })
    }
}

impl FromRequest for Player {
    type Error = ApiError;
    type Future = Ready<std::result::Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let mut ext = req.extensions_mut();
        let claims = ext.remove::<auth::UserClaims>();
        match claims {
            Some(claims) => {
                let account = Player::find_by_uid(&claims.user_id);
                match account {
                    Ok(account) => ok(account),
                    Err(_) => err(ModelError::NotRegistered.into()),
                }
            }
            // This should never happen anyways
            None => err(ApiError::Unauthorized("MISSING_CLAIMS".to_string())),
        }
    }
}

use actix_web::{
    dev::Payload, error::ErrorForbidden, error::ErrorUnauthorized, Error, FromRequest, HttpRequest,
};
use chrono::{DateTime, Utc};
use color_eyre::Result;
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use futures_util::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::models::enums::Role;
use crate::utils::auth;

use crate::schema::*;

// type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "player"]
pub struct NewPlayer {
    nickname: String,
    email: String,
    uid: String,
    role: Role,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Player {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub uid: String,
    pub role: Role,
    pub picture: Option<String>,
    pub registered_at: DateTime<Utc>,
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

        diesel::insert_into(player::table)
            .values(new_account.clone())
            .get_result(&conn)
            .map_err(|e| {
                let err_str = format!(
                    "Failed in registering {}",
                    serde_json::to_string(&new_account).unwrap()
                );
                color_eyre::Report::new(e).wrap_err(err_str)
            })
    }
}

impl FromRequest for Player {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let mut ext = req.extensions_mut();
        let claims = ext.remove::<auth::UserClaims>();
        match claims {
            Some(claims) => {
                let account = Player::find_by_uid(&claims.user_id);
                match account {
                    Ok(account) => ok(account),
                    Err(_) => err(ErrorForbidden("You must register your account")),
                }
            }
            None => err(ErrorUnauthorized("Invalid token")),
        }
    }
}

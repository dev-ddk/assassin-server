use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::models::enums::Role;

use crate::schema::*;

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "account"]
pub struct NewAccount {
    nickname: String,
    email: String,
    uid: String,
    role: Role,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Account {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub uid: String,
    pub role: Role,
    pub created_at: NaiveDateTime,
}

impl Account {
    pub fn find(id: i32) -> Result<Self> {
        let conn = db::connection()?;

        let player = account::table.filter(account::id.eq(id)).first(&conn)?;

        Ok(player)
    }

    pub fn find_by_uid(uid: &String) -> Result<Self> {
        let conn = db::connection()?;

        let account = account::table.filter(account::uid.eq(uid)).first(&conn)?;

        Ok(account)
    }

    pub fn register(nickname: String, email: String, uid: String) -> Result<Account> {
        let conn = db::connection()?;

        let new_account = NewAccount {
            nickname,
            email,
            uid,
            role: Role::USER,
        };

        diesel::insert_into(account::table)
            .values(new_account)
            .get_result(&conn)
            .map_err(|e| eyre::Report::new(e))
    }
}
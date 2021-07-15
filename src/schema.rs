table! {
    use diesel::sql_types::*;
    use crate::models::enums::Role_t;
    account (id) {
        id -> Int4,
        nickname -> Varchar,
        email -> Varchar,
        uid -> Varchar,
        role -> Role_t,
        registered_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::Game_status_t;
    game (id) {
        id -> Int4,
        lobby_name -> Nullable<Varchar>,
        lobby_owner -> Int4,
        code -> Varchar,
        game_status -> Game_status_t,
    }
}

joinable!(game -> account (lobby_owner));

allow_tables_to_appear_in_same_query!(account, game,);

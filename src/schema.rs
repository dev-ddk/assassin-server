table! {
    game (id) {
        id -> Int4,
        lobby_name -> Nullable<Varchar>,
        lobby_owner -> Int4,
        code -> Varchar,
        game_status -> Game_status_t,
    }
}

table! {
    player (id) {
        id -> Int4,
        nickname -> Varchar,
        email -> Varchar,
        uid -> Varchar,
        role -> Role_t,
    }
}

joinable!(game -> player (lobby_owner));

allow_tables_to_appear_in_same_query!(game, player,);

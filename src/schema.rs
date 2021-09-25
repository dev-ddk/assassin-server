table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    assignment (assassin, target, game) {
        assassin -> Int4,
        target -> Int4,
        game -> Int4,
        status -> Target_status_t,
        start_time -> Timestamptz,
        end_time -> Nullable<Timestamptz>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    game (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        owner -> Int4,
        code -> Varchar,
        max_players -> Int4,
        status -> Game_status_t,
        created_at -> Timestamptz,
        start_time -> Nullable<Timestamptz>,
        end_time -> Nullable<Timestamptz>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    player (id) {
        id -> Int4,
        nickname -> Varchar,
        email -> Varchar,
        uid -> Varchar,
        role -> Role_t,
        picture -> Nullable<Varchar>,
        registered_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    playergame (player, game) {
        player -> Int4,
        game -> Int4,
        codename -> Varchar,
        status -> Player_status_t,
        joined_at -> Timestamptz,
    }
}

joinable!(assignment -> game (game));
joinable!(game -> player (owner));
joinable!(playergame -> game (game));
joinable!(playergame -> player (player));

allow_tables_to_appear_in_same_query!(
    assignment,
    game,
    player,
    playergame,
);

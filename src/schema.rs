table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    assignment (assassin, target, game) {
        assassin -> Int4,
        target -> Int4,
        game -> Int4,
        status -> Target_status_t,
        start_time -> Timestamp,
        end_time -> Nullable<Timestamp>,
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
        status -> Game_status_t,
        created_at -> Timestamp,
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
        registered_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    playergame (player, game) {
        player -> Int4,
        game -> Int4,
        target -> Nullable<Int4>,
        codename -> Varchar,
        status -> Player_status_t,
        target_status -> Target_status_t,
        joined_at -> Timestamp,
    }
}

joinable!(assignment -> game (game));
joinable!(game -> player (owner));
joinable!(playergame -> game (game));

allow_tables_to_appear_in_same_query!(assignment, game, player, playergame,);

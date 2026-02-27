// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Text,
        first_visit -> TimestamptzSqlite,
        last_visit -> TimestamptzSqlite,
    }
}

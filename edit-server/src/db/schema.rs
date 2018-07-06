table! {
    logs (rowid) {
        rowid -> Integer,
        source -> Text,
        body -> Text,
    }
}

table! {
    posts (id) {
        id -> Text,
        body -> Text,
    }
}

allow_tables_to_appear_in_same_query!(logs, posts,);

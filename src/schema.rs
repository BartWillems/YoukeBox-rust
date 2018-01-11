table! {
    rooms (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Varchar>,
        is_public -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        added_on -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    videos (id) {
        id -> Int4,
        video_id -> Varchar,
        title -> Varchar,
        description -> Nullable<Varchar>,
        room_id -> Int4,
        duration -> Varchar,
        played -> Bool,
        added_on -> Timestamp,
        started_on -> Nullable<Timestamp>,
    }
}

joinable!(videos -> rooms (room_id));

allow_tables_to_appear_in_same_query!(
    rooms,
    users,
    videos,
);

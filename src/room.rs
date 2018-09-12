use diesel;
use diesel::pg::PgConnection;
use regex::Regex;
use rocket::http::Status;
use rocket::response::Failure;
use super::schema::rooms;
use player::play_video_thread;

#[derive(Clone, Serialize, Deserialize, Queryable, Identifiable)]
pub struct Room {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Insertable, Deserialize)]
#[table_name = "rooms"]
pub struct NewRoom {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(FromForm)]
pub struct SearchRoom {
    pub name: String,
}

impl Room {
    #[inline]
    pub fn create(conn: &PgConnection, mut new_room: NewRoom) -> Result<Room, Failure> {
        use diesel::prelude::*;
        use diesel::result::Error;

        new_room.name = new_room.name.trim().to_string();

        let regex = Regex::new(r"^[[:word:]]{3,20}$").unwrap();

        if !regex.is_match(&new_room.name) {
            return Err(Failure(Status::BadRequest));
        }

        // I add the type here because othwerise the clone() doesn't know which type it is.
        let created_room: Result<Room, Error> = diesel::insert_into(rooms::table)
            .values(&new_room)
            .get_result(conn);

        match created_room {
            Ok(room) => {
                play_video_thread(room.clone());
                Ok(room)
            }
            Err(_) => Err(Failure(Status::Conflict)),
        }
    }

    #[inline]
    pub fn update(conn: &PgConnection, room: &Room) -> Result<Room, Failure> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;

        let regex = Regex::new(r"^[[:word:]]{3,20}$").unwrap();

        if !regex.is_match(&room.name) {
            return Err(Failure(Status::BadRequest));
        }

        let result = diesel::update(rooms)
            .set((
                description.eq(room.description.clone()),
                name.eq(room.name.clone()),
                is_public.eq(room.is_public),
            ))
            .get_result(conn);

        match result {
            Ok(updated_room) => Ok(updated_room),
            Err(_) => Err(Failure(Status::Conflict)),
        }
    }

    #[inline]
    pub fn delete(conn: &PgConnection, room_id: i64) -> Result<(), Failure> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;
        use std::fs;

        let result = diesel::delete(rooms.filter(id.eq(room_id))).execute(conn);

        if result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }

        let picture_url = format!("{}/{}", *super::PICTURES_DIR, room_id).to_string();

        let _res = fs::remove_file(picture_url);

        Ok(())
    }

    // Find & return a room by id
    #[inline]
    pub fn find(conn: &PgConnection, room_id: i64) -> Option<Room> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;

        let result = rooms.filter(id.eq(room_id)).first::<Room>(conn);

        match result {
            Ok(result) => Some(result),
            Err(_e) => None,
        }
    }

    // Return all rooms
    #[inline]
    pub fn all(conn: &PgConnection, query: Option<String>) -> Result<Vec<Room>, Failure> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;

        let result;

        match query {
            Some(query) => {
                result = rooms
                    .filter(name.ilike(format!("%{}%", query.to_lowercase())))
                    .order(name.desc())
                    .load::<Room>(conn);
            }
            None => {
                result = rooms.order(id.asc()).load::<Room>(conn);
            }
        }

        match result {
            Ok(result) => Ok(result),
            Err(e) => {
                println!("Error while fetching the rooms: {}", e);
                Err(Failure(Status::InternalServerError))
            }
        }
    }
}

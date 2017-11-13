extern crate diesel;

use diesel::pg::PgConnection;
use rocket::http::Status;
use rocket::response::Failure;
use super::schema::rooms;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Insertable)]
#[derive(Deserialize)]
#[table_name="rooms"]
pub struct NewRoom {
    pub name: String,
    pub description: Option<String>,
}

impl Room {
    #[inline]
    pub fn create(conn: &PgConnection, mut new_room: NewRoom) -> Result<Room, Failure> {
        use diesel::prelude::*;

        new_room.name = new_room.name.trim().to_string();

        if new_room.name.is_empty() {
            return Err(Failure(Status::BadRequest));
        }

        // Only allow  [a-Z], [0-9], ' ' & '_'
        for c in new_room.name.chars() {
            if !c.is_alphanumeric() && c != ' ' && c !=  '_' {
                return Err(Failure(Status::BadRequest));
            }
        }

        let result = diesel::insert(&new_room)
                        .into(rooms::table)
                        .get_result(conn);

        match result {
            Ok(result) => {
                Ok(result)
            },
            Err(_) => {
                Err(Failure(Status::InternalServerError))
            }
        }
    }

    pub fn delete(conn: &PgConnection, room_id: i32) -> Result<(), Failure> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;

        let result = diesel::delete(rooms.filter(id.eq(room_id)))
                        .execute(conn);

        match result {
            Ok(_result) => {
                Ok(())
            },
            Err(_) => {
                Err(Failure(Status::NotFound))
            }
        }
    }

    // Find & return a room by id
    #[inline]
    pub fn find(conn: &PgConnection, room_id: i32) -> Option<Room> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;

        let result = rooms.filter(id.eq(room_id)).first::<Room>(conn);

        match result {
            Ok(result) => {
                Some(result)
            },
            Err(_e) => {
                None
            }
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
                result = rooms.filter(name.like(format!("%{}%", query)))
                        .order(name.desc())
                        .load::<Room>(conn);
            },
            None => {
                result = rooms.order(name.desc()).load::<Room>(conn);
            }
        }

        match result {
            Ok(result) => {
                Ok(result)
            },
            Err(e) => {
                println!("Error while fetching the rooms: {}", e);
                Err(Failure(Status::InternalServerError))
            }
        }
    }
}
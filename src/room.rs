use diesel;
use diesel::pg::PgConnection;
use rocket::http::Status;
use rocket::response::Failure;
use super::schema::rooms;
use player::play_video_thread;

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

        if new_room.name.is_empty() {
            return Err(Failure(Status::BadRequest));
        }

        // Only allow  [a-Z], [0-9], ' ' & '_'
        for c in new_room.name.chars() {
            if !c.is_alphanumeric() && c != ' ' && c !=  '_' {
                return Err(Failure(Status::BadRequest));
            }
        }

        // I add the type here because othwerise the clone() doesn't know which type it is.
        let created_room: Result<Room, Error>
                = diesel::insert_into(rooms::table)
                    .values(&new_room)
                    .get_result(conn);

        match created_room {
            Ok(room) => {
                play_video_thread(room.clone());
                Ok(room)
            },
            Err(_) => {
                Err(Failure(Status::InternalServerError))
            }
        }
    }

    #[inline]
    pub fn update(conn: &PgConnection, room: &Room) -> Result<Room, Failure> {
        use diesel::prelude::*;
        use schema::rooms::dsl::*;

        let result = diesel::update(rooms)
                    .set((
                        description.eq(room.description.clone()),
                        name.eq(room.name.clone())
                    ))
                    .get_result(conn);

        match result {
            Ok(updated_room) => Ok(updated_room),
            Err(_) => Err(Failure(Status::InternalServerError))
        }
    }

    #[inline]
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
                Err(Failure(Status::InternalServerError))
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
                result = rooms.filter(
                            name.ilike(format!("%{}%", query.to_lowercase()))
                        )
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
extern crate diesel;

use diesel::pg::PgConnection;
use rocket::http::Status;
use rocket::response::Failure;
use std::time::SystemTime;
use models::Video;
use room::Room;

#[derive(Serialize)]
pub struct Playlist {
    pub videos: Vec<Video>,
    pub timestamp: Option<u64>,
}

impl Playlist {
    #[inline]
    pub fn get(conn: &PgConnection, r_id: i32) -> Result<Playlist, Failure> {
        use diesel::prelude::*;
        use schema::videos::dsl::*;

        let room = Room::find(conn, r_id);

        if room.is_none() {
            return Err(Failure(Status::NotFound))
        }

        let room = room.unwrap();

        let result = Video::belonging_to(&room)
                    .filter(played.eq(false))
                    .order(id)
                    .load::<Video>(conn);

        match result {
            Ok(result) => {
                let timestamp = get_timestamp(&result);
                Ok(Playlist {
                    videos: result,
                    timestamp: timestamp
                })
            },
            Err(e) => {
                println!("Error while fetching the playlist: {}", e);
                Err(Failure(Status::InternalServerError))
            }
        }
    }
}

fn get_timestamp(playlist: &[Video]) -> Option<u64> {
    if playlist.is_empty() {
        None
    } else {
        let started_on = playlist[0].started_on;

        println!("First song: {}", playlist[0].title);

        let now = SystemTime::now();
        let elapsed = now.duration_since(started_on.unwrap());

        match elapsed {
            Ok(elapsed) => Some(elapsed.as_secs()),
            Err(e) => {
                println!("Error while calulcating the playlist timestamp: {:?}", e);
                None
            }
        }
    }
}
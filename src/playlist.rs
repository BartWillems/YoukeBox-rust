extern crate diesel;

use diesel::pg::PgConnection;
use rocket::http::Status;
use rocket::response::Failure;
use std::time::SystemTime;
use models::Video;

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

        let room = super::room::Room::find(conn, r_id);

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
                Ok(set_playlist_timestamp(result))
            },
            Err(e) => {
                println!("Error while fetching the playlist: {}", e);
                Err(Failure(Status::InternalServerError))
            }
        }
    }
}

fn set_playlist_timestamp(playlist: Vec<Video>) -> Playlist {
    if ! playlist.is_empty() {
        let started_on = playlist[0].started_on;

        let now = SystemTime::now();
        let elapsed = now.duration_since(started_on.unwrap());

        match elapsed {
            Ok(elapsed) => {
                return Playlist {
                    videos: playlist,
                    timestamp: Some(elapsed.as_secs())
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Playlist {
                    videos: playlist,
                    timestamp: None
                }
            }
        }
    }

    Playlist {
        videos: playlist,
        timestamp: None
    }
}
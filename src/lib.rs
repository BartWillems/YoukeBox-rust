#![feature(plugin)]
#![plugin(dotenv_macros)]

#![recursion_limit="128"]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate serde_json;
extern crate reqwest;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use self::models::{NewVideo, NewRoom, Video, Playlist, Room, YoutubeVideos, YoutubeVideosDetailed};
use self::player::*;
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::response::{Failure};
use rocket::{Request, State, Outcome};
use std::time::SystemTime;
use std::io::Read;
use diesel::types;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub mod schema;
pub mod models;
pub mod player;
pub mod user;

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

sql_function!(lower, lower_t, (a: types::VarChar) -> types::VarChar);

lazy_static! {
	static ref API_KEY: &'static str = dotenv!("YOUTUBE_API_KEY");
	static ref API_URL: &'static str = "https://www.googleapis.com/youtube/v3";
	pub static ref APPLICATION_URL: &'static str = dotenv!("APPLICATION_URL");
}

// Return a single connection from the db pool
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
	type Error = ();

	fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
		let pool = request.guard::<State<Pool>>()?;
		match pool.get() {
			Ok(conn) => Outcome::Success(DbConn(conn)),
			Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
		}
	}
}

impl Deref for DbConn {
	type Target = PgConnection;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub fn establish_connection() -> PgConnection {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set");

	PgConnection::establish(&database_url)
		.expect(&format!("Error connecting to {}", database_url))
}

/// Create a new room
pub fn create_room<'a>(conn: &PgConnection, mut new_room: NewRoom) -> Result<Room, Failure> {
	use schema::rooms;

    new_room.name = new_room.name.trim().to_string();

    if new_room.name.len() == 0 {
        return Err(Failure(Status::BadRequest));
    }

    // Only allow  [a-Z], [0-9], ' ' & '_'
    for c in new_room.name.chars() {
        if !c.is_alphanumeric() && c != ' ' && c !=  '_' {
            return Err(Failure(Status::BadRequest));
        }
    }

	let room = diesel::insert(&new_room)
                .into(rooms::table)
                .get_result(conn);

    match room {
        Ok(room) => {
            let room: Room = room;
            play_video_thread(room.clone());
            return Ok(room);
        },
        Err(e) => {
            println!("Error while inserting a new room: {}", e);
            return Err(Failure(Status::Conflict));
        }
    }
}

/// List all the rooms
pub fn get_rooms<'a>(conn: &PgConnection, query: Option<String>) -> Vec<Room> {
	use self::schema::rooms::dsl::*;

	match query {
		Some(query) => {
            let room = query.trim().replace("%20", " ");

			rooms.filter(name.like(format!("%{}%", room)))
				.order(name)
				.load::<Room>(conn)
				.expect("Error while loading the rooms.")
		},
		None => {
			rooms.order(name)
				.load::<Room>(conn)
				.expect("Error while loading the rooms.")
		}
	}
}

/// Get a single room by name
pub fn get_room<'a>(conn: &PgConnection, room_name: &String) -> Option<Room> {
	use self::schema::rooms::dsl::*;

	let room = rooms.filter(lower(name).eq(room_name.to_lowercase()))
		.first::<Room>(conn);

	match room {
		Ok(room) => return Some(room),
		Err(_)	=> return None,
	}
}


/// Takes a string of youtube video id's seperated by a comma
/// eg: ssxNqBPRL6Y,_wy4tuFEpz0,...
/// Those videos will be searched on youtube and added to the videos db table
pub fn create_video<'a>(conn: &PgConnection, video_id: Vec<String>, room_name: String) -> Result<Vec<Video>, Failure> {
	use schema::videos;

	let mut videos: Vec<NewVideo> = Vec::new();
	let id_list = video_id.join(",");
    let room_name = room_name.trim().replace("%20", " ");

    let room;
    {
        let r = get_room(conn, &room_name);
        match r {
            Some(r) => {
                room = r;
            },
            None => {
                return Err(Failure(Status::NotFound));
            }
        }
    }


	let url = format!(
		"{}/videos?id={}&part=id,snippet,contentDetails&key={}", 
		*API_URL,
		id_list,
		*API_KEY
	);

	let resp = reqwest::get(&url);
	let mut content;

	match resp {
		Ok(mut resp) => {
			content = String::new();
			resp.read_to_string(&mut content).unwrap();
		},
		Err(_) => content = String::new(),
	}

	let result: YoutubeVideosDetailed = serde_json::from_str(&content).unwrap();

	for youtube_video in *result.items {
		let new_video = NewVideo {
			video_id: youtube_video.id.to_string(),
			title: youtube_video.snippet.title.to_string(),
			description: Some(youtube_video.snippet.description.to_string()),
			room_id: room.id,
			duration: youtube_video.contentDetails.duration.to_string(),
			added_on: SystemTime::now(),
		};

		videos.push(new_video);
	}

    let result = diesel::insert(&videos).into(videos::table)
    	.get_results(conn);

    match result {
        Ok(result) => {
            return Ok(result);
        },
        Err(e) => {
            println!("{}", e);
            return Err(Failure(Status::InternalServerError));
        }
    }
}

/// Get all videos as a vector
pub fn get_playlist<'a>(conn: &PgConnection, room_name: String) -> Result<Playlist, Failure> {
	use self::schema::videos::dsl::*;

    let room_name = room_name.replace("%20", " ").to_lowercase();
    let room = get_room(conn, &room_name);

    match room {
        Some(room) => {
            let result = videos.filter(played.eq(false))
                            .filter(room_id.eq(room.id))
                            .order(id)
                            .load::<Video>(conn);

            match result {
                Ok(result) => {
                    return Ok(set_playlist_timestamp(result))
                },
                Err(e) => {
                    println!("Error while fetching the playlist: {}", e);
                    return Err(Failure(Status::InternalServerError))
                }
            }
        },
        None => {
            return Err(Failure(Status::NotFound))
        }
    }
}


// 
fn set_playlist_timestamp(playlist: Vec<Video>) -> Playlist {
    if playlist.len() > 0 {
        let played_on = playlist[0].played_on;
        let now = SystemTime::now();
        let elapsed = now.duration_since(played_on.unwrap());

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

    return Playlist {
        videos: playlist,
        timestamp: None
    }
}

/// Returns a list of videos from Youtube
pub fn get_videos<'a>(query: &str) -> Option<String> {

	let url = format!(
		"{}/search?type=video&part=id,snippet&maxResults=20&key={}&q={}&videoCategoryId=10", 
		*API_URL,
		*API_KEY, 
		query);
	let resp = reqwest::get(&url);

	match resp {
		Ok(mut resp) => {
			let mut content = String::new();
			resp.read_to_string(&mut content).unwrap();
			return get_video_durations(Some(&content))
		},
		Err(_)	=> return None,
	}
}

/// Fetches the duration from Youtube for a list of videos
pub fn get_video_durations<'a>(json_videos: Option<&String>) -> Option<String> {
	let videos;
	let mut url: String = format!("{}/videos?id=", *API_URL).to_string();

	match json_videos {
		Some(json_videos) => {
			videos = Some(json_videos).unwrap();
		},
		None => return None
	}

	let result: YoutubeVideos = serde_json::from_str(&videos).unwrap();

	for youtube_video in *result.items {
		url = format!("{},{}", url, youtube_video.id.videoId);
	}

	url = format!("{}&part=id,snippet,contentDetails&key={}", url, *API_KEY);
	let resp = reqwest::get(&url);

	match resp {
		Ok(mut resp) => {
			let mut content = String::new();
			resp.read_to_string(&mut content).unwrap();
			return Some(content)
		},
		Err(_)	=> return None,
	}
}

pub fn init_pool() -> Pool {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set");

	let config = r2d2::Config::default();
	let manager = ConnectionManager::<PgConnection>::new(database_url);
	r2d2::Pool::new(config, manager).expect("db pool")
}
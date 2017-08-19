#![feature(plugin)]
#![plugin(dotenv_macros)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate reqwest;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use self::models::{NewVideo, NewRoom, Video, Room, YoutubeVideos, YoutubeVideosDetailed};
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use std::{thread, time};
use std::time::SystemTime;
use std::io::Read;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub mod schema;
pub mod models;

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

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

/// Fetches the current video from the playlist and waits for the duration of the video
/// Afterwards it updates the database and marks the video as played.
pub fn play_current_video<'a>(conn: &PgConnection, room_name: Option<String>) -> bool {
	use self::schema::videos::dsl::*;

	let video;
	match room_name.clone() {
		Some(room_name) => {
			video = videos
				.filter(played.eq(false))
				.filter(room.eq(room_name.to_lowercase()))
				.order(added_on)
				.first::<Video>(conn);
		},
		None => {
			video = videos
				.filter(played.eq(false))
				.filter(room.is_null())
				.order(added_on)
				.first::<Video>(conn);
		}
	};
	

	match video {
		Ok(video) => {
			let video_duration = time::Duration::from_secs(duration_to_seconds(&video.duration));

			diesel::update(&video)
				.set(played_on.eq(SystemTime::now()))
				.execute(conn)
				.expect("Unable to start playing the current video.");

			println!("Start playing: [{}] With ID: [{}] and duration: [{}] in room: [{:?}].", 
				&video.title, 
				&video.id, 
				&video.duration,
				room_name);

			// Wait until the video is played
			thread::sleep(video_duration);

			println!("Done playing [{}] from room [{:?}]", &video.title, room_name);

			// Mark the video as played
			diesel::update(&video)
				.set(played.eq(true))
				.execute(conn)
				.expect("Unable to mark the current video as played.");

			return true
		},
		Err(_) => return false,
	};
}


/// Start a thread to watch a certain playlist
pub fn play_video_thread<'a>(room: Option<String>) {
	thread::spawn(move  || {
       	let mut result;
       	let c = establish_connection();

    	println!("Room name: {:?}", room);
    	loop {
        	result = play_current_video(&c, room.clone());

        	if ! result {
        	   	// Wait 1 second before trying to play a new video
        	    thread::sleep(time::Duration::from_secs(1));
        	}
    	}
    });
}


/// Loop through every room & start playing their playlists
/// At the end of the loop, start the FFA playlist(room None)
pub fn init_playlist_listener<'a>() {
	use self::schema::rooms::dsl::*;

	let conn = establish_connection();

	let result = rooms.load::<Room>(&conn)
				.expect("Error loading videos");

	for room in result {
		play_video_thread(Some(room.name));
	}

	// Also play the FFA room
	play_video_thread(None);
}

/// Create a new room
pub fn create_room<'a>(conn: &PgConnection, mut r: NewRoom) -> Room {
	use schema::rooms;

	r.name = r.name.to_lowercase().replace(" ", "_").trim().to_string();

	let room = diesel::insert(&r).into(rooms::table)
		.get_result(conn)
		.expect("Error while inserting the room.");

	play_video_thread(Some(r.name));

	return room;
}

/// List all the rooms
pub fn get_rooms<'a>(conn: &PgConnection, query: Option<String>) -> Vec<Room> {
	use self::schema::rooms::dsl::*;

	match query {
		Some(query) => {
			rooms.filter(name.like(format!("%{}%", query)))
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

	let room = rooms.filter(name.eq(room_name.to_lowercase()))
		.first::<Room>(conn);

	match room {
		Ok(room) => return Some(room),
		Err(_)	=> return None,
	}
}


/// Takes a string of youtube video id's seperated by a comma
/// eg: ssxNqBPRL6Y,_wy4tuFEpz0,...
/// Those videos will be searched on youtube and added to the videos db table
pub fn create_video<'a>(conn: &PgConnection, video_id: Vec<String>, room: Option<String>) -> Vec<Video> {
	use schema::videos;

	let mut videos: Vec<NewVideo> = Vec::new();
	let id_list = video_id.join(",");
	
	let room_name;
	match room {
		Some(room) 	=> room_name = Some(room.to_lowercase()),
		None 		=> room_name = None,
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
			room: room_name.clone(),
			duration: youtube_video.contentDetails.duration.to_string(),
			added_on: SystemTime::now(),
		};

		videos.push(new_video);
	}

    diesel::insert(&videos).into(videos::table)
    	.get_results(conn)
    	.expect("Error while inserting the video in the playlist")

}

/// Get all videos as a vector
pub fn get_playlist<'a>(conn: &PgConnection, room_name: Option<String>) -> Option<Vec<Video>> {
	use self::schema::videos::dsl::*;

	let query = videos.filter(played.eq(false))
				.order(id);

	match room_name {
		Some(room_name) => {
			let r = get_room(conn, &room_name);
			match r {
				Some(_) => {
					let result = query.filter(room.eq(room_name.to_lowercase()))
						.load::<Video>(conn)
						.expect("Error loading videos");
					return Some(result)

				},
				None => return None,
			}
		},
		None => {
			let result = query.filter(room.is_null())
				.load::<Video>(conn)
				.expect("Error loading videos");

			return Some(result)
		},
	};
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

/// Returns a duration string as seconds
/// EG: "PT1H10M10S" -> 4210
pub fn duration_to_seconds(duration: &String) -> u64 {
	let v: Vec<&str> = duration.split(|c: char| !c.is_numeric()).collect();
	let mut index: u32 = 0;
	let mut tmp: i32 = 0;

	for i in (0..v.len()).rev() {
		if ! v[i].is_empty() {
			tmp += v[i].parse::<i32>().unwrap() * (60i32.pow(index));
			index += 1;
		}
	}

	return tmp as u64
}

pub fn init_pool() -> Pool {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set");

	let config = r2d2::Config::default();
	let manager = ConnectionManager::<PgConnection>::new(database_url);
	r2d2::Pool::new(config, manager).expect("db pool")
}
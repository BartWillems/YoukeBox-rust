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
use self::models::{Post, NewPost, Video, NewVideo, YoutubeVideos, YoutubeVideosDetailed};
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

pub fn create_post<'a>(conn: &PgConnection, title: &'a str, body: &'a str) -> Post {
	use schema::posts;

	let new_post = NewPost {
		title: title,
		body: body,
	};

	diesel::insert(&new_post).into(posts::table)
		.get_result(conn)
		.expect("Error saving post")
}

/// Fetches the current video from the playlist and waits for the duration of the video
/// Afterwards it updates the database and marks the video as played.
pub fn play_current_video<'a>(conn: &PgConnection) -> bool {
	use self::schema::videos::dsl::*;

	let video = videos.filter(played.eq(false))
		.first::<Video>(conn);

	match video {
		Ok(video) => {
			let video_duration = time::Duration::from_secs(duration_to_seconds(video.duration.clone()));

			// Wait until the video is played
			thread::sleep(video_duration);

			// Mark the video as played
			diesel::update(&video)
				.set(played.eq(false))
				.execute(conn)
				.expect("Unable to mark the current video as played.");

			return true
		},
		Err(_) => return false,
	}
}


/// Takes a string of youtube video id's seperated by a comma
/// eg: ssxNqBPRL6Y,_wy4tuFEpz0,...
/// Those videos will be searched on youtube and added to the videos db table
pub fn create_video<'a>(conn: &PgConnection, video_id: Vec<String>) -> Vec<Video> {
	use schema::videos;

	let mut videos: Vec<NewVideo> = Vec::new();
	let id_list = video_id.join(",");

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
			duration: youtube_video.contentDetails.duration.to_string(),
			added_on: SystemTime::now(),
		};

		videos.push(new_video);
	}

    diesel::insert(&videos).into(videos::table)
    	.get_results(conn)
    	.expect("Error while inserting the video in the playlist")

}

/// Get all posts as a vector
pub fn get_posts<'a>(conn: &PgConnection) -> Vec<Post> {
	use self::schema::posts::dsl::*;

	// Todo: 

	posts.filter(published.eq(false))
		// .limit(5)
		.load::<Post>(conn)
		.expect("Error loading posts")
}

/// Get a post by id, returns None when a post is not found
pub fn get_post<'a>(conn: &PgConnection, post_id: i32) -> Option<Post> {
	use self::schema::posts::dsl::*;

	let post = posts.find(post_id)
		.first::<Post>(conn);

	match post {
		Ok(post) => return Some(post),
		Err(_) => return None,
	}
}

/// Get all videos as a vector
pub fn get_playlist<'a>(conn: &PgConnection) -> Vec<Video> {
	use self::schema::videos::dsl::*;

	videos.filter(played.eq(false))
		.load::<Video>(conn)
		.expect("Error loading videos")
}

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
			return get_video_durations(Some(content))
		},
		Err(_)	=> return None,
	}
}

pub fn get_video_durations<'a>(json_videos: Option<String>) -> Option<String> {
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
pub fn duration_to_seconds(duration: String) -> u64 {
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
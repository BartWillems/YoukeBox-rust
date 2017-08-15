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
use self::models::{Post, NewPost, Video, NewVideo, YoutubeVideo};
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket_contrib::{Json};
use std::time::SystemTime;
use std::io::Read;


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub mod schema;
pub mod models;

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

lazy_static! {
	static ref API_KEY: &'static str = dotenv!("YOUTUBE_API_KEY");
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

pub fn create_video<'a>(conn: &PgConnection, youtube_video: Json<YoutubeVideo>) -> Video {
	use schema::videos;

	let new_video = NewVideo {
        video_id: youtube_video.id.videoId.to_string(),
        title: youtube_video.snippet.title.to_string(),
        description: Some(youtube_video.snippet.description.to_string()),
        duration: "PT4M13S".to_string(), // TODO: Fetch the duration from the YoutubeApi
        added_on: SystemTime::now(),
    };

    diesel::insert(&new_video).into(videos::table)
    	.get_result(conn)
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
		"https://www.googleapis.com/youtube/v3/search?part=id,snippet&maxResults=20&key={}&q={}", 
		*API_KEY, 
		query);
	let resp = reqwest::get(&url);

	match resp {
		Ok(mut resp) 	=>  {
			let mut content = String::new();
			resp.read_to_string(&mut content).unwrap();
			return Some(content)
		},
		Err(_)		=> return None,
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
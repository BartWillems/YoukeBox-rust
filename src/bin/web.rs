#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate diesel_demo;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

use rocket::response::status;
use rocket::http::RawStr;
use rocket_contrib::{Json};
use self::diesel_demo::*;
use self::diesel_demo::models::*;
use std::time::SystemTime;

#[get("/")]
fn index() -> &'static str {
    "It works!"
}

#[get("/playlist")]
fn show_playlist(conn: DbConn) -> Json<Vec<Video>> {
    Json(get_playlist(&conn))
}

#[post("/playlist", format = "application/json", data = "<youtube_video>")]
fn add_video(conn: DbConn, youtube_video: Json<YoutubeVideo>) -> status::Created<Json<NewVideo>> {
    
    let video = NewVideo {
        video_id: youtube_video.id.videoId.to_string(),
        title: youtube_video.snippet.title.to_string(),
        description: Some(youtube_video.snippet.description.to_string()),
        duration: "PT4M13S".to_string(), // TODO: Fetch the duration from the YoutubeApi
        added_on: SystemTime::now(),
    };


    return status::Created("".to_string(), Some(Json(video)))
}

#[get("/youtube/<video_id>")]
fn search_video(conn: DbConn, video_id: &RawStr) -> Option<String> {
    let res = get_videos(&conn, video_id);

    match res {
        Some(res)   => return Some(res),
        None        => return None,
    }
}

#[get("/posts/<id>")]
fn display_post(conn: DbConn, id: i32) -> Option<Json<Post>> {
    let post = get_post(&conn, id);

    match post {
        Some(post)  => return Some(Json(post)),
        None        => return None,
    }
}

#[post("/posts", format = "application/json", data = "<post>")]
fn add_post(conn: DbConn, post: Json<AddPost>) -> status::Created<Json<Post>> {
    let post = create_post(&conn, &post.title[..], &post.body[..]);

    return status::Created(format!("{}/posts/{}", *APPLICATION_URL, post.id), Some(Json(post)))
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index, add_post, display_post, show_playlist, search_video, add_video])
        .launch();
}
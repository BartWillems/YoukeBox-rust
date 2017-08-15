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

#[get("/")]
fn index() -> &'static str {
    "It works!"
}

#[get("/playlist")]
fn show_playlist(conn: DbConn) -> Json<Vec<Video>> {
    Json(get_playlist(&conn))
}

#[post("/playlist", data = "<video_id>")]
fn add_video(conn: DbConn, video_id: String) -> status::Created<Json<Vec<Video>>> {
    return status::Created("".to_string(), Some(Json(create_video(&conn, video_id))))
}

#[get("/youtube/<query>")]
fn search_video(query: &RawStr) -> Option<String> {
    let res = get_videos(query);

    match res {
        Some(res)   => return Some(res),
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
        .mount("/", routes![index, add_post, show_playlist, search_video, add_video])
        .launch();
}
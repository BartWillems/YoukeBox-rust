#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate youkebox;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

use rocket::response::status;
use rocket::http::RawStr;
use rocket_contrib::{Json};
use self::youkebox::*;
use self::youkebox::models::*;
use std::{thread, time};

#[get("/playlist")]
fn show_playlist(conn: DbConn) -> Json<Vec<Video>> {
    Json(get_playlist(&conn))
}

#[post("/playlist", format = "application/json", data = "<id_list>")]
fn add_video(conn: DbConn, id_list: String) -> status::Created<Json<Vec<Video>>> {
    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    return status::Created("".to_string(), Some(Json(create_video(&conn, videos))))
}

#[get("/youtube/<query>")]
fn search_video(query: &RawStr) -> Option<String> {
    let res = get_videos(query);

    match res {
        Some(res)   => return Some(res),
        None        => return None,
    }
}

fn main() {

    // Start the playlist watching thread
    thread::spawn(move  || {
        let mut result;
        let conn = establish_connection();
        loop {
            println!("Started playing video.");
            result = play_current_video(&conn);

            if ! result {
                // Wait 1 second before trying to play a new video
                thread::sleep(time::Duration::from_secs(1));
            }
        }
    });

    rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![show_playlist, search_video, add_video])
        .launch();
}
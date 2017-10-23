#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate youkebox;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate serde_json;

use rocket::response::{status, Failure};
use rocket::http::{Method,RawStr};
use rocket_contrib::Json;
use self::youkebox::*;
use self::youkebox::models::*;
use self::youkebox::player::{init_playlist_listener, skip_video};
// use self::youkebox::user::User;
use room::*;
use playlist::*;


// Youtube queries
#[get("/youtube/<query>")]
fn search_video(query: &RawStr) -> Option<String> {
    let res = get_videos(query);

    match res {
        Some(res)   => return Some(res),
        None        => return None,
    }
}

// Rooms
#[get("/rooms")]
fn show_rooms(conn: DbConn) -> Json<Vec<Room>> {
    let rooms = room::Room::all(&conn, None).unwrap();

    Json(rooms)
}

// Return a playlist for a room
#[get("/rooms/<room>")]
fn get_playlist(conn: DbConn, room: i32) -> Result<Json<Playlist>, Failure>{
    let playlist = playlist::Playlist::get(&conn, room);

    match playlist {
        Ok(playlist) => {
            Ok(Json(playlist))
        },
        Err(e) => {
            Err(e)
        }
    }
}

#[get("/rooms/search/<query>")]
fn show_rooms_filtered(conn: DbConn, query: &RawStr) -> Json<Vec<Room>> {
    let rooms = room::Room::all(&conn, Some(query.to_string())).unwrap();

    Json(rooms)
}

// Add a song to a room
#[post("/rooms/<room>", format = "application/json", data = "<id_list>")]
fn add_video(conn: DbConn, id_list: String, room: i32) -> Result<status::Created<Json<Vec<Video>>>, Failure> {

    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    let result = create_video(&conn, videos, room);

    match result {
        Ok(result) => {
            Ok(status::Created("".to_string(), Some(Json(result))))
        },
        Err(e) => {
            Err(e)
        }
    }
}

// Skip a song in a room
#[post("/rooms/<room>/skip")]
fn skip_song_in_room(room: i32) -> Json<HttpStatus> {

    skip_video(room);

    Json(HttpStatus{
        status: 200,
        message: "Successfully skipped the song".to_string(),
    })
}

#[post("/rooms", format = "application/json", data = "<room>")]
fn add_room(conn: DbConn, room: Json<NewRoom>) -> Result<Json<Room>, Failure> {

    let room = room::Room::create(&conn, room.into_inner());

    match room {
        Ok(room) => {
            Ok(Json(room))
        },
        Err(e) => {
            Err(e)
        }
    }
}

// Error pages
#[error(400)]
fn bad_request() -> Json<HttpStatus> {
    Json(HttpStatus{
        status: 400,
        message: "Bad Request".to_string(),
    })
}

#[error(404)]
fn not_found() -> Json<HttpStatus> {
    Json(HttpStatus{
        status: 404,
        message: "The requested resource was not found".to_string(),
    })
}

#[error(409)]
fn conflict() -> Json<HttpStatus> {
    Json(HttpStatus{
        status: 409,
        message: "Conflict".to_string(),
    })
}

#[error(500)]
fn internal_error() -> Json<HttpStatus> {
    Json(HttpStatus{
        status: 500,
        message: "Internal Server Error".to_string(),
    })
}

fn main() {
    // Start playing every playlist for every room
    init_playlist_listener();

    // Leave 'allowed_origins' empty because All is the default
    let options = rocket_cors::Cors {
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
        allow_credentials: true,
        ..Default::default()
    };

    rocket::ignite()
        .manage(init_pool())
        .mount("/api/v1", routes![
            get_playlist, 
            search_video, 
            add_video, 
            skip_song_in_room,
            show_rooms,
            show_rooms_filtered,
            add_room])
        .catch(errors![bad_request, not_found, conflict, internal_error])
        .attach(options)
        .launch();
}
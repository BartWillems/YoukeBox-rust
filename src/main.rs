#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate youkebox;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;

use rocket::response::status;
use rocket::http::RawStr;
use rocket_contrib::Json;
use self::youkebox::*;
use self::youkebox::models::*;
use self::youkebox::player::init_playlist_listener;

// Playlist pages

#[get("/playlist")]
fn show_playlist(conn: DbConn) -> Option<Json<Vec<Video>>> {
    let playlist = get_playlist(&conn, None);

    match playlist {
        Some(playlist) => return Some(Json(playlist)),
        None => return None,
    }
}

#[get("/playlist/<room>")]
fn show_room_playlist(conn: DbConn, room: &RawStr) -> Option<Json<Vec<Video>>> {
    let playlist = get_playlist(&conn, Some(room.to_string()));

    match playlist {
        Some(playlist) => return Some(Json(playlist)),
        None => return None,
    }
}

#[post("/playlist", format = "application/json", data = "<id_list>")]
fn add_video(conn: DbConn, id_list: String) -> status::Created<Json<Vec<Video>>> {
    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    return status::Created("".to_string(), Some(Json(create_video(&conn, videos, None))))
}

#[post("/playlist/<room>", format = "application/json", data = "<id_list>")]
fn add_video_to_room(conn: DbConn, id_list: String, room: &RawStr) -> status::Created<Json<Vec<Video>>> {
    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    return status::Created("".to_string(), Some(Json(create_video(&conn, videos, Some(room.to_string()) ) ) ) )
}

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
    Json(get_rooms(&conn, None))
}

#[get("/rooms/search/<query>")]
fn show_rooms_query(conn: DbConn, query: &RawStr) -> Json<Vec<Room>> {
    Json(get_rooms(&conn, Some(query.to_string())))
}

#[post("/rooms", format = "application/json", data = "<room>")]
fn add_room(conn: DbConn, room: Json<NewRoom>) -> Json<Room> {
    Json(create_room(&conn, room.into_inner() ))
}

// Error pages

#[error(404)]
fn not_found() -> Json<Error> {
    Json(Error{
        status: 404,
        message: "The requested resource was not found".to_string(),
    })
}

#[error(500)]
fn internal_error() -> Json<Error> {
    Json(Error{
        status: 500,
        message: "Internal Server Error".to_string(),
    })
}

fn main() {
    // Start playing every playlist for every room
    init_playlist_listener();

    rocket::ignite()
        .manage(init_pool())
        .mount("/api/v1", routes![
            show_playlist, 
            show_room_playlist, 
            search_video, 
            add_video, 
            add_video_to_room,
            show_rooms,
            show_rooms_query,
            add_room])
        .catch(errors![not_found, internal_error])
        .launch();
}
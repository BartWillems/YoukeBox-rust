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
use rocket_contrib::Json;
use self::youkebox::*;
use self::youkebox::models::*;

// Playlist pages

#[get("/api/v1/playlist")]
fn show_playlist(conn: DbConn) -> Option<Json<Vec<Video>>> {
    let playlist = get_playlist(&conn, None);

    match playlist {
        Some(playlist) => return Some(Json(playlist)),
        None => return None,
    }
}

#[get("/api/v1/playlist/<room>")]
fn show_room_playlist(conn: DbConn, room: &RawStr) -> Option<Json<Vec<Video>>> {
    let playlist = get_playlist(&conn, Some(room.to_string()));

    match playlist {
        Some(playlist) => return Some(Json(playlist)),
        None => return None,
    }
}

#[post("/api/v1/playlist", format = "application/json", data = "<id_list>")]
fn add_video(conn: DbConn, id_list: String) -> status::Created<Json<Vec<Video>>> {
    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    return status::Created("".to_string(), Some(Json(create_video(&conn, videos, None))))
}

#[post("/api/v1/playlist/<room>", format = "application/json", data = "<id_list>")]
fn add_video_to_room(conn: DbConn, id_list: String, room: &RawStr) -> status::Created<Json<Vec<Video>>> {
    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    return status::Created("".to_string(), Some(Json(create_video(&conn, videos, Some(room.to_string()) ) ) ) )
}

// Youtube queries

#[get("/api/v1/youtube/<query>")]
fn search_video(query: &RawStr) -> Option<String> {
    let res = get_videos(query);

    match res {
        Some(res)   => return Some(res),
        None        => return None,
    }
}

// Rooms
#[get("/api/v1/rooms")]
fn show_rooms(conn: DbConn) -> Json<Vec<Room>> {
    Json(get_rooms(&conn))
}

#[post("/api/v1/rooms", format = "application/json", data = "<room>")]
fn add_room(conn: DbConn, room: Json<NewRoom>) -> Json<Room> {
    Json(create_room(&conn, room.name.clone()))
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
        .mount("/", routes![
            show_playlist, 
            show_room_playlist, 
            search_video, 
            add_video, 
            add_video_to_room,
            show_rooms,
            add_room])
        .catch(errors![not_found, internal_error])
        .launch();
}
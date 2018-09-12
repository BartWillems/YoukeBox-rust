#![allow(unknown_lints, needless_pass_by_value)]

use DbConn;

use bytes::BufMut;
use image;
use image::GenericImage;
use rocket::http::Status;
use rocket::response::{content, status, Failure, NamedFile, Redirect};
use rocket::Data;
use rocket::State;
use rocket_contrib::Json;
use serde_json;
use std::fs::File;
use std::path::Path;

use http::HttpStatus;
use player::skip_video;
use playlist::*;
use room::*;
use video::*;
use youtube::*;

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/api/v1/")
}

#[get("/")]
fn api_index() -> &'static str {
    "
    You've arrived at the api endpoint 👌👌👌

    Go check out the documentation at our github page:
    https://github.com/BartWillems/YoukeBox-rust
    "
}

// Youtube queries
#[get("/youtube?<query>")]
fn search_video(
    api_key: State<ApiKey>,
    query: YoutubeQuery,
) -> Result<content::Json<String>, Failure> {
    let res = YoutubeVideo::search(&api_key.0.clone(), &query.query[..])?;

    Ok(content::Json(res))
}

// Rooms
#[get("/rooms")]
fn show_rooms(conn: DbConn) -> Json<Vec<Room>> {
    let rooms = Room::all(&conn, None).unwrap();
    Json(rooms)
}

#[get("/rooms?<room>")]
fn search_rooms(conn: DbConn, room: SearchRoom) -> Json<Vec<Room>> {
    let rooms = Room::all(&conn, Some(room.name))?;
    Json(rooms)
}

#[get("/rooms/<id>")]
fn show_room(conn: DbConn, id: i64) -> Option<Json<Room>> {
    let room = Room::find(&conn, id)?;

    Json(r)
}

// Return a playlist for a room
#[get("/rooms/<id>/playlist")]
fn get_playlist(conn: DbConn, id: i64) -> Result<Json<Playlist>, Failure> {
    let playlist = Playlist::get(&conn, id)?;
    Json(playlist)
}

// Add a song to a room
#[post("/rooms/<room>", format = "application/json", data = "<id_list>")]
fn add_video(
    api_key: State<ApiKey>,
    conn: DbConn,
    id_list: String,
    room: i64,
) -> Result<status::Created<Json<Vec<Video>>>, Failure> {
    let videos: Vec<String> = serde_json::from_str(&id_list)?;
    let result = YoutubeVideo::get(&api_key.0.clone(), &conn, &videos, room)?;
    status::Created("".to_string(), Json(result))
}

#[post("/rooms", format = "application/json", data = "<room>")]
fn add_room(conn: DbConn, room: Json<NewRoom>) -> Result<Json<Room>, Failure> {
    let room = Room::create(&conn, room.into_inner())?;
    Json(room)
}

// TODO:
// Create the picture when the room is created
#[post("/rooms/<id>/picture", data = "<picture_stream>")]
fn set_room_picture(id: i64, picture_stream: Data) -> Result<String, Failure> {
    use establish_connection;
    let con = establish_connection();

    if Room::find(&con, id).is_none() {
        return Err(Failure(Status::NotFound));
    }

    // 262144 bytes = max filesize for a 512x512 png
    let mut buf = Vec::with_capacity(262_144).writer();
    let res = picture_stream.stream_to(&mut buf);

    if res.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    let im = image::load_from_memory(buf.get_ref());
    let picture;
    let image_format;
    match im {
        Ok(im) => {
            picture = im;
            image_format = image::guess_format(buf.get_ref());
        }
        Err(_e) => return Err(Failure(Status::UnsupportedMediaType)),
    }

    if picture.width() > 512 || picture.height() > 512 {
        return Err(Failure(Status::BadRequest));
    }

    let picture_url = format!("{}/{}", *super::PICTURES_DIR, id).to_string();
    let picture_path = Path::new(&picture_url);

    let fout = &mut File::create(&picture_path).unwrap();
    let result = picture.save(fout, image_format.unwrap());

    match result {
        Ok(_r) => Ok(format!("/rooms/{}/picture", id)),
        Err(e) => {
            println!("Failed to save the picture: {}", e);
            Err(Failure(Status::InternalServerError))
        }
    }
}

// TODO:
// Set the picture url in the room table
// this way, we could do NamedFile::open(Path::new(room.picture))
// Because right now, we don't know if it's a jpg or png
#[get("/rooms/<id>/picture")]
fn get_room_picture(id: i64) -> Option<NamedFile> {
    let picture_url = format!("content/rooms/pictures/{}", id).to_string();
    NamedFile::open(Path::new(&picture_url)).ok()
}

#[put("/rooms", format = "application/json", data = "<room>")]
fn update_room(conn: DbConn, room: Json<Room>) -> Result<Json<Room>, Failure> {
    let result = Room::update(&conn, &room.into_inner());

    match result {
        Ok(new_room) => Ok(Json(new_room)),
        Err(e) => Err(e),
    }
}

#[delete("/rooms/<id>")]
fn delete_room(conn: DbConn, id: i64) -> Result<Json<HttpStatus>, Failure> {
    let result = Room::delete(&conn, id);

    match result {
        Ok(_result) => Ok(Json(HttpStatus {
            status: 200,
            message: "Successfully removed the room.".to_string(),
        })),
        Err(e) => Err(e),
    }
}

// Skip a song in a room
#[post("/rooms/<id>/skip")]
fn skip_song_in_room(id: i64) -> Json<HttpStatus> {
    skip_video(&id);

    Json(HttpStatus {
        status: 200,
        message: "Successfully skipped the song".to_string(),
    })
}

// Error pages
#[error(400)]
fn bad_request() -> Json<HttpStatus> {
    Json(HttpStatus {
        status: 400,
        message: "Bad Request".to_string(),
    })
}

#[error(404)]
fn not_found() -> Json<HttpStatus> {
    Json(HttpStatus {
        status: 404,
        message: "The requested resource was not found".to_string(),
    })
}

#[error(409)]
fn conflict() -> Json<HttpStatus> {
    Json(HttpStatus {
        status: 409,
        message: "Conflict".to_string(),
    })
}

#[error(415)]
fn unsupported_media_type() -> Json<HttpStatus> {
    Json(HttpStatus {
        status: 409,
        message: "Unsupported Media Type".to_string(),
    })
}

#[error(500)]
fn internal_error() -> Json<HttpStatus> {
    Json(HttpStatus {
        status: 500,
        message: "Internal Server Error".to_string(),
    })
}

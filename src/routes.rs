#![allow(unknown_lints, needless_pass_by_value)]

use DbConn;
use http::HttpStatus;
use rocket::Data;
use rocket::response::{content, status, Failure, Redirect, NamedFile};
use rocket::State;
use rocket_contrib::Json;
use serde_json;
use image;
use image::GenericImage;
use std::io;
use std::fs::File;
use std::fs;
use std::path::Path;

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
fn search_video(api_key: State<ApiKey>, query: YoutubeQuery) -> Result<content::Json<String>, Failure> {
    
    let res = YoutubeVideo::search(api_key.0.clone(), &query.query[..]);

    match res {
        Ok(res) => Ok(content::Json(res)),
        Err(e)  => Err(e),
    }
}

// Rooms
#[get("/rooms")]
fn show_rooms(conn: DbConn) -> Json<Vec<Room>> {
    let rooms = Room::all(&conn, None).unwrap();
    Json(rooms)
}

#[get("/rooms?<room>")]
fn search_rooms(conn: DbConn, room: SearchRoom) -> Json<Vec<Room>> {
    let rooms = Room::all(&conn, Some(room.name)).unwrap();
    Json(rooms)
}

#[get("/rooms/<id>")]
fn show_room(conn: DbConn, id: i32) -> Option<Json<Room>> {
    let room = Room::find(&conn, id);

    match room {
        Some(r) => return Some(Json(r)),
        None => return None
    }
}

// Return a playlist for a room
#[get("/rooms/<id>/playlist")]
fn get_playlist(conn: DbConn, id: i32) -> Result<Json<Playlist>, Failure>{
    let playlist = Playlist::get(&conn, id);

    match playlist {
        Ok(playlist) => {
            Ok(Json(playlist))
        },
        Err(e) => {
            Err(e)
        }
    }
}

// Add a song to a room
#[post("/rooms/<room>", format = "application/json", data = "<id_list>")]
fn add_video(api_key: State<ApiKey>, conn: DbConn, id_list: String, room: i32) -> Result<status::Created<Json<Vec<Video>>>, Failure> {

    let videos: Vec<String> = serde_json::from_str(&id_list).unwrap();
    let result = YoutubeVideo::get(api_key.0.clone(), &conn, &videos, room);

    match result {
        Ok(result) => {
            Ok(status::Created("".to_string(), Some(Json(result))))
        },
        Err(e) => {
            Err(e)
        }
    }
}


#[post("/rooms", format = "application/json", data = "<room>")]
fn add_room(conn: DbConn, room: Json<NewRoom>) -> Result<Json<Room>, Failure> {

    let room = Room::create(&conn, room.into_inner());

    match room {
        Ok(room) => {
            Ok(Json(room))
        },
        Err(e) => {
            Err(e)
        }
    }
}

// TODO:
// Actually detect if the picture is a picture
// Create the picture when the room is created
// The image library can detect if it's an image  when it's loaded from memory
#[post("/rooms/<id>/picture", data = "<picture>")]
fn set_room_picture(id: i32, picture: Data) -> io::Result<String> {
    // use std::io::Read;
    let picture_url = format!("content/rooms/pictures/{}.png", id).to_string();
    let picture_path = Path::new(&picture_url);

    // picture.stream()

    // Max amount of bytes for a 512x512 png
    // let mut buffer = [0; 1048576];
    // let mut stream = picture.open();
    // stream.read(&mut buffer);

    // let im = image::load_from_memory(&buffer);

    // Write the paste out to the file and return the URL.
    picture.stream_to_file(picture_path)?;

    // let im = image::load_from_memory(picture.peek());

    // match im {
    //     Ok(mut picture) => {
    //         let fout = &mut File::create(picture_url.clone()).unwrap();

    //         //  picture.resize(512, 512, DINK)

    //         if picture.width() > 512 || picture.height() > 512 {
    //             picture.crop(0, 0, 512, 512);
    //         }

    //         picture.save(fout, image::JPEG).unwrap();
    //     },
    //     Err(_) => {
    //         println!("Oh no we are in error");
    //         fs::remove_file(picture_url.clone()).unwrap();
    //     }
    // }
    
    Ok(picture_url.clone())
    
}

// TODO:
// Set the picture url in the room table
// this way, we could do NamedFile::open(Path::new(room.picture))
// Because right now, we don't know if it's a jpg or png
#[get("/rooms/<id>/picture")]
fn get_room_picture(id: i32) -> Option<NamedFile> {
    let picture_url = format!("content/rooms/pictures/{}", id).to_string();
    NamedFile::open(Path::new(&picture_url)).ok()
}

#[put("/rooms", format = "application/json", data = "<room>")]
fn update_room(conn: DbConn, room: Json<Room>) -> Result<Json<Room>, Failure> {
    let result = Room::update(&conn, &room.into_inner());

    match result {
        Ok(new_room) => Ok(Json(new_room)),
        Err(e) => Err(e)
    }
}

#[delete("/rooms/<id>")]
fn delete_room(conn: DbConn, id: i32) -> Result<Json<HttpStatus>, Failure> {
    let result = Room::delete(&conn, id);

    match result {
        Ok(_result) => {
            Ok(Json(HttpStatus{
                status: 200,
                message: "Successfully removed the room.".to_string(),
            }))
        },
        Err(e) => {
            Err(e)
        }
    }
}

// Skip a song in a room
#[post("/rooms/<id>/skip")]
fn skip_song_in_room(id: i32) -> Json<HttpStatus> {

    skip_video(&id);

    Json(HttpStatus{
        status: 200,
        message: "Successfully skipped the song".to_string(),
    })
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
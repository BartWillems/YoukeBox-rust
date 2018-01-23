#![feature(plugin)]
#![plugin(rocket_codegen)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate youkebox;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;

use youkebox::init_pool;
use youkebox::routes::*;
use youkebox::player::init_playlist_listener;

use rocket::http::Method;
use rocket::fairing::AdHoc;

use youkebox::youtube::ApiKey;


fn main() {
    // Start playing every playlist for every room
    init_playlist_listener();

    // Leave 'allowed_origins' empty because All is the default
    let options = rocket_cors::Cors {
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete].into_iter().map(From::from).collect(),
        allow_credentials: true,
        ..Default::default()
    };

    rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index])
        .mount("/api/v1", routes![
            api_index,
            get_playlist,
            search_video,
            add_video,
            skip_song_in_room,
            show_rooms,
            search_rooms,
            add_room,
            delete_room])
        .catch(errors![bad_request, not_found, conflict, internal_error])
        .attach(options)
        .attach(AdHoc::on_attach(|rocket| {
            let youtube_api_key = rocket.config()
                    .get_str("YOUTUBE_API_KEY")
                    .expect("YOUTUBE_API_KEY not set in Rocket.toml.")
                    .to_string();

            if youtube_api_key.is_empty() {
                panic!("Missing YOUTUBE_API_KEY.")
            }

            Ok(rocket.manage(ApiKey(youtube_api_key)))
        }))
        .launch();
}

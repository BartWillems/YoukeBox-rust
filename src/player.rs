use diesel::prelude::*;
use diesel::pg::PgConnection;
use self::models::{Video, Room};
use std::{thread, time};
use schema;
use models;
use std::time::SystemTime;

use establish_connection;

/// Fetches the current video from the playlist and waits for the duration of the video
/// Afterwards it updates the database and marks the video as played.
pub fn play_current_video<'a>(conn: &PgConnection, room_name: Option<String>) -> bool {
    use self::schema::videos::dsl::*;

    let video;
    match room_name.clone() {
        Some(room_name) => {
            video = videos
                .filter(played.eq(false))
                .filter(room.eq(room_name.to_lowercase()))
                .order(added_on)
                .first::<Video>(conn);
        },
        None => {
            video = videos
                .filter(played.eq(false))
                .filter(room.is_null())
                .order(added_on)
                .first::<Video>(conn);
        }
    };
    

    match video {
        Ok(video) => {
            let video_duration = time::Duration::from_secs(duration_to_seconds(&video.duration));

            super::diesel::update(&video)
                .set(played_on.eq(SystemTime::now()))
                .execute(conn)
                .expect("Unable to start playing the current video.");

            println!("Start playing: [{}] With ID: [{}] and duration: [{}] in room: [{:?}].", 
                &video.title, 
                &video.id, 
                &video.duration,
                room_name);

            // Wait until the video is played
            thread::sleep(video_duration);

            println!("Done playing [{}] from room [{:?}]", &video.title, room_name);

            // Mark the video as played
            super::diesel::update(&video)
                .set(played.eq(true))
                .execute(conn)
                .expect("Unable to mark the current video as played.");

            return true
        },
        Err(_) => return false,
    };
}


/// Start a thread to watch a certain playlist
pub fn play_video_thread<'a>(room: Option<String>) {
    thread::spawn(move  || {
        let mut result;
        let c = establish_connection();

        println!("Room name: {:?}", room);
        loop {
            result = play_current_video(&c, room.clone());

            if ! result {
                // Wait 1 second before trying to play a new video
                thread::sleep(time::Duration::from_secs(1));
            }
        }
    });
}


/// Loop through every room & start playing their playlists
/// At the end of the loop, start the FFA playlist(room None)
pub fn init_playlist_listener<'a>() {
    use self::schema::rooms::dsl::*;

    let conn = establish_connection();

    let result = rooms.load::<Room>(&conn)
                .expect("Error loading videos");

    for room in result {
        play_video_thread(Some(room.name));
    }

    // Also play the FFA room
    play_video_thread(None);
}

/// Returns a duration string as seconds
/// EG: "PT1H10M10S" -> 4210
pub fn duration_to_seconds(duration: &String) -> u64 {
    let v: Vec<&str> = duration.split(|c: char| !c.is_numeric()).collect();
    let mut index: u32 = 0;
    let mut tmp: i32 = 0;

    for i in (0..v.len()).rev() {
        if ! v[i].is_empty() {
            tmp += v[i].parse::<i32>().unwrap() * (60i32.pow(index));
            index += 1;
        }
    }

    return tmp as u64
}
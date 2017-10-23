use diesel::prelude::*;
use diesel::pg::PgConnection;
use models::Video;
use room::Room;
use std::{thread, time};
use schema;
use std::time::SystemTime;
use std::collections::HashMap;
use std::sync::Mutex;

use establish_connection;

lazy_static! {
    static ref PLAYLIST_THREADS: Mutex<HashMap<i32, VideoStatus>> = Mutex::new(HashMap::new());
}

enum VideoStatus {
    Play,
    Skip,
}

/// Fetches the current video from the playlist and waits for the duration of the video
/// Afterwards it updates the database and marks the video as played.
pub fn play_current_video<'a>(conn: &PgConnection, room: Room) -> bool {
    use self::schema::videos::dsl::*;

    let video = Video::belonging_to(&room)
                    .filter(played.eq(false))
                    .order(id)
                    .first::<Video>(conn);

    match video {
        Ok(video) => {
            let video_duration = time::Duration::from_secs(duration_to_seconds(&video.duration));

            super::diesel::update(&video)
                .set(started_on.eq(SystemTime::now()))
                .execute(conn)
                .expect("Unable to start playing the current video.");

            println!("Start playing: [{}] With ID: [{}] and duration: [{}] in room: [{}].", 
                &video.title, 
                &video.id, 
                &video.duration,
                &room.name);

            PLAYLIST_THREADS.lock().unwrap().insert(room.id.clone(), VideoStatus::Play);  

            let now = SystemTime::now();
            let mut playing: bool = true;

            // Continue playing this song while playing is true
            // Playing will be set to false if either the timer has run out
            // Or when someone skips the song by setting the PLAYLIST_THREADS[ROOM_NAME] to something other than "play"
            while playing {
                // Check if the video has ran out of time
                match now.elapsed() {
                    Ok(elapsed) => {
                        if elapsed.as_secs() >= video_duration.as_secs() {
                            playing = false;
                        }
                    },
                    Err(e) => {
                        playing = false;
                        println!("SystemTime elapsed error: {}", e);
                    }
                }

                let room = room.id.clone();
                // Check if someone tried to skip the video
                match PLAYLIST_THREADS.lock().unwrap().get(&room) {
                    Some(status) => {
                        playing = handle_video_event(&status);
                    },
                    None => {
                        PLAYLIST_THREADS.lock().unwrap().insert(room, VideoStatus::Play);
                    }
                }

                thread::sleep(time::Duration::from_millis(250));
            }

            println!("Done playing [{}] from room [{}]", &video.title, &room.name);

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

fn handle_video_event(status: &VideoStatus) -> bool {
    match status {
        &VideoStatus::Play => return true,
        &VideoStatus::Skip => return false
    };
}


/// Start a thread to watch a certain playlist
pub fn play_video_thread<'a>(room: Room) {
    thread::Builder::new()
        .name(room.name.clone())
        .spawn(move || {
            let mut result;
            let c = establish_connection();

            println!("Room name: {:?}", room.name.clone());
            loop {
                result = play_current_video(&c, room.clone());

                if ! result {
                    // Wait 1 second before trying to play a new video
                    thread::sleep(time::Duration::from_secs(1));
                }
            }
        })
        .unwrap();
}


/// Loop through every room & start playing their playlists
/// At the end of the loop, start the FFA playlist(room None)
pub fn init_playlist_listener<'a>() {
    use self::schema::rooms::dsl::*;

    let conn = establish_connection();

    let result = rooms.load::<Room>(&conn)
                .expect("Error loading videos");

    for room in result {
        PLAYLIST_THREADS.lock().unwrap().insert(room.id,VideoStatus::Play);
        play_video_thread(room);
    }
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


pub fn skip_video(room: i32) {
    let mut map = PLAYLIST_THREADS.lock().unwrap();

    println!("Skipping a song in room [{}]", room);

    if let Some(mut_key) = map.get_mut(&room) {
        *mut_key = VideoStatus::Skip;
    } else {
        println!("Invalid room, could not skip song.");
    }
}
use diesel::prelude::*;
use diesel::pg::PgConnection;
use video::Video;
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
pub fn play_current_video(conn: &PgConnection, room: &Room) -> bool {
    use self::schema::videos::dsl::*;

    let video = Video::belonging_to(room)
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

            PLAYLIST_THREADS.lock().unwrap().insert(room.id, VideoStatus::Play);

            let now = SystemTime::now();
            let mut playing: bool = true;

            // Continue playing this song while playing is true
            // Playing will be set to false if either the timer has run out
            // Or when someone skips the song by setting the PLAYLIST_THREADS[ROOM_NAME] to something other than "play"
            while playing {

                // Check if someone tried to skip the video
                match PLAYLIST_THREADS.lock().unwrap().get(&room.id) {
                    Some(status) => {
                        playing = handle_video_event(status);
                    },
                    None => {
                        PLAYLIST_THREADS.lock().unwrap().insert(room.id, VideoStatus::Play);
                    }
                }

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

                thread::sleep(time::Duration::from_millis(250));
            }

            println!("Done playing [{}] from room [{}]", &video.title, &room.name);

            // Mark the video as played
            super::diesel::update(&video)
                .set(played.eq(true))
                .execute(conn)
                .expect("Unable to mark the current video as played.");

            true
        },
        Err(_) => {
            stop_playing(room);
            false
        },
    }
}

fn handle_video_event(status: &VideoStatus) -> bool {
    match *status {
        VideoStatus::Play => true,
        VideoStatus::Skip => false
    }
}


/// Start a thread to watch a certain playlist
pub fn play_video_thread(room: Room) {
    thread::Builder::new()
        .spawn(move || {
            let c = establish_connection();

            loop {
                play_current_video(&c, &room);

                if ! PLAYLIST_THREADS.lock().unwrap().contains_key(&room.id) {
                    println!("Stop playin thread with id: {}", room.id );
                    break;
                }
            }
        })
        .unwrap();
}


// Loop through every room & start playing their playlists IF the playlist isn't empty.
// At the end of the loop, start the FFA playlist(room None)
pub fn init_playlist_listener() {
    use self::schema::rooms::dsl::*;

    use playlist::Playlist;

    let conn = establish_connection();

    let result = rooms.load::<Room>(&conn)
                .expect("Error loading videos");

    for room in result {
        if Playlist::is_empty(&conn, &room) {
            continue;
        }
        start_playing(room);
    }
}

pub fn start_playing(room: Room) {
    let mut hashmap = PLAYLIST_THREADS.lock().unwrap();

    if ! hashmap.contains_key(&room.id) {
        hashmap.insert(room.id, VideoStatus::Play);
        play_video_thread(room);
    }
}

pub fn stop_playing(room: &Room) {
    PLAYLIST_THREADS.lock().unwrap().remove(&room.id);
}

// Returns a duration string as seconds
// EG: "PT1H10M10S" -> 4210
pub fn duration_to_seconds(duration: &str) -> u64 {
    let v: Vec<&str> = duration.split(|c: char| !c.is_numeric()).collect();
    let mut index: u32 = 0;
    let mut total: i32 = 0;

    for i in (0..v.len()).rev() {
        if ! v[i].is_empty() {
            total += v[i].parse::<i32>().unwrap() * (60i32.pow(index));
            index += 1;
        }
    }

    total as u64
}


pub fn skip_video(room: &i32) {
    let mut rooms = PLAYLIST_THREADS.lock().unwrap();

    println!("Skipping a song in room [{}]", room);

    if let Some(mut_key) = rooms.get_mut(room) {
        *mut_key = VideoStatus::Skip;
    } else {
        println!("Invalid room, could not skip song.");
    }
}
use diesel::pg::PgConnection;

use video::{NewVideo, Video};
use room::Room;
use rocket::http::Status;
use rocket::response::Failure;
use std::time::SystemTime;
use std::io::Read;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideoId {
    pub kind: String,
    pub videoId: String,
}

#[derive(Deserialize)]
pub struct YoutubeVideoThumbnail {
    pub url: String,
    pub width: i16,
    pub height: i16,
}

#[derive(Deserialize)]
pub struct YoutubeVideoThumbnails {
    pub default: Box<YoutubeVideoThumbnail>,
    pub medium: Box<Option<YoutubeVideoThumbnail>>,
    pub high: Box<Option<YoutubeVideoThumbnail>>,
    pub standard: Box<Option<YoutubeVideoThumbnail>>,
    pub maxres: Box<Option<YoutubeVideoThumbnail>>,
}

#[derive(Deserialize)]
pub struct Localized {
    pub title: String,
    pub description: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideoSnippet {
    pub publishedAt: String,
    pub channelId: String,
    pub title: String,
    pub description: String,
    pub thumbnails: Box<YoutubeVideoThumbnails>,
    pub channelTitle: String,
    pub tags: Option<Vec<String>>,
    pub categoryId: Option<String>,
    pub liveBroadcastContent: String,
    pub defaultLanguage: Option<String>,
    pub localized: Box<Option<Localized>>,
    pub defaultAudioLanguage: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideo {
    pub kind: String,
    pub etag: String,
    pub id: Box<YoutubeVideoId>,
    pub snippet: Box<YoutubeVideoSnippet>,
    pub ContentDetails: Box<Option<ContentDetails>>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideoDetailed {
    pub kind: String,
    pub etag: String,
    pub id: String,
    pub snippet: Box<YoutubeVideoSnippet>,
    pub contentDetails: Box<ContentDetails>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ContentDetails {
    pub duration: String,
    pub dimension: Option<String>,
    pub definition: Option<String>,
    pub caption: Option<String>,
    pub licensedContent: Option<bool>,
    pub regionRestriction: Option<String>,
    pub projection: String,
    pub contentRating: Option<String>,
    pub hasCustomThumbnail: Option<bool>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct PageInfo {
    pub totalResults: u32,
    pub resultsPerPage: u8,
}

// This is the full result from the youtube search api
#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideos {
    pub kind: String,
    pub etag: String,
    pub nextPageToken: Option<String>,
    pub regionCode: Option<String>,
    pub pageInfo: Box<Option<PageInfo>>,
    pub items: Vec<YoutubeVideo>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideosDetailed {
    pub kind: String,
    pub etag: String,
    pub nextPageToken: Option<String>,
    pub regionCode: Option<String>,
    pub pageInfo: Box<PageInfo>,
    pub items: Vec<YoutubeVideoDetailed>,
}

impl YoutubeVideo {
    /// Returns a list of videos from Youtube
    #[inline]
    pub fn search(query: &str) -> Option<String> {
        use reqwest;

        let url = format!(
            "{}/search?type=video&part=id,snippet&maxResults=20&key={}&q={}&videoCategoryId=10",
            *super::API_URL,
            *super::API_KEY,
            query);
        let resp = reqwest::get(&url);

        match resp {
            Ok(mut resp) => {
                let mut content = String::new();
                resp.read_to_string(&mut content).unwrap();
                YoutubeVideo::get_video_durations(Some(&content))
            },
            Err(_)  => None,
        }
    }

    // Fetches the duration from Youtube for a list of videos
    fn get_video_durations(json_videos: Option<&String>) -> Option<String> {
        use serde_json;
        use reqwest;

        let videos;
        let mut url: String = format!("{}/videos?id=", *super::API_URL).to_string();

        match json_videos {
            Some(json_videos) => {
                videos = Some(json_videos).unwrap();
            },
            None => return None
        }

        let result: YoutubeVideos = serde_json::from_str(videos).unwrap();

        for youtube_video in &result.items {
            url = format!("{},{}", url, youtube_video.id.videoId);
        }

        url = format!("{}&part=id,snippet,contentDetails&key={}", url, *super::API_KEY);
        let resp = reqwest::get(&url);

        match resp {
            Ok(mut resp) => {
                let mut content = String::new();
                resp.read_to_string(&mut content).unwrap();
                Some(content)
            },
            Err(_)  => None,
        }
    }
    // Takes a string of youtube video id's seperated by a comma
    // eg: ssxNqBPRL6Y,_wy4tuFEpz0,...
    // Those videos will be searched on youtube and added to the videos db table
    pub fn get(conn: &PgConnection, video_id: &[String], room_id: i32) -> Result<Vec<Video>, Failure> {
        use schema::videos;
        use reqwest;
        use serde_json;
        use diesel;
        use diesel::RunQueryDsl;

        let mut videos: Vec<NewVideo> = Vec::new();
        let id_list = video_id.join(",");

        let room = Room::find(conn, room_id);

        if room.is_none() {
            return Err(Failure(Status::NotFound));
        }

        let room = room.unwrap();

        let url = format!(
            "{}/videos?id={}&part=id,snippet,contentDetails&key={}",
            *super::API_URL,
            id_list,
            *super::API_KEY
        );

        let resp = reqwest::get(&url);
        let mut content = String::new();

        match resp {
            Ok(mut resp) => {
                resp.read_to_string(&mut content).unwrap();
            },
            Err(_) => return Err(Failure(Status::InternalServerError))
        }

        let result: YoutubeVideosDetailed = serde_json::from_str(&content).unwrap();

        for youtube_video in &result.items {
            let new_video = NewVideo {
                video_id: youtube_video.id.to_string(),
                title: youtube_video.snippet.title.to_string(),
                description: Some(youtube_video.snippet.description.to_string()),
                room_id: room.id,
                duration: youtube_video.contentDetails.duration.to_string(),
                added_on: SystemTime::now(),
            };

            videos.push(new_video);
        }

        let result = diesel::insert_into(videos::table)
                        .values(&videos)
                        .get_results(conn);

        match result {
            Ok(result) => {
                Ok(result)
            },
            Err(e) => {
                println!("{}", e);
                Err(Failure(Status::InternalServerError))
            }
        }
    }
}
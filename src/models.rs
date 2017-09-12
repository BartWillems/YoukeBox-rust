use super::schema::videos;
use super::schema::rooms;
use std::time::SystemTime;


// Nullable SQL types should be an Option struct
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
pub struct Video {
    pub id: i32,
    pub video_id: String,
    pub title: String,
    pub description: Option<String>,
    pub room: Option<String>,
    pub duration: String,
    pub played: bool,
    pub added_on: SystemTime,
    pub played_on: Option<SystemTime>,
}

#[derive(Insertable)]
#[derive(Serialize)]
#[table_name="videos"]
pub struct NewVideo {
    pub video_id: String,
    pub title: String,
    pub description: Option<String>,
    pub room: Option<String>,
    pub duration: String,
    pub added_on: SystemTime,
}

#[derive(Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Insertable)]
#[derive(Deserialize)]
#[table_name="rooms"]
pub struct NewRoom {
    pub name: String,
    pub description: Option<String>,
}

// Start with the Youtube models

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
    pub items: Box<Vec<YoutubeVideo>>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct YoutubeVideosDetailed {
    pub kind: String,
    pub etag: String,
    pub nextPageToken: Option<String>,
    pub regionCode: Option<String>,
    pub pageInfo: Box<PageInfo>,
    pub items: Box<Vec<YoutubeVideoDetailed>>,
}

#[derive(Serialize)]
pub struct Error {
    pub status: u16,
    pub message: String,
}
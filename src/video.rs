// THIS IS NOT YET INCLUDED (23/10/2017)

use super::schema::videos;
use room::Room;
use std::time::SystemTime;


// Nullable SQL types should be an Option struct
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Room)]
pub struct Video {
    pub id: i32,
    pub video_id: String,
    pub title: String,
    pub description: Option<String>,
    pub room_id: i32,
    pub duration: String,
    pub played: bool,
    pub added_on: SystemTime,
    pub started_on: Option<SystemTime>,
}

#[derive(Insertable)]
#[derive(Serialize)]
#[table_name="videos"]
pub struct NewVideo {
    pub video_id: String,
    pub title: String,
    pub description: Option<String>,
    pub room_id: i32,
    pub duration: String,
    pub added_on: SystemTime,
}
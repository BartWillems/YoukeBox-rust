# YoukeBox-Rust

This is the YoukeBox backend, written in rust.

## What is `The YoukeBox`?

The YoukeBox is an application that multiple people can use to build music playlists.
Users can create a music-room and submit songs from youtube to that room.

## Routes

Each route has a prefix: "/api/$version/"

**GET**

* /rooms
    * Display every room
* /rooms?\<room\>
    * Search for rooms with a query. eg: /rooms?name=death
* /room/\<id\>
    * Display the playlist for the room with id: \<id\>
* /youtube/\<query\>
    * Search songs on youtube

**POST**

* /rooms/\<id\>
    * Add songs to the room with id:  \<id\>
    * Format: "application/json"
    * [ "ZnJVcuUDnW4" ]
* /rooms/\<id\>/skip
    * Skip a song in the room with id: \<id\>
    * i32: 4
* /rooms
    * Add a new room
    * Format: "application/json"
    * { name: "room name", description: "room description" }

**DELETE**

* /rooms/\<id\>
    * Delete the room with id: \<id\>
    * i32: 4

## What does the YoukeBox support atm?

* Pseudo live-streaming
* Creating rooms
* Skipping songs

## Coming soon

* Music ordering
* Accounts support with room administrators
* Upvotes & Downvotes

## Developer notes

I will add the following structs with the following impl methods in the near future

* youtube.rs
    * search(query) -> Vec<YoutubeVideo>
    * verify(YoutubeVideoId) -> Result<YoutubeVideo, Failure>

* video.rs
    * add_to_playlist(room) -> (?)


## Running in production
```
ROCKET_ENV=production cargo run
```